use chrono::Utc;
use rdkafka::{Offset, consumer::StreamConsumer};
use tracing::instrument;

use crate::{
    cli_structs::Steps,
    finder::{
        SearchResults, SearchStatus, SearchTarget,
        searcher::Searcher,
        task::{SearchTask, TaskClass},
    },
    messages::{Cache, FBMessage},
};

pub(crate) struct SearchByTimestamp;
impl TaskClass for SearchByTimestamp {}

impl<'a> SearchTask<'a, SearchByTimestamp> {
    ///
    #[instrument(skip_all)]
    async fn search_topic<M, E, A, G>(
        &self,
        searcher: Searcher<'a, M, StreamConsumer, G>,
        target: &SearchTarget,
        emit: E,
        acquire_while: A,
    ) -> (Vec<M>, i64)
    where
        E: Fn(u32) -> SearchStatus,
        M: FBMessage<'a>,
        A: Fn(&M) -> bool,
        G: Fn(i64) -> Offset,
    {
        self.emit_status(emit(0)).await;

        let mut iter = searcher.iter_backstep();
        for step in 0..steps.num_step_passes {
            self.emit_status(emit(step)).await;
            let sz =
                steps.min_step_size * steps.step_mul_coef.pow(steps.num_step_passes - 1 - step);
            iter.step_size(sz)
                .backstep_until_time(|t| t > target.timestamp)
                .await
                .expect("");
        }

        self.emit_status(emit(steps.num_step_passes)).await;

        let searcher = iter.collect();
        let offset = searcher.get_offset();

        let results: Vec<M> = searcher
            .iter_forward()
            .move_until(|t| t >= target.timestamp)
            .await
            .acquire_while(acquire_while, target.number)
            .await
            .collect()
            .into();

        (results, offset)
    }

    /// Performs a FromEnd search.
    /// # Attributes
    /// - target: what to search for.
    #[instrument(skip_all)]
    pub(crate) async fn search(self, target: SearchTarget) -> (StreamConsumer, SearchResults) {
        let start = Utc::now();

        let mut cache = Cache::default();

        // Find Digitiser Traces
        let searcher = Searcher::new(
            &self.consumer,
            &self.topics.trace_topic,
            1,
            Offset::OffsetTail,
            self.send_status.clone(),
        )
        .expect("");
        let (trace_results, offset) = self
            .search_topic(
                searcher,
                &target,
                SearchStatus::TraceSearchInProgress,
                |msg| target.filter_trace_by_channel_and_digtiser_id(msg),
            )
            .await;
        self.emit_status(SearchStatus::TraceSearchFinished).await;

        // Find Digitiser Event Lists
        let searcher = Searcher::new(
            &self.consumer,
            &self.topics.digitiser_event_topic,
            offset,
            Offset::OffsetTail,
            self.send_status.clone(),
        )
        .expect("");
        let (eventlist_results, _) = self
            .search_topic(
                searcher,
                &target,
                SearchStatus::EventListSearchInProgress,
                |msg| target.filter_eventlist_digtiser_id(msg),
            )
            .await;
        self.emit_status(SearchStatus::EventListSearchFinished)
            .await;

        for trace in trace_results.iter() {
            cache.push_trace(&trace.get_unpacked_message().expect(""));
        }

        for eventlist in eventlist_results.iter() {
            cache.push_events(&eventlist.get_unpacked_message().expect(""));
        }
        cache.attach_event_lists_to_trace();

        // Send cache via status
        self.emit_status(SearchStatus::Successful).await;
        let time = Utc::now() - start;
        (self.consumer, SearchResults { cache, time })
    }
}

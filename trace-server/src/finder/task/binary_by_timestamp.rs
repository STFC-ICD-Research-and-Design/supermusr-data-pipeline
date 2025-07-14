use crate::{
    Timestamp,
    finder::{
        SearchResults, SearchStatus, SearchTargetBy,
        searcher::Searcher,
        task::{SearchTask, TaskClass},
    },
    messages::{Cache, EventListMessage, FBMessage, TraceMessage},
};
use chrono::Utc;
use rdkafka::{Offset, consumer::StreamConsumer};
use tracing::instrument;

/// Size of each backstep when a target timestamp has been found
const BACKSTEP_SIZE: i64 = 32;

pub(crate) struct BinarySearchByTimestamp;
impl TaskClass for BinarySearchByTimestamp {}

impl<'a> SearchTask<'a, BinarySearchByTimestamp> {
    /// Performs a binary tree search on a given topic, with generic filtering functions.
    #[instrument(skip_all)]
    async fn search_topic<M, E, A, G>(
        &self,
        searcher: Searcher<'a, M, StreamConsumer, G>,
        target: Timestamp,
        number: usize,
        emit: E,
        acquire_while: A,
    ) -> (Vec<M>, i64)
    where
        E: Fn(f64) -> SearchStatus,
        M: FBMessage<'a>,
        A: Fn(&M) -> bool,
        G: Fn(i64) -> Offset,
    {
        let mut iter = searcher.iter_binary(target);
        iter.init().await;

        loop {
            self.emit_status(emit(iter.get_progress())).await;
            if iter.bisect().await.expect("") {
                break;
            }
        }

        self.emit_status(emit(1.0)).await;
        let searcher = iter.collect();
        let offset = searcher.get_offset();

        let mut iter = searcher.iter_backstep();
        iter.step_size(BACKSTEP_SIZE)
            .backstep_until_time(|t| t > target)
            .await
            .expect("");

        let searcher = iter.collect();

        let results: Vec<M> = searcher
            .iter_forward()
            .move_until(|t| t >= target)
            .await
            .acquire_while(acquire_while, number)
            .await
            .collect()
            .into();

        (results, offset)
    }

    /// Performs a FromEnd search.
    /// # Parameters
    /// - target: what to search for.
    #[instrument(skip_all)]
    pub(crate) async fn search(
        self,
        target: Timestamp,
        by: SearchTargetBy,
        number: usize,
    ) -> SearchResults {
        let start = Utc::now();

        let mut cache = Cache::default();

        // Find Digitiser Traces
        let searcher =
            Searcher::new(self.consumer, &self.topics.trace_topic, 1, Offset::Offset).expect("");

        let (trace_results, offset) = self
            .search_topic(
                searcher,
                target,
                number,
                SearchStatus::TraceSearchInProgress,
                |msg: &TraceMessage| msg.filter_by(&by),
            )
            .await;
        self.emit_status(SearchStatus::TraceSearchFinished).await;

        let digitiser_ids = {
            let mut digitiser_ids = trace_results
                .iter()
                .map(TraceMessage::digitiser_id)
                .collect::<Vec<_>>();
            digitiser_ids.sort();
            digitiser_ids.dedup();
            digitiser_ids
        };

        // Find Digitiser Event Lists
        let searcher = Searcher::new(
            self.consumer,
            &self.topics.digitiser_event_topic,
            offset,
            Offset::Offset,
        )
        .expect("");

        let (eventlist_results, _) = self
            .search_topic(
                searcher,
                target,
                number,
                SearchStatus::EventListSearchInProgress,
                |msg: &EventListMessage| msg.filter_by_digitiser_id(&digitiser_ids),
            )
            .await;
        self.emit_status(SearchStatus::EventListSearchFinished)
            .await;

        for trace in trace_results.iter() {
            cache.push_trace(&trace.try_unpacked_message().expect("Cannot Unpack Trace"));
        }

        for eventlist in eventlist_results.iter() {
            cache.push_events(
                &eventlist
                    .try_unpacked_message()
                    .expect("Cannot Unpack Eventlist"),
            );
        }
        cache.attach_event_lists_to_trace();

        // Send cache via status
        let time = Utc::now() - start;
        self.emit_status(SearchStatus::Successful {
            num: cache.iter().len(),
            time,
        })
        .await;
        SearchResults { cache }
    }
}

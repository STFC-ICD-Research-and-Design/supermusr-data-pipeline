mod binary_by_timestamp;
//mod by_timestamp;
mod capture;
mod from_end;

use std::marker::PhantomData;

use rdkafka::consumer::StreamConsumer;
use tokio::sync::mpsc;
use tracing::{error, instrument};

use crate::{finder::SearchStatus, Select, Topics};

//pub(crate) use by_timestamp::SearchByTimestamp;
pub(crate) use binary_by_timestamp::BinarySearchByTimestamp;
pub(crate) use from_end::SearchFromEnd;

pub(crate) trait TaskClass {}

pub(crate) struct SearchTask<'a, C: TaskClass> {
    consumer: StreamConsumer,
    send_status: &'a mpsc::Sender<SearchStatus>,
    select: &'a Select,
    topics: &'a Topics,
    phantom: PhantomData<C>,
}

impl<'a, C: TaskClass> SearchTask<'a, C> {
    pub(crate) fn new(
        consumer: StreamConsumer,
        send_status: &'a mpsc::Sender<SearchStatus>,
        select: &'a Select,
        topics: &'a Topics,
    ) -> Self {
        Self {
            consumer,
            send_status,
            select,
            topics,
            phantom: PhantomData,
        }
    }

    #[instrument(skip_all)]
    pub(crate) async fn emit_status(&self, new_status: SearchStatus) {
        if let Err(e) = self.send_status.send(new_status).await {
            error!("{e}");
        }
    }
    /*
    #[instrument(skip_all)]
    async fn search_topic_by_timestamp<M, E, A>(
        &self,
        searcher: Searcher<'a, M>,
        steps: &Steps,
        target: &SearchTarget,
        emit: E,
        acquire_while: A,
    ) -> (Vec<M>, i64)
    where
        E: Fn(u32) -> SearchStatus,
        M: FBMessage<'a>,
        A: Fn(&M) -> bool,
    {
        self.emit_status(emit(0)).await;

        let mut iter = searcher.iter_backstep();
        for step in 0..steps.num_step_passes {
            self.emit_status(emit(step)).await;
            let sz =
                steps.min_step_size * steps.step_mul_coef.pow(steps.num_step_passes - 1 - step);
            iter.step_size(sz)
                .backstep_until_time(|t| t > target.timestamp)
                .await;
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
    pub(crate) async fn search_by_timestamp(
        self,
        target: SearchTarget,
    ) -> (BaseConsumer, SearchResults) {
        let start = Utc::now();

        let mut cache = Cache::default();

        let send_status = self.send_status;

        // Find Digitiser Traces
        let searcher = Searcher::new(
            &self.consumer,
            &self.topics.trace_topic,
            1,
            send_status.clone(),
        );
        let (trace_results, offset) = self
            .search_topic_by_timestamp(
                searcher,
                &self.select.step,
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
            send_status.clone(),
        );
        let (eventlist_results, _) = self
            .search_topic_by_timestamp(
                searcher,
                &self.select.step,
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

    /// Performs a FromEnd search.
    /// # Attributes
    /// - target: what to search for.
    #[instrument(skip_all)]
    pub(crate) async fn search_from_end(
        self,
        target: SearchTarget,
    ) -> (BaseConsumer, SearchResults) {
        let start = Utc::now();

        let mut cache = Cache::default();

        let send_status = self.send_status;

        // Find Digitiser Traces
        self.emit_status(SearchStatus::TraceSearchInProgress(0))
            .await;

        let searcher = Searcher::new(
            &self.consumer,
            &self.topics.trace_topic,
            target.number as i64 + 1,
            send_status.clone(),
        );

        let trace_results: Vec<TraceMessage> = searcher
            .iter_forward()
            .acquire_while(|_| true, target.number)
            .await
            .collect()
            .into();

        // Find Digitiser Event Lists
        self.emit_status(SearchStatus::EventListSearchInProgress(0))
            .await;

        let searcher = Searcher::new(
            &self.consumer,
            &self.topics.digitiser_event_topic,
            2 * target.number as i64 + 1,
            send_status.clone(),
        );

        let eventlist_results: Vec<EventListMessage> = searcher
            .iter_forward()
            .acquire_while(|_| true, 2 * target.number)
            .await
            .collect()
            .into();

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
    } */
}

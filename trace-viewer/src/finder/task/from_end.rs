use chrono::Utc;
use rdkafka::{Offset, consumer::StreamConsumer};
use tracing::instrument;

use crate::{
    finder::{
        SearchResults, SearchStatus,
        searcher::Searcher,
        task::{SearchTask, TaskClass},
    },
    messages::{Cache, EventListMessage, FBMessage, TraceMessage},
};

pub(crate) struct SearchFromEnd;
impl TaskClass for SearchFromEnd {}

impl<'a> SearchTask<'a, SearchFromEnd> {
    /// Performs a FromEnd search.
    /// # Attributes
    /// - target: what to search for.
    #[instrument(skip_all)]
    pub(crate) async fn search(self, number: usize) -> (StreamConsumer, SearchResults) {
        let start = Utc::now();

        let mut cache = Cache::default();

        // Find Digitiser Traces
        self.emit_status(SearchStatus::TraceSearchInProgress(0))
            .await;

        let searcher = Searcher::new(
            &self.consumer,
            &self.topics.trace_topic,
            number as i64 + 1,
            Offset::OffsetTail,
            self.send_status.clone(),
        )
        .expect("");

        let trace_results: Vec<TraceMessage> = searcher
            .iter_forward()
            .acquire_while(|_| true, number)
            .await
            .collect()
            .into();

        // Find Digitiser Event Lists
        self.emit_status(SearchStatus::EventListSearchInProgress(0))
            .await;

        let searcher = Searcher::new(
            &self.consumer,
            &self.topics.digitiser_event_topic,
            2 * number as i64 + 1,
            Offset::OffsetTail,
            self.send_status.clone(),
        )
        .expect("");

        let eventlist_results: Vec<EventListMessage> = searcher
            .iter_forward()
            .acquire_while(|_| true, 2 * number)
            .await
            .collect()
            .into();

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
        self.emit_status(SearchStatus::Successful).await;
        let time = Utc::now() - start;
        (self.consumer, SearchResults { cache, time })
    }
}

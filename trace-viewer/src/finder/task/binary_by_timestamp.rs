use crate::{
    Timestamp,
    finder::{
        task::{SearchTask, TaskClass},
        topic_searcher::{Searcher, SearcherError},
    },
    structs::{
        Cache, EventListMessage, FBMessage, SearchResults, SearchStatus, SearchTargetBy,
        TraceMessage,
    },
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
    ) -> Option<(Vec<M>, i64)>
    where
        E: Fn(f64) -> SearchStatus,
        M: FBMessage<'a>,
        A: Fn(&M) -> bool,
        G: Fn(i64) -> Offset,
    {
        let mut iter = searcher.iter_binary(target);
        iter.init().await;

        if iter.empty() {
            return None;
        }

        loop {
            self.emit_status(emit(iter.get_progress())).await;
            if iter
                .bisect()
                .await
                .expect("bisect works, this should never fail.")
            {
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

        Some((results, offset))
    }

    /// Performs a binary tree search.
    /// # Parameters
    /// - target: what to search for.
    /// - by:
    #[instrument(skip_all)]
    pub(crate) async fn search(
        self,
        target_timestamp: Timestamp,
        search_by: SearchTargetBy,
        number: usize,
    ) -> Result<SearchResults, SearcherError> {
        let start = Utc::now();

        // Find Digitiser Traces
        let searcher = Searcher::new(self.consumer, &self.topics.trace_topic, 1, Offset::Offset)?;

        let trace_results = self
            .search_topic(
                searcher,
                target_timestamp,
                number,
                SearchStatus::TraceSearchInProgress,
                |msg: &TraceMessage| msg.filter_by(&search_by),
            )
            .await;
        self.emit_status(SearchStatus::TraceSearchFinished).await;

        let digitiser_ids = {
            let mut digitiser_ids = trace_results
                .as_ref()
                .map(|(trace_results, _)| {
                    trace_results
                        .iter()
                        .map(TraceMessage::digitiser_id)
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
            digitiser_ids.sort();
            digitiser_ids.dedup();
            digitiser_ids
        };

        let mut cache = Cache::default();

        if let Some((trace_results, offset)) = trace_results {
            // Find Digitiser Event Lists
            let searcher = Searcher::new(
                self.consumer,
                &self.topics.digitiser_event_topic,
                offset,
                Offset::Offset,
            )?;

            let eventlist_results = self
                .search_topic(
                    searcher,
                    target_timestamp,
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

            if let Some((eventlist_results, _)) = eventlist_results {
                for eventlist in eventlist_results.iter() {
                    cache.push_events(
                        &eventlist
                            .try_unpacked_message()
                            .expect("Cannot Unpack Eventlist"),
                    );
                }
            }
        }
        cache.attach_event_lists_to_trace();

        // Send cache via status
        let time = Utc::now() - start;
        self.emit_status(SearchStatus::Successful {
            num: cache.iter().len(),
            time,
        })
        .await;
        Ok(SearchResults::Successful { cache })
    }
}

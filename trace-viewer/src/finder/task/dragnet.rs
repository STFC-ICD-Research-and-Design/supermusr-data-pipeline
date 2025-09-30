use crate::{
    Timestamp,
    finder::{
        task::{SearchTask, TaskClass},
        topic_searcher::{Searcher, SearcherError},
    },
    structs::{Cache, EventListMessage, FBMessage, SearchResults, SearchTargetBy, TraceMessage},
};
use rdkafka::consumer::StreamConsumer;
use tracing::instrument;
pub(crate) struct Dragnet;
impl TaskClass for Dragnet {}

impl<'a> SearchTask<'a, Dragnet> {
    /// Performs a binary tree search on a given topic, with generic filtering functions.
    #[instrument(skip_all)]
    async fn search_topic<M, A>(
        &self,
        searcher: Searcher<'a, M, StreamConsumer>,
        target: Timestamp,
        backstep: i64,
        forward_distance: usize,
        number: usize,
        acquire_matches: A,
    ) -> Option<(Vec<M>, i64)>
    where
        M: FBMessage<'a>,
        A: Fn(&M) -> bool,
    {
        let mut iter = searcher.iter_binary(target);
        iter.init().await;

        if iter.empty() {
            return None;
        }

        loop {
            if iter
                .bisect()
                .await
                .expect("bisect works, this should never fail.")
            {
                break;
            }
        }

        let searcher = iter.collect();
        let offset = searcher.get_offset();

        let mut iter = searcher.iter_dragnet();
        iter.backstep_by(backstep)
            .acquire_matches(forward_distance, number, acquire_matches)
            .await;
        let searcher = iter.collect();
        let results: Vec<M> = searcher.into();

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
        backstep: i64,
        forward_distance: usize,
        search_by: SearchTargetBy,
        number: usize,
    ) -> Result<SearchResults, SearcherError> {
        // Find Digitiser Traces
        let searcher = Searcher::new(self.consumer, &self.topics.trace_topic, 1)?;

        let trace_results = self
            .search_topic(
                searcher,
                target_timestamp,
                backstep,
                forward_distance,
                number,
                |msg: &TraceMessage| msg.filter_by(&search_by),
            )
            .await;

        //Self::get_digitiser_ids_from_traces(trace_results.map(|(traces,_)|&traces));
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
            let searcher =
                Searcher::new(self.consumer, &self.topics.digitiser_event_topic, offset)?;

            let eventlist_results = self
                .search_topic(
                    searcher,
                    target_timestamp,
                    backstep,
                    forward_distance,
                    number,
                    |msg: &EventListMessage| msg.filter_by_digitiser_id(&digitiser_ids),
                )
                .await;

            for trace in trace_results.iter() {
                cache.push_trace(
                    &trace
                        .try_unpacked_message()
                        .expect("Cannot Unpack Trace. TODO should be handled"),
                )?;
            }

            if let Some((eventlist_results, _)) = eventlist_results {
                for eventlist in eventlist_results.iter() {
                    cache.push_events(
                        &eventlist
                            .try_unpacked_message()
                            .expect("Cannot Unpack Eventlist. TODO should be handled"),
                    )?;
                }
            }
        }
        cache.attach_event_lists_to_trace();

        Ok(SearchResults::Successful { cache })
    }
}

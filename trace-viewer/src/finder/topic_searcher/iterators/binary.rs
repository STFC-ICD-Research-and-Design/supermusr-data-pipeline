use crate::{
    Timestamp,
    finder::topic_searcher::{Searcher, searcher::SearcherError},
    structs::FBMessage,
};
use rdkafka::consumer::StreamConsumer;
use std::ops::Range;
use tracing::{debug, warn};

/// Searches on a topic forwards, one message at a time.
///
/// Note this iterator can both move the [Searcher]'s offset and accumulate results.
/// Also note, this iterator is not a real iterator (as in it does not implement [Iterator]).
/// Instead it's methods are inspired by those frequently found in actual iterators.
pub(crate) struct BinarySearchIter<'a, M, C> {
    pub(crate) inner: Searcher<'a, M, C>,
    pub(crate) bound: Range<i64>,
    pub(crate) max_bound: Range<i64>,
    pub(crate) target: Timestamp,
}

impl<'a, M> BinarySearchIter<'a, M, StreamConsumer> {
    pub(crate) fn collect(mut self) -> Searcher<'a, M, StreamConsumer> {
        self.inner.set_offset(self.bound.start);
        self.inner
    }
}

impl<'a, M> BinarySearchIter<'a, M, StreamConsumer>
where
    M: FBMessage<'a>,
{
    #[tracing::instrument(skip_all, level = "debug", fields(start, end, size))]
    pub(crate) async fn init(&mut self) {
        // TODO: Should implement some sort of buffer to avoid the left bound being pushed out of range.
        let bounds = self.inner.get_current_bounds();

        self.max_bound = bounds.0..bounds.1;
        self.bound = self.max_bound.clone();

        tracing::Span::current()
            .record("start", self.bound.start)
            .record("end", self.bound.end)
            .record("size", self.bound.end - self.bound.start);

        debug!("New Binary Search Iterator");
    }

    pub(crate) fn empty(&self) -> bool {
        self.max_bound.is_empty()
    }

    ///
    /// # Invariant
    /// Given the assumption that all topic messages are weakly-ascending in chronological order,
    /// this method preserves the invariant that: the timestamps, `S` and `T`, of the messages
    /// at respective offsets `self.bound.start` and `self.bound.end`,
    /// satisfy `S <= self.target < T`.
    /// # Return
    /// - Ok(true) if `self.bound` has length at most `1``.
    /// - Ok(false) otherwise
    #[tracing::instrument(skip_all, level = "debug", fields(start = self.bound.start, end = self.bound.end, size = self.bound.end-self.bound.start))]
    pub(crate) async fn bisect(&mut self) -> Result<bool, SearcherError> {
        if self.bound.end - self.bound.start > 1 {
            let mid = (self.bound.end + self.bound.start) / 2;

            match self.inner.message(mid).await {
                Ok(msg) => {
                    if msg.timestamp() <= self.target {
                        self.bound.start = mid;
                        debug!("Bisecting upwards.");
                    } else if msg.timestamp() > self.target {
                        self.bound.end = mid;
                        debug!("Bisecting downwards.");
                    }
                }
                Err(e) => {
                    warn!("{e}");
                    // Should we do something else here?
                    self.bound.start += 2;
                }
            }
            // If we have reached the start or end.
            if mid == self.max_bound.start {
                Err(SearcherError::StartOfTopicReached)
            } else if mid == self.max_bound.end {
                Err(SearcherError::EndOfTopicReached)
            } else {
                Ok(false)
            }
        } else {
            debug!("Bisecting complete.");
            Ok(true)
        }
    }
}

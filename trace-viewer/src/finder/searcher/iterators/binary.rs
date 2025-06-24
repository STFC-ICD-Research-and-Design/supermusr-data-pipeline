use std::{ops::Range, time::Duration};

use rdkafka::{consumer::{Consumer, StreamConsumer}, util::Timeout, Offset};
use tracing::info;

use crate::{
    Timestamp,
    finder::{
        SearchStatus,
        searcher::{Searcher, searcher_structure::SearcherError},
    },
    messages::FBMessage,
};

/// Searches on a topic forwards, one message at a time.
///
/// Note this iterator can both move the [Searcher]'s offset and accumulate results.
/// Also note, this iterator is not a real iterator (as in it does not implement [Iterator]).
/// Instead it's methods are inspired by those frequently found in actual iterators.
pub(crate) struct BinarySearchIter<'a, M, C, G> {
    pub(crate) inner: Searcher<'a, M, C, G>,
    pub(crate) bound: Range<i64>,
    pub(crate) max_bound: Range<i64>,
    pub(crate) target: Timestamp,
}

impl<'a, M, G> BinarySearchIter<'a, M, StreamConsumer, G> {
    pub(crate) fn collect(mut self) -> Searcher<'a, M, StreamConsumer, G> {
        self.inner.set_offset(self.bound.start);
        self.inner
    }
}

impl<'a, M, G: Fn(i64) -> Offset> BinarySearchIter<'a, M, StreamConsumer, G>
where
    M: FBMessage<'a>,
{
    pub(crate) async fn init(&mut self) {
        
        // TODO: Should implement some sort of buffer to avoid the left bound being pushed out of range.
        let bounds = self.inner
            .consumer
            .fetch_watermarks(&self.inner.topic, 0, Timeout::After(Duration::from_millis(2000)))
            .expect("Should get watermarks, this should not fail.");

        self.max_bound = bounds.0..bounds.1;
        self.bound = self.max_bound.clone();

        info!(
            "Bisecting Binary Tree: {} <= {}",
            self.bound.start, self.bound.end
        );
        self.inner
            .send_status
            .send(SearchStatus::Text(format!(
                "Bisecting Binary Tree: {} <= {}",
                self.bound.start, self.bound.end
            )))
            .await
            .expect("");
    }

    pub(crate) async fn bisect(&mut self) -> Result<bool, SearcherError> {
        if self.bound.end - self.bound.start > 1 {
            let mid = (self.bound.end + self.bound.start) / 2;

            info!(
                "Bisecting Binary Tree: {} <= {mid} <= {}: size: {}",
                self.bound.start,
                self.bound.end,
                self.bound.end - self.bound.start
            );
            self.inner
                .send_status
                .send(SearchStatus::Text(format!(
                    "Bisecting Binary Tree: {} <= {mid} <= {}: size: {}",
                    self.bound.start,
                    self.bound.end,
                    self.bound.end - self.bound.start
                )))
                .await
                .expect("");

            let msg = self
                .inner
                .message_from_raw_offset(Offset::Offset(mid))
                .await?;
            if msg.timestamp() <= self.target {
                self.bound.start = mid;
            } else if msg.timestamp() > self.target {
                self.bound.end = mid;
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
            info!("Found match {}", self.bound.start);
            Ok(true)
        }
    }
}

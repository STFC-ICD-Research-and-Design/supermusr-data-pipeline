use crate::{
    Timestamp,
    finder::topic_searcher::{Searcher, searcher::SearcherError},
    structs::FBMessage,
};
use rdkafka::consumer::StreamConsumer;
use tracing::{instrument, warn};

/// Performs a dragnet search on the broker from the searcher's offset.
///
/// Note this iterator can only move the [Searcher]'s offset, it cannot accumulate results.
/// Also note, this iterator is not a real iterator (as in it does not implement [Iterator]).
/// Instead it's methods are inspired by those frequently found in actual iterators.
pub(crate) struct DragNetIter<'a, M, C> {
    pub(crate) inner: Searcher<'a, M, C>,
}

impl<'a, M, C> DragNetIter<'a, M, C> {
    /// Consumes the iterator and returns the original [Searcher] object.
    pub(crate) fn collect(self) -> Searcher<'a, M, C> {
        self.inner
    }
}

impl<'a, M> DragNetIter<'a, M, StreamConsumer>
where
    M: FBMessage<'a>,
{
    /// Moves the topic's offset back, clamping at the minimum offset.
    ///
    /// # Parameters
    /// - backstep: amount to backstep by.
    #[instrument(skip_all)]
    pub(crate) fn backstep_by(&mut self, backstep: i64) -> &mut Self {
        self.inner
            .set_offset((self.inner.offset - backstep).min(self.inner.get_current_bounds().0));
        self
    }

    /// Steps forward, message by message, ignoring timestamp order, acquiring messages which satisfy the predicate,
    /// until the given number of messages have been tested.
    ///
    /// # Parameters
    /// - f: a predicte taking a timestamp, it should return true if a message satisfies the matching criteria.
    #[instrument(skip_all)]
    pub(crate) async fn acquire_matches<F: Fn(&M) -> bool>(
        &mut self,
        message_num: usize,
        max_timestamps: usize,
        f: F,
    ) -> &mut Self {
        let mut timestamps = Vec::<Timestamp>::with_capacity(max_timestamps);
        for _ in 0..message_num {
            if let Some(msg) = self
                .inner
                .recv()
                .await
                .map(TryFrom::try_from)
                .and_then(Result::ok)
            {
                if f(&msg) {
                    if timestamps.contains(&msg.timestamp()) {
                        debug!("Message with existing timestamp found");
                        self.inner.results.push(msg);
                    } else if timestamps.len() < max_timestamps {
                        debug!("Message with new timestamp found");
                        timestamps.push(msg.timestamp());
                        self.inner.results.push(msg);
                    }
                }
            }
        }
        self
    }
}

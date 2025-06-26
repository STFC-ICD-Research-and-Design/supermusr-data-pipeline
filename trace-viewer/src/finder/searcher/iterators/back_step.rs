use crate::{
    Timestamp,
    finder::searcher::{Searcher, searcher_structure::SearcherError},
    messages::FBMessage,
};
use rdkafka::{Offset, consumer::StreamConsumer};
use tracing::{error, instrument};

/// Performs a backwards search on the broker from the searcher's offset.
///
/// Note this iterator can only move the [Searcher]'s offset, it cannot accumulate results.
/// Also note, this iterator is not a real iterator (as in it does not implement [Iterator]).
/// Instead it's methods are inspired by those frequently found in actual iterators.
pub(crate) struct BackstepIter<'a, M, C, F> {
    pub(crate) inner: Searcher<'a, M, C, F>,
    pub(crate) step_size: Option<i64>,
}

impl<'a, M, C, F> BackstepIter<'a, M, C, F> {
    /// Sets the size the backstep.
    pub(crate) fn step_size(&mut self, step_size: i64) -> &mut Self {
        self.step_size = Some(step_size);
        self
    }

    /// Consumes the iterator and returns the original [Searcher] object.
    pub(crate) fn collect(self) -> Searcher<'a, M, C, F> {
        self.inner
    }
}

impl<'a, M, G: Fn(i64) -> Offset> BackstepIter<'a, M, StreamConsumer, G>
where
    M: FBMessage<'a>,
{
    /// Repeatedly search the topic backwards, in increments of [Self::step_size],
    /// until the given predicate of the message's timestamp is satisfied.
    ///
    /// # Attributes
    /// - f: a predicte taking a timestamp, it should return true when the timestamp is later than the target.
    #[instrument(skip_all)]
    pub(crate) async fn backstep_until_time<F: Fn(Timestamp) -> bool>(
        &mut self,
        f: F,
    ) -> Result<&mut Self, SearcherError> {
        let mut offset = self.inner.offset;
        let mut earliest = self.inner.message(offset).await?.timestamp();

        while f(earliest) {
            let new_offset = offset
                - self
                    .step_size
                    .expect("Size step should have been set. This should never fail.");
            match self.inner.message(new_offset).await {
                Ok(message) => {
                    let new_timestamp = message.timestamp();
                    if f(new_timestamp) {
                        offset = new_offset;
                        earliest = new_timestamp;
                    } else {
                        break;
                    }
                }
                Err(e) => {
                    error!("{e}");
                    break;
                }
            }
        }

        self.inner.set_offset(offset);
        Ok(self)
    }
}

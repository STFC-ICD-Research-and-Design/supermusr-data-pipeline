use crate::{Timestamp, finder::topic_searcher::Searcher, structs::FBMessage};
use rdkafka::consumer::StreamConsumer;
use tracing::instrument;

/// Searches on a topic forwards, one message at a time.
///
/// Note this iterator can both move the [Searcher]'s offset and accumulate results.
/// Also note, this iterator is not a real iterator (as in it does not implement [Iterator]).
/// Instead it's methods are inspired by those frequently found in actual iterators.
pub(crate) struct ForwardSearchIter<'a, M, C> {
    pub(crate) inner: Searcher<'a, M, C>,
    pub(crate) message: Option<M>,
}

impl<'a, M, C> ForwardSearchIter<'a, M, C> {
    /// Consumes the iterator and returns the original [Searcher] object.
    pub(crate) fn collect(self) -> Searcher<'a, M, C> {
        self.inner
    }
}

impl<'a, M> ForwardSearchIter<'a, M, StreamConsumer>
where
    M: FBMessage<'a>,
{
    /// Steps forward, message by message, until the given predicate fails.
    ///
    /// # Parameters
    /// - f: a predicte taking a timestamp, it should return true when the timestamp is earlier than the target.
    #[instrument(skip_all)]
    pub(crate) async fn move_until<F: Fn(Timestamp) -> bool>(mut self, f: F) -> Self {
        while let Some(msg) = self.inner.recv().await {
            if let Some(msg) = M::try_from(msg).ok().filter(|m| f(FBMessage::timestamp(m))) {
                self.message = Some(msg);
                break;
            }
        }
        self
    }

    /// Steps forward, message by message, acquiring messages which satisfy the predicate, until the given number of messages are obtained. [TODO]
    ///
    /// # Parameters
    /// - f: a predicte taking a timestamp, it should return true when the timestamp is earlier than the target.
    #[instrument(skip_all)]
    pub(crate) async fn acquire_while<F: Fn(&M) -> bool>(mut self, f: F, number: usize) -> Self {
        if let Some(first_message) = self.message.take() {
            let mut timestamp = first_message.timestamp();
            if f(&first_message) {
                self.inner.results.push(first_message);
            }

            let mut messages: Option<M> = self
                .inner
                .recv()
                .await
                .map(TryFrom::try_from)
                .and_then(Result::ok);

            for _ in 0..number {
                while let Some(msg) = messages {
                    messages = self
                        .inner
                        .recv()
                        .await
                        .map(TryFrom::try_from)
                        .and_then(Result::ok);

                    let new_timestamp = msg.timestamp();
                    if new_timestamp == timestamp {
                        if f(&msg) {
                            self.inner.results.push(msg);
                        }
                    } else {
                        timestamp = new_timestamp;
                        break;
                    }
                }
            }
        }
        self
    }
}

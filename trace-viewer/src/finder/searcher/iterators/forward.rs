use rdkafka::consumer::StreamConsumer;
use tracing::instrument;

use crate::{
    finder::{searcher::Searcher, SearchStatus},
    messages::FBMessage,
    Timestamp,
};

/// Searches on a topic forwards, one message at a time.
///
/// Note this iterator can both move the [Searcher]'s offset and accumulate results.
/// Also note, this iterator is not a real iterator (as in it does not implement [Iterator]).
/// Instead it's methods are inspired by those frequently found in actual iterators.
pub(crate) struct ForwardSearchIter<'a, M, C, G> {
    pub(crate) inner: Searcher<'a, M, C, G>,
    pub(crate) message: Option<M>,
}

impl<'a, M, C, G> ForwardSearchIter<'a, M, C, G> {
    /// Consumes the iterator and returns the original [Searcher] object.
    pub(crate) fn collect(self) -> Searcher<'a, M, C, G> {
        self.inner
    }
}

impl<'a, M, G> ForwardSearchIter<'a, M, StreamConsumer, G>
where
    M: FBMessage<'a>,
{
    /// Steps forward, message by message, until the given predicate fails.
    ///
    /// # Attributes
    /// - f: a predicte taking a timestamp, it should return true when the timestamp is earlier than the target.
    #[instrument(skip_all)]
    pub(crate) async fn move_until<F: Fn(Timestamp) -> bool>(mut self, f: F) -> Self {
        while let Ok(msg) = self.inner.consumer.recv().await {
            if let Some(msg) =
                FBMessage::from_borrowed_message(msg).filter(|m| f(FBMessage::timestamp(m)))
            {
                self.message = Some(msg);
                self.inner
                    .send_status
                    .send(SearchStatus::Text(format!(
                        "Message timestamp: {0}",
                        self.message.as_ref().expect("").timestamp()
                    )))
                    .await
                    .expect("");
                break;
            }
        }
        self
    }

    /// Steps forward, message by message, acquiring messages which satisfy the predicate, until the given number of messages are obtained. [TODO]
    ///
    /// # Attributes
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
                .consumer
                .recv()
                .await
                .ok()
                .and_then(FBMessage::from_borrowed_message);

            for _ in 0..number {
                while let Some(msg) = messages {
                    messages = self
                        .inner
                        .consumer
                        .recv()
                        .await
                        .ok()
                        .and_then(FBMessage::from_borrowed_message);

                    self.inner
                        .send_status
                        .send(SearchStatus::Text(format!(
                            "Message timestamp: {0}",
                            msg.timestamp()
                        )))
                        .await
                        .expect("");
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

use crate::{
    Timestamp,
    finder::topic_searcher::{Searcher, iterators::NonChronologicalMessageDetector},
    structs::FBMessage,
};
use rdkafka::consumer::StreamConsumer;
use tracing::{debug, instrument, warn};

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
        let mut last_timestamp = NonChronologicalMessageDetector::default();
        while let Some(msg) = self.inner.recv().await {
            debug!("Advancing.");
            if let Some(msg) = M::try_from(msg)
                .inspect_err(|e| warn!("{e}"))
                .ok()
                .inspect(|m| last_timestamp.next(m.timestamp()))
                .filter(|m| f(FBMessage::timestamp(m)))
            {
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
                debug!("Initial Message is Match.");
                self.inner.results.push(first_message);
            }
            let mut last_timestamp = NonChronologicalMessageDetector::new(timestamp);

            let mut messages: Option<M> = self
                .inner
                .recv()
                .await
                .and_then(|m| M::try_from(m).inspect_err(|e| warn!("{e}")).ok());

            for _ in 0..number {
                debug!("Matching {timestamp}.");
                while let Some(msg) = messages {
                    last_timestamp.next(msg.timestamp());
                    messages = self
                        .inner
                        .recv()
                        .await
                        .and_then(|m| M::try_from(m).inspect_err(|e| warn!("{e}")).ok());
                    debug!("Advance to Next Broker Message.");

                    let new_timestamp = msg.timestamp();
                    if new_timestamp == timestamp {
                        debug!("Found Matching Timestamp Message.");
                        if f(&msg) {
                            debug!("Found Match.");
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

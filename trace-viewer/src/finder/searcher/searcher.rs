use std::time::Duration;

use rdkafka::{
    Offset, TopicPartitionList,
    consumer::{Consumer, StreamConsumer},
    error::KafkaError,
};
use thiserror::Error;
use tokio::sync::mpsc;
use tracing::{info, instrument};

use crate::{
    Timestamp,
    finder::{
        SearchStatus,
        searcher::{BackstepIter, BinarySearchIter, ForwardSearchIter},
    },
    messages::FBMessage,
};

#[derive(Error, Debug)]
pub(crate) enum SearcherError {
    #[error("Topic start reached")]
    StartOfTopicReached,
    #[error("Topic end reached")]
    EndOfTopicReached,
    #[error("No valid message found")]
    NoMessageFound,
    #[error("Kafka Error {0}")]
    Kafka(#[from] KafkaError),
}

/// Object to search through the broker from a given offset, on a given topic, for messages of type `M`.
pub(crate) struct Searcher<'a, M, C, G> {
    /// Reference to the Kafka consumer.
    pub(super) consumer: &'a C,
    /// Topic to search on.
    pub(super) topic: String,
    /// Current offset.
    pub(super) offset: i64,
    /// Offset function.
    pub(super) offset_fn: G,
    /// Send channel, along which status messages should be sent.
    pub(super) send_status: mpsc::Sender<SearchStatus>,
    /// Results accumulate here.
    pub(super) results: Vec<M>,
}

impl<'a, M, C: Consumer, G> Searcher<'a, M, C, G> {
    /// Creates a new instance, and assigns the given topic to the broker's consumer.
    ///
    /// # Attributes
    /// - consumer: the broker's consumer to use.
    /// - topic: the topic to search on.
    /// - offset: the offset to search from.
    /// - send_status: send channel, along which status messages should be sent.
    #[instrument(skip_all)]
    pub(crate) fn new(
        consumer: &'a C,
        topic: &str,
        offset: i64,
        offset_fn: G,
        send_status: mpsc::Sender<SearchStatus>,
    ) -> Result<Self, SearcherError> {
        let mut tpl = TopicPartitionList::with_capacity(1);
        tpl.add_partition_offset(topic, 0, rdkafka::Offset::End).expect("Cannot set offset to end.");
        consumer.assign(&tpl).expect("Cannot assign to consumer.");
        Ok(Self {
            consumer,
            offset,
            offset_fn,
            topic: topic.to_owned(),
            send_status,
            results: Default::default(),
        })
    }

    #[instrument(skip_all)]
    pub(crate) async fn emit_status(
        send_status: &mpsc::Sender<SearchStatus>,
        new_status: SearchStatus,
    ) -> Result<(), SearcherError> {
        send_status.send(new_status).await.expect("Cannot send status");
        Ok(())
    }

    #[instrument(skip_all)]
    /// Consumer the searcher and create a backstep iterator.
    pub(crate) fn iter_backstep(self) -> BackstepIter<'a, M, C, G> {
        BackstepIter {
            inner: self,
            step_size: None,
        }
    }

    #[instrument(skip_all)]
    /// Consumer the searcher and create a forward iterator.
    pub(crate) fn iter_forward(self) -> ForwardSearchIter<'a, M, C, G> {
        ForwardSearchIter {
            inner: self,
            message: None,
        }
    }

    #[instrument(skip_all)]
    /// Consumer the searcher and create a forward iterator.
    pub(crate) fn iter_binary(self, target: Timestamp) -> BinarySearchIter<'a, M, C, G> {
        BinarySearchIter {
            inner: self,
            bound: Default::default(),
            max_bound: Default::default(),
            target,
        }
    }

    /// Sets the offset.
    pub(super) fn set_offset(&mut self, offset: i64) {
        self.offset = offset;
    }

    /// Gets the offset.
    pub(crate) fn get_offset(&self) -> i64 {
        self.offset
    }
}

/// Extracts the results from the searcher, when the user is finished with it.
impl<'a, M, C, F> Into<Vec<M>> for Searcher<'a, M, C, F> {
    #[instrument(skip_all)]
    fn into(self) -> Vec<M> {
        self.results
    }
}

impl<'a, M, F: Fn(i64) -> Offset> Searcher<'a, M, StreamConsumer, F>
where
    M: FBMessage<'a>,
{
    #[instrument(skip_all, fields(offset=offset))]
    pub(super) async fn message(&mut self, offset: i64) -> Result<M, SearcherError> {
        self.message_from_raw_offset((self.offset_fn)(offset)).await
    }

    #[instrument(skip_all)]
    pub(super) async fn message_from_raw_offset(
        &mut self,
        offset: Offset,
    ) -> Result<M, SearcherError> {
        self.consumer
            .seek(&self.topic, 0, offset, Duration::from_millis(1))
            .expect("Consumer cannot seek to offset");

        let msg: M = FBMessage::from_borrowed_message(self.consumer.recv().await?)
            .ok_or(SearcherError::NoMessageFound)?;

        info!(
            "Message at offset {offset:?}: timestamp: {0}",
            msg.timestamp()
        );

        self.send_status
            .send(SearchStatus::Text(format!(
                "Message at offset {offset:?}: timestamp: {0}",
                msg.timestamp()
            )))
            .await
            .expect("");

        Ok(msg)
    }
}

use crate::{
    finder::{
        status_sharer::StatusSharer,
        task::{BinarySearchByTimestamp, SearchTask},
        topic_searcher::{Searcher, SearcherError},
    },
    structs::{
        BrokerInfo, BrokerTopicInfo, EventListMessage, FBMessage, SearchResults, SearchTarget,
        SearchTargetMode, Topics, TraceMessage,
    },
};
use chrono::Utc;
use rdkafka::{
    Offset,
    consumer::{Consumer, StreamConsumer},
    error::KafkaError,
    util::Timeout,
};
use std::time::Duration;
use thiserror::Error;
use tracing::{debug, instrument};

#[derive(Error, Debug)]
pub(crate) enum SearchEngineError {
    #[error("Searcher Error: {0}")]
    Searcher(#[from] SearcherError),
    #[error("Kafka Error {0}")]
    Kafka(#[from] KafkaError),
}

pub struct SearchEngine {
    /// The Kafka consumer object, the engine uses to poll for messages.
    ///
    /// The object takes temporary ownership of the consumer object,
    /// if another instance of SearchEngine wants to use it,
    /// it must be passed to it.
    consumer: StreamConsumer,
    topics: Topics,
    status_send: StatusSharer
}

impl SearchEngine {
    pub fn new(consumer: StreamConsumer, topics: &Topics, status_send: StatusSharer) -> Self {
        Self {
            consumer,
            topics: topics.clone(),
            status_send
        }
    }

    async fn poll_broker_topic_info<'a, M: FBMessage<'a>>(
        consumer: &'a StreamConsumer,
        topic: &str,
        poll_broker_timeout_ms: u64,
    ) -> Result<BrokerTopicInfo, SearchEngineError> {
        let offsets = consumer.fetch_watermarks(
            topic,
            0,
            Timeout::After(Duration::from_millis(poll_broker_timeout_ms)),
        )?;
        debug!("Topic {topic}: (High, Low) offsets: {offsets:?}");

        if offsets.0 == offsets.1 {
            Ok(BrokerTopicInfo {
                offsets,
                timestamps: None,
            })
        } else {
            let mut searcher =
                Searcher::<M, StreamConsumer, _>::new(consumer, topic, offsets.0, Offset::Offset)?;
            let begin = searcher.message(offsets.0).await?;
            let end = searcher.message(offsets.1 - 1).await?;

            Ok(BrokerTopicInfo {
                offsets,
                timestamps: Some((begin.timestamp(), end.timestamp())),
            })
        }
    }

    #[instrument(skip_all)]
    pub(crate) async fn poll_broker(
        &self,
        poll_broker_timeout_ms: u64,
    ) -> Result<BrokerInfo, SearchEngineError> {
        let trace = Self::poll_broker_topic_info::<TraceMessage>(
            &self.consumer,
            &self.topics.trace_topic,
            poll_broker_timeout_ms,
        )
        .await?;
        let events = Self::poll_broker_topic_info::<EventListMessage>(
            &self.consumer,
            &self.topics.digitiser_event_topic,
            poll_broker_timeout_ms,
        )
        .await?;

        Ok(BrokerInfo {
            timestamp: Utc::now(),
            trace,
            events,
        })
    }

    #[instrument(skip_all)]
    pub(crate) async fn search(
        &mut self,
        target: SearchTarget,
    ) -> Result<SearchResults, SearchEngineError> {
        Ok(match target.mode {
            SearchTargetMode::Timestamp { timestamp } => {
                SearchTask::<BinarySearchByTimestamp>::new(
                    &self.consumer,
                    &self.topics,
                    self.status_send.clone(),
                )
                .search(timestamp, target.by, target.number)
                .await?
            }
        })
    }
}

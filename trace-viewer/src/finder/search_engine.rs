use crate::{
    finder::{
        MessageFinder,
        searcher::Searcher,
        task::{BinarySearchByTimestamp, SearchTask},
    },
    messages::{EventListMessage, FBMessage, TraceMessage},
    structs::{
        BrokerInfo, BrokerTopicInfo, SearchMode, SearchResults, SearchStatus, SearchTarget,
        SearchTargetMode, Topics,
    },
};
use chrono::Utc;
use rdkafka::{
    Offset,
    consumer::{Consumer, StreamConsumer},
    util::Timeout,
};
use std::sync::{Mutex, Arc};
use std::time::Duration;
//use tokio::{select, sync::mpsc, task::JoinHandle};
use tracing::{debug, instrument};

pub struct SearchEngine {
    /// The Kafka consumer object, the engine uses to poll for messages.
    ///
    /// The object takes temporary ownership of the consumer object,
    /// if another instance of SearchEngine wants to use it,
    /// it must be passed to it.
    consumer: StreamConsumer,
    //status: SearchStatus,
    topics: Topics,
    status: Arc<Mutex<SearchStatus>>,
}

impl SearchEngine {
    pub fn new(consumer: StreamConsumer, topics: &Topics, status: Arc<Mutex<SearchStatus>>) -> Self {
        let topics = topics.clone();

        /*let (send_init, mut recv_init) = mpsc::channel(1);
        let (send_results, recv_results) = mpsc::channel(1);
        let (send_status, recv_status) = mpsc::channel(1);
        let (send_poll_broker, mut recv_poll_broker) = mpsc::channel(1);
        let (send_broker_info, recv_broker_info) = mpsc::channel(1);*/
        Self {
            consumer: consumer,
            topics: topics.clone(),
            status,
        }
    }

    async fn poll_broker_topic_info<'a, M: FBMessage<'a>>(
        consumer: &'a StreamConsumer,
        topic: &str,
        poll_broker_timeout_ms: u64,
    ) -> Option<BrokerTopicInfo> {
        let offsets = consumer
            .fetch_watermarks(
                topic,
                0,
                Timeout::After(Duration::from_millis(poll_broker_timeout_ms)),
            )
            .ok()?;
        debug!("(High, Low) offsets: {offsets:?}");

        if offsets.0 == offsets.1 {
            Some(BrokerTopicInfo {
                offsets,
                timestamps: None,
            })
        } else {
            let mut searcher =
                Searcher::<M, StreamConsumer, _>::new(consumer, topic, offsets.0, Offset::Offset)
                    .ok()?;
            let begin = searcher.message(offsets.0).await.ok()?;
            let end = searcher.message(offsets.1 - 1).await.ok()?;

            Some(BrokerTopicInfo {
                offsets,
                timestamps: Some((begin.timestamp(), end.timestamp())),
            })
        }
    }
}

impl MessageFinder for SearchEngine {
    type SearchMode = SearchMode;

    #[instrument(skip_all)]
    async fn search(&mut self, target: SearchTarget) -> SearchResults {
        match target.mode {
            SearchTargetMode::Timestamp { timestamp } => {
                SearchTask::<BinarySearchByTimestamp>::new(&self.consumer, &self.topics, self.status.clone())
                    .search(timestamp, target.by, target.number)
                    .await
            }
        }
    }

    #[instrument(skip_all)]
    async fn poll_broker(&self, poll_broker_timeout_ms: u64) -> Option<BrokerInfo> {
        let trace = Self::poll_broker_topic_info::<TraceMessage>(
            &self.consumer,
            &self.topics.trace_topic,
            poll_broker_timeout_ms,
        )
        .await;
        let events = Self::poll_broker_topic_info::<EventListMessage>(
            &self.consumer,
            &self.topics.digitiser_event_topic,
            poll_broker_timeout_ms,
        )
        .await;

        Option::zip(trace, events).map(|(trace, events)| BrokerInfo {
            timestamp: Utc::now(),
            trace,
            events,
        })
    }
}
/*
    #[instrument(skip_all)]
    fn init_search(&mut self, target: SearchTarget) -> bool {
        if self.consumer.is_some() {
            self.target = Some(target);
        }
        self.consumer.is_some()
    }
    #[instrument(skip_all)]
    fn init_poll_broker_info(&mut self) -> bool {
        if self.consumer.is_some() {
            self.poll_broker = Some(());
        }
        self.consumer.is_some()
    }

    fn status(&mut self) -> Option<SearchStatus> {
        self.status.take()
    }

    fn results(&mut self) -> Option<SearchResults> {
        self.results.take()
    }

    async fn async_update(&mut self) {
        // Initiate Search
        if let Some(target) = self.target.take() {
            if let Some(consumer) = self.consumer.take() {
                if let Err(e) = self.send_init.send((consumer, target)).await {
                    error!("send_init failed {e}");
                }
            } else {
                error!("Missing Consumer");
            }
        }

        // Search Terminated
        if !self.recv_results.is_empty() {
            if let Some((consumer, results)) = self.recv_results.recv().await {
                self.consumer = Some(consumer);
                self.results = Some(results);
            }
        }

        // Initiate Broker Poll
        if self.poll_broker.take().is_some() {
            if let Some(consumer) = self.consumer.take() {
                if let Err(e) = self.send_poll_broker.send(consumer).await {
                    error!("send_poll_broker failed {e}");
                }
            }
        }

        // Broker Poll Terminated
        if !self.recv_broker_info.is_empty() {
            if let Some((consumer, broker_info)) = self.recv_broker_info.recv().await {
                self.consumer = Some(consumer);
                self.broker_info = Some(broker_info);
            }
        }

        // Status Received
        if !self.recv_status.is_empty() {
            if let Some(status) = self.recv_status.recv().await {
                self.status = Some(status);
            }
        }
    }

    fn broker_info(&mut self) -> Option<Option<BrokerInfo>> {
        self.broker_info.take()
    }
}
 */

use std::time::Duration;

use rdkafka::{consumer::{Consumer, StreamConsumer}, util::Timeout, Offset};
use tokio::{select, sync::mpsc, task::JoinHandle};
use tracing::{error, instrument};

use crate::{
    finder::{
        searcher::Searcher, task::{BinarySearchByTimestamp, SearchTask}, BrokerInfo, BrokerTopicInfo, MessageFinder, SearchMode, SearchResults, SearchStatus, SearchTarget, SearchTargetMode
    }, messages::{EventListMessage, FBMessage, TraceMessage}, Topics
};

pub(crate) struct SearchEngine {
    /// The Kafka consumer object, the engine uses to poll for messages.
    ///
    /// The object takes temporary ownership of the consumer object,
    /// if another instance of SearchEngine wants to use it,
    /// it must be passed to it.
    consumer: Option<StreamConsumer>,
    /// The search target.
    target: Option<SearchTarget>,
    /// When another instance of [Self] is finished with the [StreamConsumer] object,
    /// it is passed back via this channel.
    send_init: mpsc::Sender<(StreamConsumer, SearchTarget)>,
    recv_results: mpsc::Receiver<(StreamConsumer, SearchResults)>,
    /// If the results are available they are temporarily stored here.
    ///
    /// They are accessed by an external module calling [MessageFinder::results], which takes ownership of the results.
    results: Option<SearchResults>,

    ///
    poll_broker: Option<()>,
    /// 
    send_poll_broker: mpsc::Sender<StreamConsumer>,
    /// 
    recv_broker_info: mpsc::Receiver<(StreamConsumer, Option<BrokerInfo>)>,
    /// Information relating to the number of messages available on the broker.
    broker_info: Option<Option<BrokerInfo>>,
    
    status: Option<SearchStatus>,
    recv_status: mpsc::Receiver<SearchStatus>,

    /// When a search is in progress
    handle: JoinHandle<()>,
}

impl SearchEngine {
    pub(crate) fn new(consumer: StreamConsumer, topics: &Topics) -> Self {
        let topics = topics.clone();

        let (send_init, mut recv_init) = mpsc::channel(1);
        let (send_results, recv_results) = mpsc::channel(1);
        let (send_status, recv_status) = mpsc::channel(1);
        let (send_poll_broker, mut recv_poll_broker) = mpsc::channel(1);
        let (send_broker_info, recv_broker_info) = mpsc::channel(1);
        Self {
            consumer: Some(consumer),
            send_init,
            send_poll_broker,
            recv_broker_info,
            poll_broker: None,
            recv_results,
            recv_status,
            target: None,
            status: None,
            results: None,
            broker_info: None,
            handle: tokio::spawn(async move {
                loop {
                    select! {
                        init = recv_init.recv() => {
                            let (consumer, target) = init.expect("Cannot recieve init command");

                            let (consumer, results) = match target.mode {
                                /*SearchTargetMode::End => {
                                    SearchTask::<SearchFromEnd>::new(
                                        consumer,
                                        &send_status,
                                        &topics,
                                    )
                                    .search(target.number)
                                    .await
                                }*/
                                SearchTargetMode::Timestamp { timestamp } => {
                                    SearchTask::<BinarySearchByTimestamp>::new(
                                        consumer,
                                        &send_status,
                                        &topics,
                                    )
                                    .search(timestamp, target.by, target.number)
                                    .await
                                } //_ => unimplemented!(),
                            };

                            send_results.send((consumer, results)).await.expect("");
                        }
                        poll_broker = recv_poll_broker.recv() => {
                            let consumer = poll_broker.expect("");
                            let trace = Self::poll_broker_topic_info::<TraceMessage>(&consumer, &topics.trace_topic).await;
                            let events = Self::poll_broker_topic_info::<EventListMessage>(&consumer, &topics.digitiser_event_topic).await;
                            
                            let broker_info = Option::zip(trace, events).map(|(trace,events)|BrokerInfo { trace, events });
                            send_broker_info.send((consumer, broker_info)).await.expect("");
                        }
                    }
                }
            }),
        }
    }

    async fn poll_broker_topic_info<'a, M : FBMessage<'a>>(consumer: &'a StreamConsumer, topic: &str) -> Option<BrokerTopicInfo> {
        let offsets = consumer.fetch_watermarks(topic, 0, Timeout::After(Duration::from_millis(1000))).ok()?;
        let mut searcher = Searcher::<M, StreamConsumer, _>::new(&consumer, topic, offsets.0, Offset::Offset).ok()?;
        let begin = searcher.message(offsets.0).await.ok()?;
        let end = searcher.message(offsets.1 - 1).await.ok()?;
        Some(BrokerTopicInfo {
            offsets,
            timestamps: (begin.timestamp(), end.timestamp())
        })
    }
}

impl Drop for SearchEngine {
    fn drop(&mut self) {
        self.handle.abort();
    }
}

impl MessageFinder for SearchEngine {
    type SearchMode = SearchMode;

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

        /*if !self.recv_halt.is_empty() {
            if let Some(consumer) = self.recv_halt.recv().await {
                self.consumer = Some(consumer);
            }
        }*/

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

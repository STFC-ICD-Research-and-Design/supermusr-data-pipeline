use rdkafka::consumer::StreamConsumer;
use tokio::{sync::mpsc, task::JoinHandle};
use tracing::{error, instrument};

use crate::{
    Topics,
    finder::{
        MessageFinder, SearchMode, SearchResults, SearchStatus, SearchTarget, SearchTargetMode,
        task::{BinarySearchByTimestamp, SearchTask},
    },
};

pub(crate) struct SearchEngine {
    /// The Kafka consumer object, the engine uses to poll for messages.
    ///
    /// The object takes temporary ownership of the consumer object,
    /// if another instance of SearchEngine wants to use it,
    /// it must be passed to it.
    consumer: Option<StreamConsumer>,
    target: Option<SearchTarget>,
    /// When another instance of [Self] is finished with the [StreamConsumer] object,
    /// it is passed back via this channel.
    send_init: mpsc::Sender<(StreamConsumer, SearchTarget)>,
    recv_results: mpsc::Receiver<(StreamConsumer, SearchResults)>,
    recv_status: mpsc::Receiver<SearchStatus>,
    status: Option<SearchStatus>,
    //
    results: Option<SearchResults>,
    //select: Select,
    //topics: Topics,
    /// When a search is in progress
    handle: JoinHandle<()>,
}

impl SearchEngine {
    pub(crate) fn new(consumer: StreamConsumer, topics: &Topics) -> Self {
        let topics = topics.clone();

        let (send_init, mut recv_init) = mpsc::channel(1);
        let (send_results, recv_results) = mpsc::channel(1);
        let (send_status, recv_status) = mpsc::channel(1);
        Self {
            consumer: Some(consumer),
            send_init,
            recv_results,
            recv_status,
            target: None,
            status: None,
            results: None,
            handle: tokio::spawn(async move {
                loop {
                    let (consumer, target) = recv_init.recv().await.expect("");

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
                        }
                        //_ => unimplemented!(),
                    };

                    send_results.send((consumer, results)).await.expect("");
                }
            }),
        }
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

    fn status(&mut self) -> Option<SearchStatus> {
        self.status.take()
    }

    fn results(&mut self) -> Option<SearchResults> {
        self.results.take()
    }

    async fn update(&mut self) {
        if let Some(target) = self.target.take() {
            if let Some(consumer) = self.consumer.take() {
                if let Err(e) = self.send_init.send((consumer, target)).await {
                    error!("send_init failed {e}");
                }
            } else {
                error!("Missing Consumer");
            }
        }

        if !self.recv_results.is_empty() {
            if let Some((consumer, results)) = self.recv_results.recv().await {
                self.consumer = Some(consumer);
                self.results = Some(results);
            }
        }

        /*if !self.recv_halt.is_empty() {
            if let Some(consumer) = self.recv_halt.recv().await {
                self.consumer = Some(consumer);
            }
        }*/

        if !self.recv_status.is_empty() {
            if let Some(status) = self.recv_status.recv().await {
                self.status = Some(status);
            }
        }
    }
}

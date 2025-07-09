mod binary_by_timestamp;

use std::marker::PhantomData;

use rdkafka::consumer::StreamConsumer;
use tokio::sync::mpsc;
use tracing::{error, instrument};

use crate::{Topics, finder::SearchStatus};

pub(crate) use binary_by_timestamp::BinarySearchByTimestamp;

pub(crate) trait TaskClass {}

pub(crate) struct SearchTask<'a, C: TaskClass> {
    consumer: StreamConsumer,
    send_status: &'a mpsc::Sender<SearchStatus>,
    topics: &'a Topics,
    phantom: PhantomData<C>,
}

impl<'a, C: TaskClass> SearchTask<'a, C> {
    pub(crate) fn new(
        consumer: StreamConsumer,
        send_status: &'a mpsc::Sender<SearchStatus>,
        topics: &'a Topics,
    ) -> Self {
        Self {
            consumer,
            send_status,
            topics,
            phantom: PhantomData,
        }
    }

    #[instrument(skip_all)]
    pub(crate) async fn emit_status(&self, new_status: SearchStatus) {
        if let Err(e) = self.send_status.send(new_status).await {
            error!("{e}");
        }
    }
}

mod binary_by_timestamp;

use std::marker::PhantomData;

use rdkafka::consumer::StreamConsumer;
use tracing::instrument;

use crate::structs::{Topics, SearchStatus};

pub(crate) use binary_by_timestamp::BinarySearchByTimestamp;

pub(crate) trait TaskClass {}

pub(crate) struct SearchTask<'a, C: TaskClass> {
    consumer: &'a StreamConsumer,
    //send_status: &'a mpsc::Sender<SearchStatus>,
    topics: &'a Topics,
    phantom: PhantomData<C>,
}

impl<'a, C: TaskClass> SearchTask<'a, C> {
    pub(crate) fn new(
        consumer: &'a StreamConsumer,
        topics: &'a Topics,
    ) -> Self {
        Self {
            consumer,
            topics,
            phantom: PhantomData,
        }
    }

    #[instrument(skip_all)]
    pub(crate) async fn emit_status(&self, new_status: SearchStatus) {
        /*if let Err(e) = self.send_status.send(new_status).await {
            error!("{e}");
        }*/
    }
}

mod binary_by_timestamp;

use std::marker::PhantomData;

use rdkafka::consumer::StreamConsumer;
use tracing::{info, instrument, warn};
use std::sync::{Mutex, Arc};

use crate::structs::{SearchStatus, Topics};

pub(crate) use binary_by_timestamp::BinarySearchByTimestamp;

pub(crate) trait TaskClass {}

pub(crate) struct SearchTask<'a, C: TaskClass> {
    consumer: &'a StreamConsumer,
    //send_status: &'a mpsc::Sender<SearchStatus>,
    topics: &'a Topics,
    status: Arc<Mutex<SearchStatus>>,
    phantom: PhantomData<C>,
}

impl<'a, C: TaskClass> SearchTask<'a, C> {
    pub(crate) fn new(consumer: &'a StreamConsumer, topics: &'a Topics, status: Arc<Mutex<SearchStatus>>) -> Self {
        Self {
            consumer,
            topics,
            status,
            phantom: PhantomData,
        }
    }

    #[instrument(skip_all)]
    pub(crate) async fn emit_status(&self, new_status: SearchStatus) {
        match self.status.try_lock() {
             Ok(mut status) => {
                info!("New status {new_status:?}");
                *status = new_status;
             },
             Err(e) => warn!{"{e}"}
        }
        /*if let Err(e) = self.send_status.send(new_status).await {
            error!("{e}");
        }*/
    }
}

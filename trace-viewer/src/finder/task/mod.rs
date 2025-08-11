//! Contains structs responsible for executing a particular search method.
//!
//!

mod binary_by_timestamp;

use std::{
    marker::PhantomData,
    sync::{LockResult, MutexGuard, PoisonError},
};

use rdkafka::consumer::StreamConsumer;
use std::sync::{Arc, Mutex};
use tracing::{instrument, warn};

use crate::{
    finder::status_sharer::StatusSharer,
    structs::{SearchStatus, Topics},
};
use thiserror::Error;

pub(crate) use binary_by_timestamp::BinarySearchByTimestamp;

pub(crate) trait TaskClass {}

pub(crate) struct SearchTask<'a, C: TaskClass> {
    consumer: &'a StreamConsumer,
    //send_status: &'a mpsc::Sender<SearchStatus>,
    topics: &'a Topics,
    //status: Arc<Mutex<SearchStatus>>,
    phantom: PhantomData<C>,
    status_send: StatusSharer,
}

impl<'a, C: TaskClass> SearchTask<'a, C> {
    pub(crate) fn new(
        consumer: &'a StreamConsumer,
        topics: &'a Topics,
        status_send: StatusSharer,
    ) -> Self {
        Self {
            consumer,
            topics,
            status_send,
            phantom: PhantomData,
        }
    }

    #[instrument(skip_all)]
    pub(crate) async fn emit_status(&self, new_status: SearchStatus) {
        self.status_send.emit(new_status).await;
    }
}

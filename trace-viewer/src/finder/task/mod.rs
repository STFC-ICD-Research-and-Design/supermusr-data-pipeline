mod binary_by_timestamp;

use std::{marker::PhantomData, sync::{LockResult, MutexGuard, PoisonError}};

use rdkafka::consumer::StreamConsumer;
//use tokio::sync::mpsc;
use std::sync::{Arc, Mutex};
use tracing::{debug, error, instrument, warn};

use crate::structs::{SearchStatus, Topics};
use thiserror::Error;

pub(crate) use binary_by_timestamp::BinarySearchByTimestamp;

#[derive(Debug, Error)]
pub enum StatusError {
    #[error("Oh no")]
    NoStatus,
    #[error("Oh no")]
    PoisonError(#[from]PoisonError<MutexGuard<'static, Option<SearchStatus>>>)
}

#[derive(Clone)]
pub struct StatusSharer {
    inner: Arc<Mutex<Option<SearchStatus>>>
    //inner: mpsc::Sender<SearchStatus>
}

impl StatusSharer {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(Some(SearchStatus::Off)))
        }
    }

    pub(crate) async fn emit(&self, new_status: SearchStatus) {
        debug!("Emitting status: {:?}.", new_status);
        
        match self.inner.lock() {
            Ok(mut status) => {
                status.replace(new_status);
                debug!("status successfully emitted");
            }
            Err(e) => warn! {"{e}"},
        }
        /*if let Err(e) = self.status_send.try_send(new_status) {
            warn!("{e}");
        }*/
    }

    pub async fn get(&self) -> Option<SearchStatus> {
        self.inner.lock()
            .expect("Mutex should lock, this should never fail.")
            .take()
    }
}

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

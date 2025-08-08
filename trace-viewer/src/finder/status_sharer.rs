use crate::structs::{BrokerInfo, SearchResults, SearchStatus, SearchTarget, Topics};

use std::{
    marker::PhantomData,
    sync::{LockResult, MutexGuard, PoisonError},
};

use rdkafka::consumer::StreamConsumer;
//use tokio::sync::mpsc;
use std::sync::{Arc, Mutex};
use tracing::{instrument, trace, warn};

#[derive(Clone)]
pub struct StatusSharer {
    inner: Arc<Mutex<Option<SearchStatus>>>, //inner: mpsc::Sender<SearchStatus>
}

impl StatusSharer {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(Some(SearchStatus::Off))),
        }
    }

    pub(crate) async fn emit(&self, new_status: SearchStatus) {
        trace!("Emitting status: {:?}.", new_status);

        match self.inner.lock() {
            Ok(mut status) => {
                status.replace(new_status);
                trace!("status successfully emitted");
            }
            Err(e) => warn! {"{e}"},
        }
        /*if let Err(e) = self.status_send.try_send(new_status) {
            warn!("{e}");
        }*/
    }

    pub async fn get(&self) -> Option<SearchStatus> {
        self.inner
            .lock()
            .expect("Mutex should lock, this should never fail.")
            .take()
    }
}

use chrono::{DateTime, TimeDelta, Utc};
use leptos::prelude::ServerFnError;
use serde_json::map::Entry;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
use tokio::{sync::oneshot, task::JoinHandle};
use tracing::{debug, error};
use uuid::Uuid;

use crate::{
    DefaultData,
    finder::{MessageFinder, SearchEngine},
    messages::{Cache, VectorisedCache},
    structs::{SearchResults, SearchStatus, SearchTarget, Topics},
};

#[derive(Default)]
pub struct SessionEngine {
    default_data: DefaultData,
    sessions: HashMap<String, Session>,
}

impl SessionEngine {
    pub fn new(default_data: DefaultData) -> Self {
        Self {
            default_data,
            sessions: Default::default(),
        }
    }
    fn generate_key(&self) -> String {
        let mut key = Uuid::new_v4().to_string();
        while self.sessions.contains_key(&key) {
            key = Uuid::new_v4().to_string();
        }
        key
    }

    pub fn create_new_search(
        &mut self,
        broker: String,
        trace_topic: String,
        digitiser_event_topic: String,
        consumer_group: String,
        target: SearchTarget,
    ) -> Result<String, ServerFnError> {
        let consumer = supermusr_common::create_default_consumer(
            &broker,
            &self.default_data.username,
            &self.default_data.password,
            &consumer_group,
            None,
        )?;

        let searcher = SearchEngine::new(
            consumer,
            &Topics {
                trace_topic,
                digitiser_event_topic,
            },
        );

        let key = self.generate_key();
        self.sessions.insert(
            key.clone(),
            Session::new_search(key.clone(), searcher, target),
        );
        Ok(key)
    }

    pub async fn get_search_results(&mut self, uuid: String) -> SearchResults {
        let session = self.sessions.get_mut(&uuid).expect("");

        let results = session.get_search_results().await;
        self.sessions.remove(&uuid);
        results
    }

    pub(crate) fn purge_expired(&mut self) {
        let dead_uuids: Vec<String> = self
            .sessions
            .keys()
            .filter(|&uuid| self.sessions.get(uuid).is_some_and(Session::expired))
            .cloned()
            .collect::<Vec<_>>();

        for uuid in dead_uuids {
            self.sessions.remove_entry(&uuid);
        }
    }
}

pub struct Session {
    uuid: String,
    pub results: SearchResults,
    pub status: Arc<Mutex<SearchStatus>>,
    handle: JoinHandle<()>,
    results_recv: Option<oneshot::Receiver<SearchResults>>,
    expiration: DateTime<Utc>,
}

impl Session {
    const EXPIRE_TIME_MIN: i64 = 10;

    fn new_search(uuid: String, mut searcher: SearchEngine, target: SearchTarget) -> Self {
        let (results_send, results_recv) = oneshot::channel();
        Session {
            uuid: uuid,
            results: SearchResults::default(),
            handle: tokio::task::spawn(async move {
                if let Err(e) = results_send.send(searcher.search(target).await) {
                    error!("{e:?}");
                }
            }),
            results_recv: Some(results_recv),
            status: Arc::new(Mutex::new(SearchStatus::Off)),
            expiration: Utc::now() + TimeDelta::minutes(Self::EXPIRE_TIME_MIN),
        }
    }
    pub async fn get_search_results(&mut self) -> SearchResults {
        self.results = self.results_recv.take().expect("").await.expect("");
        self.results.clone()
    }

    fn expired(&self) -> bool {
        self.expiration < Utc::now()
    }

    fn update(&mut self) {
        self.expiration = Utc::now() + TimeDelta::minutes(Self::EXPIRE_TIME_MIN)
    }
}

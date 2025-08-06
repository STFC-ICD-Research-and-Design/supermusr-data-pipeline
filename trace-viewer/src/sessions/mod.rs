use chrono::{DateTime, TimeDelta, Utc};
use leptos::prelude::ServerFnError;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
use tokio::{
    task::{JoinError, JoinHandle},
    time::Timeout,
};
use tracing::{debug, instrument};
use uuid::Uuid;

use crate::{
    finder::{MessageFinder, SearchEngine, StatusSharer}, messages::TraceWithEvents, structs::{SearchResults, SearchStatus, SearchTarget, SelectedTraceIndex, Topics, TraceSummary}, DefaultData
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
        broker: &String,
        topics: &Topics,
        consumer_group: &String,
        target: SearchTarget,
    ) -> Result<String, ServerFnError> {
        let consumer = supermusr_common::create_default_consumer(
            broker,
            &self.default_data.username,
            &self.default_data.password,
            consumer_group,
            None,
        )?;

        let status_sharer = StatusSharer::new();

        let searcher = SearchEngine::new(consumer, topics, status_sharer.clone());

        let key = self.generate_key();
        self.sessions.insert(
            key.clone(),
            Session::new_search(key.clone(), searcher, target, status_sharer),
        );
        Ok(key)
    }

    pub async fn get_session_status(&mut self, uuid: &str) -> Result<SearchStatus, ServerFnError> {
        let session_sharer = {
            let session = self.sessions.get_mut(uuid).expect("");
            debug!("Attempting to get session status");
            session.status_recv.clone()
        };
        loop {
            let status = session_sharer.get().await;
            if let Some(status) = status {
                debug!("Found session status: {status:?}");
                return Ok(status);
            }
        }
    }

    pub async fn cancel_session(&mut self, uuid: &str) -> Result<(), ServerFnError> {
        let session = self.sessions.get_mut(uuid).expect("");
        session
            .handle
            .take()
            .expect("This should not fail.")
            .abort();
        debug!("Session sucessfully cancelled.");
        self.sessions.remove(uuid);
        Ok(())
    }

    pub fn session(&self, uuid: &str) -> &Session {
        self.sessions.get(uuid).expect("")
    }

    pub fn session_mut(&mut self, uuid: &str) -> &mut Session {
        self.sessions.get_mut(uuid).expect("")
    }

    #[instrument(skip_all)]
    pub fn purge_expired(&mut self) {
        debug!("Purging expired sessions.");

        let dead_uuids: Vec<String> = self
            .sessions
            .keys()
            .filter(|&uuid| self.sessions.get(uuid).is_some_and(Session::expired))
            .cloned()
            .collect::<Vec<_>>();

        debug!("Found {} dead session(s)", dead_uuids.len());

        for uuid in dead_uuids {
            self.sessions.remove_entry(&uuid);
        }
    }
}

pub struct Session {
    uuid: String,
    results: Option<SearchResults>,
    handle: Option<JoinHandle<SearchResults>>,
    status_recv: StatusSharer,
    expiration: DateTime<Utc>,
}

impl Session {
    const EXPIRE_TIME_MIN: i64 = 10;

    fn new_search(
        uuid: String,
        mut searcher: SearchEngine,
        target: SearchTarget,
        status_recv: StatusSharer,
    ) -> Self {
        Session {
            uuid: uuid,
            results: None,
            handle: Some(tokio::task::spawn(
                async move { searcher.search(target).await },
            )),
            status_recv,
            expiration: Utc::now() + TimeDelta::minutes(Self::EXPIRE_TIME_MIN),
        }
    }

    #[instrument(skip_all)]
    pub fn take_search_handle(&mut self) -> JoinHandle<SearchResults> {
        self.handle
            .take()
            .expect("Search handle should be some, this should never fail.")
    }

    #[instrument(skip_all)]
    pub fn register_results(&mut self, result: Option<SearchResults>) {
        self.results = result;
    }

    #[instrument(skip_all)]
    pub fn get_search_summaries(&self) -> Option<Vec<TraceSummary>> {
        let mut digitiser_messages = self.results.as_ref()?.cache.iter().cloned().collect::<Vec<_>>();
        digitiser_messages.sort_by(|(metadata1, _), (metadata2, _)| {
            metadata1.timestamp.cmp(&metadata2.timestamp)
        });

        Some(digitiser_messages
            .iter()
            .enumerate()
            .map(|(index, (metadata, trace))| {
                let date = metadata
                    .timestamp
                    .date_naive()
                    .format("%y-%m-%d")
                    .to_string();
                let time = metadata.timestamp.time().format("%H:%M:%S.%f").to_string();
                let id = metadata.id;
                let channels = trace.traces.keys().copied().collect::<Vec<_>>();
                TraceSummary {
                    date,
                    time,
                    index,
                    id,
                    channels,
                }
            })
            .collect::<Vec<_>>())
    }

    pub fn get_selected_trace_index(&self, index_and_channel: SelectedTraceIndex) -> Result<Option<TraceWithEvents>, ServerFnError> {
        Ok(self.results.as_ref().map(|results| {
            let (metadata, trace) = results.cache.iter().nth(index_and_channel.index).expect("");
            TraceWithEvents::new(metadata, trace, index_and_channel.channel)
        }))
    }

    fn expired(&self) -> bool {
        self.expiration < Utc::now()
    }

    fn update(&mut self) {
        self.expiration = Utc::now() + TimeDelta::minutes(Self::EXPIRE_TIME_MIN)
    }
}

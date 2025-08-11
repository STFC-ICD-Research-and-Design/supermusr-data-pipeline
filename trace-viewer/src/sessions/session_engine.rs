use chrono::{DateTime, TimeDelta, Utc};
use leptos::prelude::{ServerFnError, use_context};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex, mpsc::Receiver},
};
use tokio::{
    sync::oneshot,
    task::{JoinError, JoinHandle},
    time::Timeout,
};
use tracing::{debug, instrument, trace};
use uuid::Uuid;

use crate::{
    app::{ServerError, SessionError},
    finder::{SearchEngine, StatusSharer},
    sessions::session::Session,
    structs::{
        BrokerInfo, SearchResults, SearchStatus, SearchTarget, SelectedTraceIndex, ServerSideData,
        Topics, TraceSummary,
    },
};

#[derive(Default)]
pub struct SessionEngine {
    server_side_data: ServerSideData,
    sessions: HashMap<String, Session>,
}

impl SessionEngine {
    pub fn new(server_side_data: &ServerSideData) -> Self {
        Self {
            server_side_data: server_side_data.clone(),
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

    pub fn create_new_search(&mut self, target: SearchTarget) -> Result<String, SessionError> {
        let consumer = supermusr_common::create_default_consumer(
            &self.server_side_data.broker,
            &self.server_side_data.username,
            &self.server_side_data.password,
            &self.server_side_data.consumer_group,
            None,
        )?;

        let status_sharer = StatusSharer::new();
        let searcher = SearchEngine::new(
            consumer,
            &self.server_side_data.topics,
            status_sharer.clone(),
        );

        let key = self.generate_key();
        self.sessions.insert(
            key.clone(),
            Session::new_search(key.clone(), searcher, target, status_sharer),
        );
        Ok(key)
    }

    pub async fn get_session_status(&mut self, uuid: &str) -> Result<SearchStatus, SessionError> {
        let session_sharer = {
            let session = self.session_mut(uuid)?;

            trace!("Attempting to get session status");

            session.get_status()
        };
        loop {
            let status = session_sharer.get().await;
            if let Some(status) = status {
                debug!("Found session status: {status:?}");
                return Ok(status);
            }
        }
    }

    pub fn cancel_session(&mut self, uuid: &str) -> Result<(), SessionError> {
        self.session_mut(uuid)?.cancel()
    }

    pub fn session(&self, uuid: &str) -> Result<&Session, SessionError> {
        self.sessions.get(uuid).ok_or(SessionError::DoesNotExist)
    }

    pub fn session_mut(&mut self, uuid: &str) -> Result<&mut Session, SessionError> {
        self.sessions
            .get_mut(uuid)
            .ok_or(SessionError::DoesNotExist)
    }

    #[instrument(skip_all)]
    pub fn purge_expired(&mut self) {
        let dead_uuids: Vec<String> = self
            .sessions
            .keys()
            .filter(|&uuid| self.sessions.get(uuid).is_some_and(Session::expired))
            .cloned()
            .collect::<Vec<_>>();

        debug!("Purging {} dead session(s)", dead_uuids.len());

        for uuid in dead_uuids {
            self.sessions.remove_entry(&uuid);
        }
    }

    #[instrument(skip_all)]
    pub async fn poll_broker(
        &self,
        poll_broker_timeout_ms: u64,
    ) -> Result<BrokerInfo, ServerError> {
        debug!("Beginning Broker Poll");
        trace!("{:?}", self.server_side_data);

        let consumer = supermusr_common::create_default_consumer(
            &self.server_side_data.broker,
            &self.server_side_data.username,
            &self.server_side_data.password,
            &self.server_side_data.consumer_group,
            None,
        )?;

        let searcher =
            SearchEngine::new(consumer, &self.server_side_data.topics, StatusSharer::new());

        Ok(searcher.poll_broker(poll_broker_timeout_ms).await?)
    }
}

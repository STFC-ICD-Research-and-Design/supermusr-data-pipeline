use crate::{
    app::{ServerError, SessionError},
    finder::{SearchEngine, StatusSharer},
    sessions::session::Session,
    structs::{BrokerInfo, SearchStatus, SearchTarget, Topics},
};
use std::{collections::HashMap, sync::Arc};
use tokio::{sync::Mutex, time::Duration};
use tracing::{debug, instrument, trace};
use uuid::Uuid;

/// Encapsulates all run-time settings which are needed by the session engine.
#[derive(Default, Clone, Debug)]
pub struct SessionEngineSettings {
    pub broker: String,
    pub topics: Topics,
    pub username: Option<String>,
    pub password: Option<String>,
    pub consumer_group: String,
    pub session_ttl_sec: i64,
}

#[derive(Default)]
pub struct SessionEngine {
    settings: SessionEngineSettings,
    sessions: HashMap<String, Session>,
}

impl SessionEngine {
    pub fn with_arc_mutex(settings: SessionEngineSettings) -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(Self {
            settings,
            sessions: Default::default(),
        }))
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
            &self.settings.broker,
            &self.settings.username,
            &self.settings.password,
            &self.settings.consumer_group,
            None,
        )?;

        let status_sharer = StatusSharer::new();
        let searcher = SearchEngine::new(
            consumer,
            &self.settings.topics,
            status_sharer.clone(),
        );

        let key = self.generate_key();
        self.sessions.insert(
            key.clone(),
            Session::new_search(searcher, target, status_sharer, self.settings.session_ttl_sec),
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

    pub fn spawn_purge_task(
        session_engine: Arc<Mutex<Self>>,
        purge_session_interval_sec: u64,
    ) -> tokio::task::JoinHandle<()> {
        tokio::task::spawn(async move {
            let mut interval =
                tokio::time::interval(Duration::from_secs(purge_session_interval_sec));

            loop {
                interval.tick().await;
                session_engine.lock().await.purge_expired();
            }
        })
    }

    #[instrument(skip_all)]
    pub async fn poll_broker(
        &self,
        poll_broker_timeout_ms: u64,
    ) -> Result<BrokerInfo, ServerError> {
        debug!("Beginning Broker Poll");
        trace!("{:?}", self.settings);

        let consumer = supermusr_common::create_default_consumer(
            &self.settings.broker,
            &self.settings.username,
            &self.settings.password,
            &self.settings.consumer_group,
            None,
        )?;

        let searcher =
            SearchEngine::new(consumer, &self.settings.topics, StatusSharer::new());

        Ok(searcher.poll_broker(poll_broker_timeout_ms).await?)
    }
}

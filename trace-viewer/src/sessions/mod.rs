use chrono::{DateTime, TimeDelta, Utc};
use leptos::prelude::ServerFnError;
use tracing::debug;
use uuid::Uuid;
use std::{collections::HashMap, sync::{Arc, Mutex}};

use leptos_actix::extract;
use actix_session::Session as ActixSession;
use crate::{app::AppUuid, messages::{Cache, VectorisedCache}, structs::SearchStatus};

#[derive(Default)]
pub struct SessionEngine {
    sessions: HashMap<String, Session>,
}

impl SessionEngine {
    pub async fn get_key() -> Result<String, ServerFnError> {
        let session: ActixSession = extract().await?;
        match session.get::<String>("trace-viewer")? {
            Some(key) => {
                debug!("Cookie key found: {}", key);
                Ok(key)
            },
            None => {
                let key = Uuid::new_v4().to_string();
                session.insert("id", &key)?;
                debug!("New key created: {}", key);
                Ok(key)
            }
        }
    }

    fn session(&mut self, uuid: &str) -> &mut Session {
        self.sessions.entry(uuid.to_owned())
            .and_modify(Session::update)
            .or_insert_with(||Session::new(uuid))
    }

    pub fn query_session<R, F : Fn(&Session) -> R>(&mut self, uuid: &str, f : F) -> R {
        f(self.session(uuid))
    }
//.expect("Uuid should be valid for modify, this should never fail.")
    pub fn modify_session<F : Fn(&mut Session)>(&mut self, uuid: &str, f : F) {
        f(self.session(uuid));
    }
    
    pub(crate) fn purge_expired(&mut self) {
        let dead_uuids: Vec<String> = self.sessions.keys()
            .filter(|&uuid|
                self.sessions.get(uuid)
                    .is_some_and(Session::expired)
            ).cloned()
            .collect::<Vec<_>>();

        for uuid in dead_uuids {
            self.sessions.remove_entry(&uuid);
        }
    }
    
    pub(crate) fn update(&mut self, uuid: &str) {
        self.session(uuid);
    }
}

pub struct Session {
    uuid: String,
    pub cache: VectorisedCache,
    pub status: Arc<Mutex<SearchStatus>>,
    expiration: DateTime<Utc>,
}

impl Session {
    const EXPIRE_TIME_MIN: i64 = 10;

    fn new(uuid: &str) -> Self {
        Session {
            uuid: uuid.to_owned(),
            cache: VectorisedCache::default(),
            status: Arc::new(Mutex::new(SearchStatus::Off)),
            expiration: Utc::now() + TimeDelta::minutes(Self::EXPIRE_TIME_MIN)
        }
    }

    fn expired(&self) -> bool {
        self.expiration < Utc::now()
    }

    fn update(&mut self) {
        self.expiration = Utc::now() + TimeDelta::minutes(Self::EXPIRE_TIME_MIN)
    }
}
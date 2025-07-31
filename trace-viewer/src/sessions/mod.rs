use chrono::TimeDelta;
use uuid::Uuid;

use crate::messages::Cache;

pub(crate) struct SessionEngine {
    sessions: HashMap<Uuid, Session>,
}

impl SessionEngine {
    pub(crate) fn new() -> Self {
        SessionEngine {
            sessions: HashMap::new(),
        }
    }

    pub(crate) fn new_session(&mut self) -> Uuid {
        let session = Session::new();
        let uuid = session.uuid;
        self.sessions.insert(uuid, session);
        uuid
    }

    pub(crate) fn session(&self, uuid: Uuid) -> &Session {
        self.sessions.get(uuid)
    }

    pub(crate) fn session_mut(&self, uuid: Uuid) -> &mut Session {
        self.sessions.get_mut(uuid)
    }
    
    pub(crate) fn purge_expired(&mut self) {
        for session in self.sessions {
            if session.expired() {

            }
        }
    }
    
    pub(crate) fn update(&mut self, uuid: &Uuid) {
        self.sessions.entry(uuid).update();
    }
}

pub(crate) struct Session {
    uuid: Uuid,
    cache: Cache,
    expiration: DateTime<Utc>,
}

impl Session {
    const EXPIRE_TIME_MIN: i64 = 10;

    pub(crate) fn new() -> Self {
        Session {
            uuid: Uuid::new_v4(),
            cache: Cache::default(),
            expiration: Utc::now() + TimeDelta::minutes(Self::EXPIRE_TIME_MIN)
        }
    }

    pub(crate) fn expired(&self) -> bool {
        self.expiration < Utc::now()
    }

    pub(crate) fn update(&mut self) {
        self.expiration = Utc::now()
    }
}
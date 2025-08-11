use crate::{
    Timestamp,
    app::{ServerError, SessionError},
    finder::{SearchEngine, StatusSharer},
    structs::{
        BrokerInfo, DigitiserMetadata, DigitiserTrace, SearchResults, SearchStatus, SearchTarget,
        SelectedTraceIndex, ServerSideData, Topics, TraceSummary,
    },
};
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

pub struct SessionSearchBody {
    pub handle: JoinHandle<Result<SearchResults, SessionError>>,
    pub cancel_recv: oneshot::Receiver<()>,
}

pub struct Session {
    uuid: String,
    results: Option<SearchResults>,
    status: StatusSharer,
    search_body: Option<SessionSearchBody>,
    cancel_send: Option<oneshot::Sender<()>>,
    expiration: Timestamp,
}

impl Session {
    const EXPIRE_TIME_MIN: i64 = 10;

    pub(crate) fn new_search(
        uuid: String,
        mut searcher: SearchEngine,
        target: SearchTarget,
        status: StatusSharer,
    ) -> Self {
        let (cancel_send, cancel_recv) = oneshot::channel();
        Session {
            uuid: uuid,
            results: None,
            search_body: Some(SessionSearchBody {
                handle: tokio::task::spawn(async move { Ok(searcher.search(target).await?) }),
                cancel_recv,
            }),
            status,
            cancel_send: Some(cancel_send),
            expiration: Utc::now() + TimeDelta::minutes(Self::EXPIRE_TIME_MIN),
        }
    }

    pub(crate) fn get_status(&self) -> StatusSharer {
        self.status.clone()
    }

    #[instrument(skip_all)]
    pub fn take_search_body(&mut self) -> Result<SessionSearchBody, SessionError> {
        self.search_body
            .take()
            .ok_or(SessionError::BodyAlreadyTaken)
    }

    #[instrument(skip_all)]
    pub fn cancel(&mut self) -> Result<(), SessionError> {
        self.cancel_send
            .take()
            .ok_or(SessionError::AttemptedToCancelTwice)?
            .send(())
            .map_err(|_| SessionError::CouldNotSendCancelSignal)
    }

    #[instrument(skip_all)]
    pub fn register_results(&mut self, result: SearchResults) {
        self.results = Some(result);
    }

    #[instrument(skip_all)]
    pub fn get_search_summaries(&self) -> Result<Vec<TraceSummary>, SessionError> {
        let mut digitiser_messages = self
            .results
            .as_ref()
            .ok_or(SessionError::ResultsMissing)?
            .cache()?
            .iter()
            .collect::<Vec<_>>();

        digitiser_messages.sort_by(|(metadata1, _), (metadata2, _)| {
            metadata1.timestamp.cmp(&metadata2.timestamp)
        });

        Ok(digitiser_messages
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

    pub fn get_selected_trace<'a>(
        &'a self,
        index: usize,
    ) -> Result<(&'a DigitiserMetadata, &'a DigitiserTrace), SessionError> {
        self.results
            .as_ref()
            .ok_or(SessionError::ResultsMissing)?
            .cache()?
            .iter()
            .nth(index)
            .ok_or(SessionError::TraceNotFound)
    }

    pub(crate) fn expired(&self) -> bool {
        self.expiration < Utc::now()
    }

    pub(crate) fn refresh(&mut self) {
        self.expiration = Utc::now() + TimeDelta::minutes(Self::EXPIRE_TIME_MIN)
    }
}

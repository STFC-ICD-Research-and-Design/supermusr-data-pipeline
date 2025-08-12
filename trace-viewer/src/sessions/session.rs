use crate::{
    Timestamp,
    app::SessionError,
    finder::{SearchEngine, StatusSharer},
    structs::{DigitiserMetadata, DigitiserTrace, SearchResults, SearchTarget, TraceSummary},
};
use chrono::{TimeDelta, Utc};
use tokio::{sync::oneshot, task::JoinHandle};
use tracing::instrument;

pub struct SessionSearchBody {
    pub(crate) handle: JoinHandle<Result<SearchResults, SessionError>>,
    pub(crate) cancel_recv: oneshot::Receiver<()>,
}

pub struct Session {
    results: Option<SearchResults>,
    status: StatusSharer,
    search_body: Option<SessionSearchBody>,
    cancel_send: Option<oneshot::Sender<()>>,
    expiration: Timestamp,
}

impl Session {
    const EXPIRE_TIME_MIN: i64 = 10;

    pub(crate) fn new_search(
        mut searcher: SearchEngine,
        target: SearchTarget,
        status: StatusSharer,
    ) -> Self {
        let (cancel_send, cancel_recv) = oneshot::channel();
        Session {
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

    pub(crate) fn get_selected_trace(
        &self,
        index: usize,
    ) -> Result<(&DigitiserMetadata, &DigitiserTrace), SessionError> {
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

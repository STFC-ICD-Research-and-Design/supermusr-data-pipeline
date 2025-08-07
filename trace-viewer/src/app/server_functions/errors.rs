use cfg_if::cfg_if;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error, Serialize, Deserialize)]
pub enum SessionError {
    #[error("No such session exists.")]
    DoesNotExist,
    #[error("The session's search body has already been taken.")]
    BodyAlreadyTaken,
    #[error("The session's results have not been registered.")]
    ResultsMissing,
    #[error("The session's search was cancelled by the user.")]
    SearchCancelled,
    #[error("The requested trace message does not exist in the cache.")]
    TraceNotFound,
    #[error("The requested channel does not exist in the trace message.")]
    ChannelNotFound,
    #[error("Two cancel requests were made.")]
    AttemptedToCancelTwice,
    #[error("Could not send the cancel signal.")]
    CouldNotSendCancelSignal,
    #[error("Kafka Error Code: {0}")]
    Kafka(String),
}

cfg_if! {
    if #[cfg(feature = "ssr")] {
        use rdkafka::error::KafkaError;

        impl From<KafkaError> for SessionError {
            fn from(e: KafkaError) -> Self {
                Self::Kafka(e.rdkafka_error_code().as_ref().map(ToString::to_string).unwrap_or_default())
            }
        }
    }
}

#[derive(Debug, Error, Serialize, Deserialize)]
pub enum ServerError {
    #[error("Cannot get lock on Server Engine. Mutex poisoned.")]
    CannotObtainSessionEngine,
    #[error("{0}")]
    Session(SessionError),
}

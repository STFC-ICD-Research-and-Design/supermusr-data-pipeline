use std::fmt::Display;

use flatbuffers::InvalidFlatbuffer;
use redpanda::error::KafkaError;
use tdengine;

#[derive(Debug)]
pub enum DotEnvWriteError {
    CannotObtainCurrentDirectory(std::io::Error),
    CannotParsePath,
    CannotCreateDotEnvFile(std::io::Error),
    CannotWriteToDotEnvFile(std::io::Error),
    CannotFlushDotEnvFile(std::io::Error),
    IOError(std::io::Error),
}

#[derive(Debug)]
pub enum MessageError {
    UnexpectedMessageWithTopic(String),
    NoPayloadInMessageWithTopic(String),
    FailedToParseMessageWithTopic(String,InvalidFlatbuffer),
}

#[derive(Debug)]
pub enum Error {
    TDEngine(tdengine::error::Error),
    Kafka(KafkaError),
    EnvVar(&'static str),
    Message(MessageError),
    DotEnvWrite(DotEnvWriteError),
}

impl From<DotEnvWriteError> for Error {
    fn from(value: DotEnvWriteError) -> Self { Self::DotEnvWrite(value) }
}

impl From<KafkaError> for Error {
    fn from(value: KafkaError) -> Self { Self::Kafka(value) }
}

impl From<MessageError> for Error {
    fn from(value: MessageError) -> Self { Self::Message(value) }
}

impl Display for Error {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        fmt.write_fmt(format_args!("{self:?}"))
    }
}

impl std::error::Error for Error {}

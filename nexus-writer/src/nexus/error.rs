use std::{num::TryFromIntError, path::PathBuf};

use chrono::{DateTime, Utc};
use glob::{GlobError, PatternError};
use supermusr_streaming_types::time_conversions::GpsTimeConversionError;
use thiserror::Error;

use super::hdf5_file::NexusHDF5Error;

pub(crate) type NexusWriterResult<T> = Result<T, NexusWriterError>;

#[derive(Debug, Error)]
pub(crate) enum ErrorCodeLocation {
    #[error("set_stop_if_valid")]
    SetStopIfValid,
    #[error("set_aborted_run")]
    SetAbortedRun,
    #[error("RunParameters::new")]
    NewRunParamemters,
}

#[derive(Debug, Error)]
pub(crate) enum NexusWriterError {
    #[error("{0}")]
    HDF5(#[from] NexusHDF5Error),
    #[error("Flatbuffer Timestamp Conversion Error {0}")]
    FlatBufferTimestampConversion(#[from] GpsTimeConversionError),
    #[error("{0}")]
    FlatBufferMissing(FlatBufferMissingError),
    #[error("Unexpected RunStop Command")]
    UnexpectedRunStop,
    #[error("Cannot convert local path to string: {0}")]
    CannotConvertPath(PathBuf),
    #[error("Cannot convert filename to string: {0:?}")]
    CannotConvertFilename(PathBuf),
    #[error("Glob Pattern Error: {0}")]
    GlobPattern(#[from] PatternError),
    #[error("Glob Error: {0}")]
    Glob(#[from] GlobError),
    #[error("Integer Conversion Error")]
    TryFromInt(#[from] TryFromIntError),
    #[error("Start Time {int} Out of Range for DateTime at {location}")]
    IntOutOfRangeForDateTime {
        int: u64,
        location: ErrorCodeLocation,
    },
    #[error("Stop Command before Start Command at {0}")]
    StopCommandBeforeStartCommand(ErrorCodeLocation),
    #[error("Stop Time {stop} earlier than current Start Time {start} at {location}")]
    StopTimeEarlierThanStartTime {
        start: DateTime<Utc>,
        stop: DateTime<Utc>,
        location: ErrorCodeLocation,
    },
    #[error("RunStop already set at {0}")]
    RunStopAlreadySet(ErrorCodeLocation),
}

#[derive(Debug, Error)]
pub(crate) enum FlatBufferMissingError {
    #[error("Flatbuffer Timestamp Missing")]
    Timestamp,
    #[error("Flatbuffer Channels Missing")]
    Channels,
    #[error("Flatbuffer Intensities Missing")]
    Intensities,
    #[error("Flatbuffer Times Missing")]
    Times,
    #[error("Flatbuffer Run Name Missing")]
    RunName,
    #[error("Flatbuffer Instrument Name Missing")]
    InstrumentName,
}

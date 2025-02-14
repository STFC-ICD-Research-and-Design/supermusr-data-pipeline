use super::{hdf5_file::NexusHDF5Error, NexusDateTime};
use glob::{GlobError, PatternError};
use std::{num::TryFromIntError, path::PathBuf};
use supermusr_streaming_types::time_conversions::GpsTimeConversionError;
use thiserror::Error;

pub(crate) type NexusWriterResult<T> = Result<T, NexusWriterError>;

#[derive(Debug, Error)]
pub(crate) enum ErrorCodeLocation {
    #[error("set_stop_if_valid")]
    SetStopIfValid,
    #[error("set_aborted_run")]
    SetAbortedRun,
    #[error("RunParameters::new")]
    NewRunParamemters,
    #[error("stop_command")]
    StopCommand,
    #[error("process_event_list")]
    ProcessEventList,
    #[error("resume_partial_runs local directory path")]
    ResumePartialRunsLocalDirectoryPath,
    #[error("resume_partial_runs file path")]
    ResumePartialRunsFilePath,
}

#[derive(Debug, Error)]
pub(crate) enum NexusWriterError {
    #[error("{0}")]
    HDF5(#[from] NexusHDF5Error),
    #[error("Flatbuffer Timestamp Conversion Error {0}")]
    FlatBufferTimestampConversion(#[from] GpsTimeConversionError),
    #[error("{0}")]
    FlatBufferMissing(FlatBufferMissingError, ErrorCodeLocation),
    #[error("Unexpected RunStop Command")]
    UnexpectedRunStop(ErrorCodeLocation),
    #[error("Cannot convert local path to string: {path}")]
    CannotConvertPath {
        path: PathBuf,
        location: ErrorCodeLocation,
    },
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
        start: NexusDateTime,
        stop: NexusDateTime,
        location: ErrorCodeLocation,
    },
    #[error("RunStop already set at {0}")]
    RunStopAlreadySet(ErrorCodeLocation),
}

#[derive(Debug, Error)]
pub(crate) enum FlatBufferInvalidDataTypeContext {
    #[error("Run Log")]
    RunLog,
    #[error("Sample Environment Log")]
    SELog,
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

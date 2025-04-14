use crate::{hdf5_handlers::NexusHDF5Error, run_engine::NexusDateTime};
use glob::{GlobError, PatternError};
use std::{num::TryFromIntError, path::PathBuf};
use supermusr_streaming_types::time_conversions::GpsTimeConversionError;
use thiserror::Error;

pub(crate) type NexusWriterResult<T> = Result<T, NexusWriterError>;

#[derive(Debug, strum::Display)]
pub(crate) enum ErrorCodeLocation {
    #[strum(to_string = "flush_to_archive")]
    FlushToArchive,
    #[strum(to_string = "RunParameters::new")]
    NewRunParamemters,
    #[strum(to_string = "process_event_list")]
    ProcessEventList,
    #[strum(to_string = "resume_partial_runs file path")]
    ResumePartialRunsFilePath,
    #[strum(to_string = "resume_partial_runs local directory path")]
    ResumePartialRunsLocalDirectoryPath,
    #[strum(to_string = "set_aborted_run")]
    SetAbortedRun,
    #[strum(to_string = "set_stop_if_valid")]
    SetStopIfValid,
    #[strum(to_string = "stop_command")]
    StopCommand,
}

#[derive(Debug, Error)]
pub(crate) enum NexusWriterError {
    #[error("{0}")]
    HDF5(#[from] NexusHDF5Error),
    #[error("Glob Pattern Error: {0}")]
    GlobPattern(#[from] PatternError),
    #[error("Glob Error: {0}")]
    Glob(#[from] GlobError),
    #[error("IO Error: {0}")]
    IO(#[from] std::io::Error),
    #[error("Integer Conversion Error")]
    TryFromInt(#[from] TryFromIntError),
    #[error("Flatbuffer Timestamp Conversion Error {0}")]
    FlatBufferTimestampConversion(#[from] GpsTimeConversionError),
    #[error("{0} at {1}")]
    FlatBufferMissing(FlatBufferMissingError, ErrorCodeLocation),
    #[error("Cannot convert local path to string: {path} at {location}")]
    CannotConvertPath {
        path: PathBuf,
        location: ErrorCodeLocation,
    },
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
    #[error("Unexpected RunStop Command at {0}")]
    RunStopUnexpected(ErrorCodeLocation),
}

#[derive(Debug, strum::Display)]
pub(crate) enum FlatBufferInvalidDataTypeContext {
    #[strum(to_string = "Run Log")]
    RunLog,
    #[strum(to_string = "Sample Environment Log")]
    SELog,
}

#[derive(Debug, Error)]
pub(crate) enum FlatBufferMissingError {
    #[error("Timestamp Missing from Flatbuffer FrameEventList Message")]
    Timestamp,
    #[error("Channels Missing from Flatbuffer FrameEventList Message")]
    Channels,
    #[error("Intensities Missing from Flatbuffer FrameEventList Message")]
    Intensities,
    #[error("Times Missing from Flatbuffer FrameEventList Message")]
    Times,
    #[error("Run Name Missing from Flatbuffer RunStart Message")]
    RunName,
    #[error("Instrument Name Missing from Flatbuffer RunStart Message")]
    InstrumentName,
    #[error("Source Name Missing from Flatbuffer Alarm Message")]
    AlarmName,
    #[error("Severity Missing from Flatbuffer Alarm Message")]
    AlarmSeverity,
    #[error("Status Missing from Flatbuffer Alarm Message")]
    AlarmMessage,
}

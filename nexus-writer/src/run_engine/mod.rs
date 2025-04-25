//! This module contains the `NexusEngine` and `Run` objects,
//! as well as all objects which process flatbuffer messages.
mod engine;
mod run;
pub(crate) mod run_messages;
mod settings;

use chrono::{DateTime, Utc};
pub(crate) use engine::NexusEngine;
pub(crate) use run::{NexusConfiguration, Run, RunParameters, RunStopParameters};
pub(crate) use settings::{
    AlarmChunkSize, ChunkSizeSettings, EventChunkSize, FrameChunkSize, NexusSettings,
    PeriodChunkSize,
};

pub(crate) type NexusDateTime = DateTime<Utc>;

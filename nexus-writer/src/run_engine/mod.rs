//! Handles all runs and handles different flatbuffer messages.
mod engine;
mod run;
pub(crate) mod run_messages;
mod settings;

use chrono::{DateTime, Utc};
pub(crate) use engine::{NexusEngine, NexusEngineDependencies};
pub(crate) use run::{NexusConfiguration, Run, RunParameters, RunStopParameters};
pub(crate) use settings::{
    AlarmChunkSize, ChunkSizeSettings, EventChunkSize, FrameChunkSize, NexusSettings,
    PeriodChunkSize,
};

/// UTC-timezoned DateTime type to reduce boiler plate.
///
/// If the program is ever required to handle different timezones, this is where we need to look at.
pub(crate) type NexusDateTime = DateTime<Utc>;

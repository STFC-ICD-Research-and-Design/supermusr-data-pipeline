pub(crate) mod alarm;
pub(crate) mod create_messages;
pub(crate) mod runlog;
pub(crate) mod sample_environment;

use std::num::{ParseFloatError, ParseIntError, TryFromIntError};
use thiserror::Error;

use alarm::SeverityLevel;
use chrono::{DateTime, Utc};
use clap::{Parser, Subcommand};
use runlog::ValueType;
use sample_environment::{LocationType, ValuesType};

#[derive(Debug, Error)]
pub(crate) enum RunCommandError {
    #[error("No Values for Run Log Data")]
    EmptyRunLogSlice,
    #[error("Invalid Int Conversion: {0}")]
    IntConversion(#[from] TryFromIntError),
    #[error("Invalid String to Int: {0}")]
    IntFromStr(#[from] ParseIntError),
    #[error("Invalid String to Float {0}")]
    FloatFromStr(#[from] ParseFloatError),
    #[error("Timestamp cannot be Converted to Nanos: {0}")]
    TimestampToNanos(DateTime<Utc>),
}

#[derive(Clone, Parser)]
pub(crate) struct Start {
    /// Topic to publish command to
    #[clap(long)]
    topic: String,

    /// Timestamp of the command, defaults to now, if not given.
    #[clap(long)]
    time: Option<DateTime<Utc>>,

    /// Unique name of the run
    #[clap(long)]
    run_name: String,

    /// Name of the instrument being run
    #[clap(long)]
    instrument_name: String,
}

#[derive(Clone, Parser)]
pub(crate) struct Stop {
    /// Topic to publish command to
    #[clap(long)]
    topic: String,

    /// Timestamp of the command, defaults to now, if not given.
    #[clap(long)]
    time: Option<DateTime<Utc>>,

    /// Unique name of the run
    #[clap(long)]
    run_name: String,
}

#[derive(Clone, Debug, Parser)]
pub(crate) struct RunLogData {
    /// Topic to publish command to
    #[clap(long)]
    topic: String,

    /// Timestamp of the command, defaults to now, if not given.
    #[clap(long)]
    time: Option<DateTime<Utc>>,

    /// Name of the source being logged
    #[clap(long)]
    source_name: String,

    /// Type of the logdata
    #[clap(long)]
    value_type: ValueType,

    /// Value of the logdata
    #[clap()]
    value: Vec<String>,
}

#[derive(Clone, Debug, Parser)]
pub(crate) struct SampleEnvData {
    /// Topic to publish command to
    #[clap(long)]
    topic: String,

    /// Timestamp of the command, defaults to now, if not given.
    #[clap(long)]
    time: Option<DateTime<Utc>>,

    /// Name of the source being logged
    #[clap(long)]
    name: String,

    /// Optional: the channel id associated with the sample environment
    #[clap(long)]
    channel: Option<i32>,

    /// Optional: time between each sample in ns
    #[clap(long)]
    time_delta: Option<f64>,

    /// Type of the sample value
    #[clap(long, default_value = "int64")]
    values_type: ValuesType,

    /// Incrementing counter
    #[clap(long)]
    message_counter: Option<i64>,

    /// If sample timestamps are given, location specifies the temporal position to which the timestamps refer
    #[clap(long, default_value = "unknown")]
    location: LocationType,

    /// Vector of sample values
    #[clap()]
    values: Vec<String>,

    #[command(subcommand)]
    timestamps: Option<SampleEnvTimestamp>,
}

#[derive(Clone, Debug, Subcommand)]
enum SampleEnvTimestamp {
    Timestamps(SampleEnvTimestampData),
}

#[derive(Clone, Debug, Parser)]
pub(crate) struct SampleEnvTimestampData {
    /// Optional vector of timestamps to include (if used should be the same length as the `values` vector)
    #[clap()]
    timestamps: Vec<DateTime<Utc>>,
}

#[derive(Clone, Debug, Parser)]
pub(crate) struct AlarmData {
    /// Topic to publish command to
    #[clap(long)]
    topic: String,

    /// Timestamp of the command, defaults to now, if not given.
    #[clap(long)]
    time: Option<DateTime<Utc>>,

    /// Source Name of the alarm message
    #[clap(long)]
    source_name: String,

    /// Severity level of the alarm message
    #[clap(long)]
    severity: SeverityLevel,

    /// Custom text message of the alarm
    #[clap(long)]
    message: String,
}

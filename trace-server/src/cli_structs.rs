use crate::Timestamp;
use clap::Args;
use supermusr_common::{Channel, DigitizerId};

#[derive(Clone, Debug, Args)]
pub struct Topics {
    /// Kafka trace topic.
    #[clap(long)]
    pub(crate) trace_topic: String,

    /// Kafka digitiser event list topic.
    #[clap(long)]
    pub(crate) digitiser_event_topic: String,
}

#[derive(Clone, Debug, Args)]
pub struct Select {
    /// The timestamp of the frame to search for, should be in the format "YYYY-MM-DD hh:mm:ss.f <timezone>".
    #[clap(long)]
    pub(crate) timestamp: Option<Timestamp>,
}

#[derive(Clone, Debug, Args)]
pub(crate) struct Steps {
    /// The min step size that the Kafka searcher takes backwards in time when seeking the timestamp.
    #[clap(long, default_value = "50")]
    pub(crate) min_step_size: i64,

    /// The max step size that the Kafka searcher takes backwards in time when seeking the timestamp.
    #[clap(long, default_value = "10")]
    pub(crate) step_mul_coef: i64,

    /// The max step size that the Kafka searcher takes backwards in time when seeking the timestamp.
    #[clap(long, default_value = "5")]
    pub(crate) num_step_passes: u32,
}

use serde::Deserialize;
use supermusr_common::{Channel, DigitizerId, FrameNumber, Intensity, Time};

use crate::integrated::Interval;

pub(crate) type TraceSourceId = usize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum FrameSource {
    AggregatedFrame {
        num_channels: usize,
        source: TraceSourceId,
    },
    AutoDigitisers {
        num_digitiser: usize,
        num_channels_per_digitiser: usize,
        source: TraceSourceId,
    },
    DigitiserList(Vec<Digitiser>),
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct Digitiser {
    pub(crate) id: DigitizerId,
    pub(crate) channels: Interval<Channel>,
    pub(crate) source: TraceSourceId,
}

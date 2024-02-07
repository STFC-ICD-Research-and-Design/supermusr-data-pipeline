use chrono::{DateTime, Utc};
use serde::Deserialize;
use supermusr_common::{Intensity, Time, DigitizerId, FrameNumber};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case", untagged)]
pub(crate) enum Distribution<T> {
  Constant(T),
  Uniform {
    min: T,
    max: T,
  },
  Normal {
    mean: T,
    sd: T,
  }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct Pulse {
  weight: f64,
  attributes: PulseAttributes,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case", tag = "type")]
pub(crate) enum PulseAttributes {
  Gaussian {
    peak_height: Distribution<Intensity>,
    peak_time: Distribution<Time>,
    sd: Distribution<Time>
  }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct Interval<T> { min : T, max : T }

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct Transformation<T> { scale : T, translate : T }

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct Simulation {
  time_bins: Time,
  voltage: Interval<Intensity>,
  voltage_transformation: Transformation<f64>,
  sample_rate: u32,
  trace_messages: Vec<TraceMessage>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct TraceMessage {
  digitizer_ids: Vec<DigitizerId>,
  frames: Vec<FrameNumber>,
  pulses: Vec<Pulse>,
  noises: Option<Vec<usize>>,
  channels: usize,
  num_pulses: Distribution<usize>,
  timestamp: Option<String>,
  frame_delay_ns: u64,
}
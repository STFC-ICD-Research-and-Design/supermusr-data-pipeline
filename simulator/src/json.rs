use chrono::{DateTime, Utc};
use rand::Rng;
use serde::Deserialize;
use supermusr_common::{Channel, DigitizerId, FrameNumber, Intensity, Time};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case", untagged)]
pub(crate) enum Distribution<T> {
  Constant(T),
  Uniform { min: T, max: T },
  Normal { mean: T, sd: T }
}


impl Distribution<f64> {
  pub(crate) fn sample(&self) -> f64 {
    match self {
      Self::Constant(t) => *t,
      Self::Uniform { min, max } => rand::thread_rng().gen_range(*min..*max),
      Self::Normal { mean, sd} => *mean,
    }
  }
}

impl Distribution<Time> {
  pub(crate) fn sample(&self) -> Time {
    match self {
      Self::Constant(t) => *t,
      Self::Uniform { min, max } => rand::thread_rng().gen_range(*min..*max),
      Self::Normal { mean, sd} => *mean,
    }
  }
}


impl Distribution<Intensity> {
  pub(crate) fn sample(&self) -> Intensity {
    match self {
        Self::Constant(t) => *t,
        Self::Uniform { min, max } => rand::thread_rng().gen_range(*min..*max),
        Self::Normal { mean, sd} => *mean,
    }
  }
}

impl Distribution<usize> {
    pub(crate) fn sample(&self) -> usize {
      match self {
          Self::Constant(t) => *t,
          Self::Uniform { min, max } => rand::thread_rng().gen_range(*min..*max),
          Self::Normal { mean, sd} => *mean,
      }
    }
}


#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct Pulse {
  pub(crate) weight: f64,
  pub(crate) attributes: PulseAttributes,
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
pub(crate) struct Interval<T> { pub(crate) min : T, pub(crate) max : T }

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct Transformation<T> { pub(crate) scale : T, pub(crate) translate : T }

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct Digitizer {
  pub(crate) id: DigitizerId,
  pub(crate) channels: Interval<Channel>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct TraceMessage {
  pub(crate) time_bins: Time,
  pub(crate) digitizers: Vec<Digitizer>,
  pub(crate) frames: Vec<FrameNumber>,
  pub(crate) pulses: Vec<Pulse>,
  pub(crate) noises: Option<Vec<usize>>,
  pub(crate) num_pulses: Distribution<usize>,
  pub(crate) timestamp: Option<String>,
  pub(crate) frame_delay_ns: u64,
}


#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct Simulation {
  pub(crate) voltage: Interval<Intensity>,
  pub(crate) voltage_transformation: Transformation<f64>,
  pub(crate) sample_rate: u32,
  pub(crate) traces: Vec<TraceMessage>,
}
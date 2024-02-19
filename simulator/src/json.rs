use noise::{self, NoiseFn, Perlin};
use rand_distr::{Distribution, Exp, Normal};
use chrono::{DateTime, Utc};
use rand::Rng;
use serde::Deserialize;
use supermusr_common::{Channel, DigitizerId, FrameNumber, Intensity, Time};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case", untagged)]
pub(crate) enum RandomDistribution {
    Constant(f64),
    Uniform { min: f64, max: f64 },
    Normal { mean: f64, sd: f64 },
    Exponential { lifetime: f64 },
}

impl RandomDistribution {
    pub(crate) fn sample(&self) -> f64 {
        match self {
            Self::Constant(t) => *t,
            Self::Uniform { min, max } => rand::thread_rng().gen_range(*min..*max),
            Self::Normal { mean, sd } => Normal::new(*mean, *sd).unwrap().sample(&mut rand::thread_rng()),
            Self::Exponential { lifetime } => Exp::new(1.0 / *lifetime).unwrap().sample(&mut rand::thread_rng())//-*lifetime*f64::ln(rand::random::<f64>()),
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
        peak_height: RandomDistribution,
        peak_time: RandomDistribution,
        sd: RandomDistribution,
    },
    Biexp {
        start: RandomDistribution,
        decay: RandomDistribution,
        rise: RandomDistribution,
        peak: RandomDistribution,
    },
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum NoiseSource {
    Uniform (Intensity),
    SmoothUniform {
        max: Intensity,
        factor: f64,
    }
    /*Perlin {
        #[serde(skip, default = "Perlin::new(Utc::now().timestamp_subsec_nanos())")]
        perlin: Perlin,
    }*/
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct Interval<T> {
    pub(crate) min: T,
    pub(crate) max: T,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct Transformation<T> {
    pub(crate) scale: T,
    pub(crate) translate: T,
}

impl Transformation<f64> {
  pub(crate) fn transform(&self, x : f64) -> f64 {
    x*self.scale + self.translate
  }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct Digitizer {
    pub(crate) id: DigitizerId,
    pub(crate) channels: Interval<Channel>,
}

impl Digitizer {
    pub(crate) fn get_channels(&self) -> std::ops::Range<Channel> {
        self.channels.min..self.channels.max
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum Timestamp {
    Now,
    From(DateTime<Utc>),
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct TraceMessage {
    pub(crate) time_bins: Time,
    pub(crate) digitizers: Vec<Digitizer>,
    pub(crate) frames: Vec<FrameNumber>,
    pub(crate) pulses: Vec<Pulse>,
    pub(crate) noises: Vec<NoiseSource>,
    pub(crate) num_pulses: RandomDistribution,
    pub(crate) timestamp: Timestamp,
    pub(crate) sample_rate: Option<u64>,
    pub(crate) frame_delay_us: u64,
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct Simulation {
    pub(crate) voltage_transformation: Transformation<f64>,
    pub(crate) traces: Vec<TraceMessage>,
}

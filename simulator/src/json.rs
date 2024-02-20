use std::ops::Range;

use rand_distr::{Distribution, Exp, Normal};
use chrono::{DateTime, Utc};
use rand::{Rng, SeedableRng};
use serde::Deserialize;
use supermusr_common::{Channel, DigitizerId, FrameNumber, Time};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case", untagged)]
pub(crate) enum Expression {
    Fixed(f64),
    FrameTransform(Transformation<f64>)
}

impl Expression {
    fn value(&self, frame_index: usize) -> f64 {
        match self {
            Expression::Fixed(v) => *v,
            Expression::FrameTransform(t) => t.transform(frame_index as f64),
        }
    }
}


#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case", untagged)]
pub(crate) enum RandomDistribution {
    Constant(Expression),
    Uniform { min: Expression, max: Expression },
    Normal { mean: Expression, sd: Expression },
    Exponential { lifetime: Expression },
}

impl RandomDistribution {
    pub(crate) fn sample(&self, frame_index : usize) -> f64 {
        match self {
            Self::Constant(t) => t.value(frame_index),
            Self::Uniform { min, max } => rand::rngs::StdRng::seed_from_u64(Utc::now().timestamp_subsec_nanos() as u64).gen_range(min.value(frame_index)..max.value(frame_index)),
            Self::Normal { mean, sd } => Normal::new(mean.value(frame_index), sd.value(frame_index)).unwrap().sample(&mut rand::rngs::StdRng::seed_from_u64(Utc::now().timestamp_subsec_nanos() as u64)),
            Self::Exponential { lifetime } => Exp::new(1.0 / lifetime.value(frame_index)).unwrap().sample(&mut rand::rngs::StdRng::seed_from_u64(Utc::now().timestamp_subsec_nanos() as u64))
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
    Flat {
        start: RandomDistribution,
        width: RandomDistribution,
        height: RandomDistribution,
    },
    Triangular {
        start: RandomDistribution,
        peak_time: RandomDistribution,
        width: RandomDistribution,
        height: RandomDistribution,
    },
    Gaussian {
        height: RandomDistribution,
        peak_time: RandomDistribution,
        sd: RandomDistribution,
    },
    Biexp {
        start: RandomDistribution,
        decay: RandomDistribution,
        rise: RandomDistribution,
        height: RandomDistribution,
    },
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct NoiseSource {
    bounds: Interval<Time>,
    attributes: NoiseAttributes,
    smoothing_factor: Expression,
}

impl NoiseSource {
    pub(crate) fn smooth(&self, new_value : f64, old_value : f64, frame_index : usize) -> f64 {
        new_value*(1.0 - self.smoothing_factor.value(frame_index)) + old_value*self.smoothing_factor.value(frame_index)
    }
    pub(crate) fn sample(&self, time : Time, frame_index : usize) -> f64 {
        if self.bounds.is_in(time) {
            match &self.attributes {
                NoiseAttributes::Uniform(Interval{ min, max })
                    => (max.value(frame_index) - min.value(frame_index))*rand::random::<f64>()
                        + min.value(frame_index),
                NoiseAttributes::Gaussian { mean, sd }
                    => Normal::new(
                        mean.value(frame_index),
                        sd.value(frame_index)
                    )
                    .unwrap()
                    .sample(&mut rand::rngs::StdRng::seed_from_u64(Utc::now().timestamp_subsec_nanos() as u64)),
            }
        } else {
            f64::default()
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case", tag = "type")]
pub(crate) enum NoiseAttributes {
    Uniform(Interval<Expression>),
    Gaussian { mean : Expression, sd : Expression },
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct Interval<T> {
    pub(crate) min: T,
    pub(crate) max: T,
}

impl<T : PartialOrd + Copy> Interval<T> {
    fn range(&self) -> Range<T> {
        self.min..self.max
    }
    fn is_in(&self, value : T) -> bool {
        self.range().contains(&value)
    }
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
    pub(crate) fn get_channels(&self) -> Range<Channel> {
        self.channels.range()
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

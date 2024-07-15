use super::{FloatExpression, Interval};
use chrono::Utc;
use rand::SeedableRng;
use rand_distr::{Distribution, Normal};
use serde::Deserialize;
use supermusr_common::Time;

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct NoiseSource {
    bounds: Interval<Time>,
    attributes: NoiseAttributes,
    smoothing_factor: FloatExpression,
}

impl NoiseSource {
    pub(crate) fn smooth(&self, new_value: f64, old_value: f64, frame_index: usize) -> f64 {
        new_value * (1.0 - self.smoothing_factor.value(frame_index))
            + old_value * self.smoothing_factor.value(frame_index)
    }

    pub(crate) fn sample(&self, time: Time, frame_index: usize) -> f64 {
        if self.bounds.is_in(time) {
            match &self.attributes {
                NoiseAttributes::Uniform(Interval { min, max }) => {
                    (max.value(frame_index) - min.value(frame_index)) * rand::random::<f64>()
                        + min.value(frame_index)
                }
                NoiseAttributes::Gaussian { mean, sd } => {
                    Normal::new(mean.value(frame_index), sd.value(frame_index))
                        .unwrap()
                        .sample(&mut rand::rngs::StdRng::seed_from_u64(
                            Utc::now().timestamp_subsec_nanos() as u64,
                        ))
                }
            }
        } else {
            f64::default()
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "kebab-case", tag = "noise-type")]
pub(crate) enum NoiseAttributes {
    Uniform(Interval<FloatExpression>),
    Gaussian {
        mean: FloatExpression,
        sd: FloatExpression,
    },
}

pub(crate) struct Noise<'a> {
    source: &'a NoiseSource,
    prev: f64,
}

impl<'a> Noise<'a> {
    pub(crate) fn new(source: &'a NoiseSource) -> Self {
        Self {
            source,
            prev: f64::default(),
        }
    }

    pub(crate) fn noisify(&mut self, value: f64, time: Time, frame_index: usize) -> f64 {
        self.prev = self.source.smooth(
            self.source.sample(time, frame_index),
            self.prev,
            frame_index,
        );
        value + self.prev
    }
}

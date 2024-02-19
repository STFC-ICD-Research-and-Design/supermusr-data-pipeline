use crate::json::Transformation;

use super::json;
use rand::Rng;
use supermusr_common::{Intensity, Time};

pub(crate) enum Muon {
    Flat {
        start: Time,
        stop: Time,
        amplitude: Intensity,
    },
    Triangular {
        start: Time,
        peak: Time,
        stop: Time,
        amplitude: Intensity,
    },
    Gaussian {
        mean: Time,
        sd: Time,
        peak_amplitude: Intensity,
    },
    Biexp {},
}

impl Muon {
    pub(crate) fn sample(template: &json::PulseAttributes) -> Self {
        match template {
            json::PulseAttributes::Gaussian {
                peak_height,
                peak_time,
                sd,
            } => Self::Gaussian {
                mean: peak_time.sample(),
                sd: sd.sample(),
                peak_amplitude: peak_height.sample(),
            },
        }
    }
    pub(crate) fn time(&self) -> Time {
        match self {
            Self::Flat {
                start,
                stop,
                amplitude,
            } => *start,
            Self::Triangular {
                start,
                peak,
                stop,
                amplitude,
            } => *peak,
            Self::Gaussian {
                mean,
                sd,
                peak_amplitude,
            } => *mean,
            Self::Biexp {} => Time::default(),
        }
    }
    pub(crate) fn intensity(&self) -> Intensity {
        match self {
            Self::Flat {
                start,
                stop,
                amplitude,
            } => *amplitude,
            Self::Triangular {
                start,
                peak,
                stop,
                amplitude,
            } => *amplitude,
            Self::Gaussian {
                mean,
                sd,
                peak_amplitude,
            } => *peak_amplitude,
            Self::Biexp {} => Intensity::default(),
        }
    }

    pub(crate) fn value(&self, time: Time) -> f64 {
        match self {
            &Self::Flat {
                start,
                stop,
                amplitude,
            } => {
                if start <= time && time < stop {
                    amplitude.into()
                } else {
                    f64::default()
                }
            }
            &Self::Triangular {
                start,
                peak,
                stop,
                amplitude,
            } => {
                if start <= time && time < peak {
                    amplitude as f64 * (peak - time) as f64 / (peak - start) as f64
                } else if peak <= time && time < stop {
                    amplitude as f64 * (time - peak) as f64 / (stop - peak) as f64
                } else {
                    f64::default()
                }
            }
            &Self::Gaussian {
                mean,
                sd,
                peak_amplitude,
            } => {
                peak_amplitude as f64
                    * f64::exp(-f64::powi(0.5 * (time as f64 - mean as f64) / sd as f64, 2))
            }
            &Self::Biexp {} => f64::default(),
        }
    }
}

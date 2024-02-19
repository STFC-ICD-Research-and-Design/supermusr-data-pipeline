use crate::json::Transformation;

use super::json;
use rand::Rng;
use rand_distr::Normal;
use supermusr_common::{Intensity, Time};

#[derive(Debug)]
pub(crate) enum Muon {
    Flat {
        start: f64,
        stop: f64,
        amplitude: f64,
    },
    Triangular {
        start: f64,
        peak: f64,
        stop: f64,
        amplitude: f64,
    },
    Gaussian {
        mean: f64,
        sd: f64,
        peak_amplitude: f64,
    },
    Biexp {
        start: f64,
        decay: f64,
        rise: f64,
        peak: f64,
        coef : f64,
    },
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
            json::PulseAttributes::Biexp {
                start,
                decay,
                rise,
                peak
            } => {
                let start = start.sample();
                let decay = decay.sample();
                let rise = rise.sample();
                let peak = peak.sample();
                let ratio = decay/rise;
                let coef = peak / (f64::powf(ratio, 1.0 - ratio) - f64::powf(ratio, 1.0/ratio - 1.0));
                Self::Biexp {
                    start, decay, rise, peak, coef
                }
            }
        }
    }
    pub(crate) fn time(&self) -> Time {
        *match self {
            Self::Flat { start, stop, amplitude } => start,
            Self::Triangular {  start, peak, stop, amplitude } => peak,
            Self::Gaussian { mean, sd, peak_amplitude } => mean,
            Self::Biexp { start, decay, rise, peak, coef } => peak,
        } as Time
    }
    pub(crate) fn intensity(&self) -> Intensity {
        *match self {
            Self::Flat { start, stop, amplitude, } => amplitude,
            Self::Triangular { start, peak, stop, amplitude } => amplitude,
            Self::Gaussian { mean, sd, peak_amplitude } => peak_amplitude,
            Self::Biexp { start, decay, rise, peak, coef } => peak,
        } as Intensity
    }

    pub(crate) fn get_value_at(&self, time: Time) -> f64 {
        let time = time as f64;
        match self {
            &Self::Flat {
                start, stop, amplitude,
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
            &Self::Biexp { start, decay, rise, peak: _, coef } => if time < start {
                f64::default()
            } else {
                let time = time - start;
                let value_at_time = coef*(f64::exp(-time/decay) - f64::exp(-time/rise));
                if time < 1.0 {
                    value_at_time/2.0
                } else {
                    value_at_time
                    + coef*(
                        decay*f64::exp(-time/decay)*(1.0 - f64::exp(-1.0/decay))
                      + rise*f64::exp(-time/rise)*(f64::exp(-1.0/rise) - 1.0)
                    )
                }
            },
        }
    }
}

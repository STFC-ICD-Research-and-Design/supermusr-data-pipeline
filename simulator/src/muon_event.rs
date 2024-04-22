use super::simulation_config;
use supermusr_common::{Intensity, Time};

#[derive(Debug)]
pub(crate) enum MuonEvent {
    Flat {
        start: f64,
        stop: f64,
        amplitude: f64,
    },
    Triangular {
        start: f64,
        peak_time: f64,
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
        peak_height: f64,
        coef: f64,
        peak_time: f64,
    },
}

impl MuonEvent {
    pub(crate) fn sample(
        template: &simulation_config::PulseAttributes,
        frame_index: usize,
    ) -> Self {
        match template {
            simulation_config::PulseAttributes::Flat {
                start,
                width,
                height,
            } => {
                let start = start.sample(frame_index);
                Self::Flat {
                    start,
                    stop: start + width.sample(frame_index),
                    amplitude: height.sample(frame_index),
                }
            }
            simulation_config::PulseAttributes::Triangular {
                start,
                peak_time,
                width,
                height,
            } => {
                let start = start.sample(frame_index);
                let width = width.sample(frame_index);
                Self::Triangular {
                    start,
                    peak_time: start + peak_time.sample(frame_index) * width,
                    stop: start + width,
                    amplitude: height.sample(frame_index),
                }
            }
            simulation_config::PulseAttributes::Gaussian {
                height,
                peak_time,
                sd,
            } => Self::Gaussian {
                mean: peak_time.sample(frame_index),
                sd: sd.sample(frame_index),
                peak_amplitude: height.sample(frame_index),
            },
            simulation_config::PulseAttributes::Biexp {
                start,
                decay,
                rise,
                height,
            } => {
                let start = start.sample(frame_index);
                let decay = decay.sample(frame_index);
                let rise = rise.sample(frame_index);
                let peak_height = height.sample(frame_index);
                let ratio = decay / rise;
                let coef = peak_height
                    / (f64::powf(ratio, 1.0 / ratio - 1.0) - f64::powf(ratio, 1.0 - ratio));
                let peak_time = f64::ln(f64::powf(ratio, decay * rise / (decay - rise)));
                Self::Biexp {
                    start,
                    decay,
                    rise,
                    peak_height,
                    coef,
                    peak_time,
                }
            }
        }
    }

    pub(crate) fn time(&self) -> Time {
        (match self {
            Self::Flat {
                start,
                stop: _,
                amplitude: _,
            } => *start,
            Self::Triangular {
                start: _,
                peak_time,
                stop: _,
                amplitude: _,
            } => *peak_time,
            Self::Gaussian {
                mean,
                sd: _,
                peak_amplitude: _,
            } => *mean,
            Self::Biexp {
                start,
                decay: _,
                rise: _,
                peak_height: _,
                coef: _,
                peak_time,
            } => *start + *peak_time / 2.0,
        }) as Time
    }

    pub(crate) fn intensity(&self) -> Intensity {
        *match self {
            Self::Flat {
                start: _,
                stop: _,
                amplitude,
            } => amplitude,
            Self::Triangular {
                start: _,
                peak_time: _,
                stop: _,
                amplitude,
            } => amplitude,
            Self::Gaussian {
                mean: _,
                sd: _,
                peak_amplitude,
            } => peak_amplitude,
            Self::Biexp {
                start: _,
                decay: _,
                rise: _,
                peak_height,
                coef: _,
                peak_time: _,
            } => peak_height,
        } as Intensity
    }

    pub(crate) fn get_value_at(&self, time: f64) -> f64 {
        match *self {
            Self::Flat {
                start,
                stop,
                amplitude,
            } => {
                if start <= time && time < stop {
                    amplitude
                } else {
                    f64::default()
                }
            }
            Self::Triangular {
                start,
                peak_time,
                stop,
                amplitude,
            } => {
                if start <= time && time < peak_time {
                    amplitude * (time - start) / (peak_time - start)
                } else if peak_time <= time && time < stop {
                    amplitude * (stop - time) / (stop - peak_time)
                } else {
                    f64::default()
                }
            }
            Self::Gaussian {
                mean,
                sd,
                peak_amplitude,
            } => {
                if mean - 6.0 * sd > time || time > mean + 6.0 * sd {
                    f64::default()
                } else {
                    peak_amplitude * f64::exp(-f64::powi(0.5 * (time - mean) / sd, 2))
                }
            }
            Self::Biexp {
                start,
                decay,
                rise,
                peak_height: _,
                coef,
                peak_time: _,
            } => {
                if time < start {
                    f64::default()
                } else {
                    let time = time - start;
                    coef * (f64::exp(-time / decay) - f64::exp(-time / rise))
                }
            }
        }
    }
}

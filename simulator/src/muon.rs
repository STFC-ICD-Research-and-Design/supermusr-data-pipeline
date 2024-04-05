use super::json;
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

impl Muon {
    pub(crate) fn sample(template: &json::PulseAttributes, frame_index: usize) -> Self {
        match template {
            json::PulseAttributes::Flat {
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
            json::PulseAttributes::Triangular {
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
            json::PulseAttributes::Gaussian {
                height,
                peak_time,
                sd,
            } => Self::Gaussian {
                mean: peak_time.sample(frame_index),
                sd: sd.sample(frame_index),
                peak_amplitude: height.sample(frame_index),
            },
            json::PulseAttributes::Biexp {
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
                /*
                f(t) = A(e^{-t/d} - e^{-t/r})
                f'(t) = A(e^{-t/r}/r - e^{-t/d}/d)
                b = (d/r)^{dr/(d - r)}
                peak_time: t' = ln(b)
                peak_intensity = f(t') = A(1/b^{1/d} - 1/b^{1/r})
                f''(t) = A(e^{-t/d}/d^2 - e^{-t/r}/r^2)
                t'' = 2ln(r^2/d^2)/(1/d - 1/r)
                */
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

    pub(crate) fn get_value_at(&self, time: Time) -> f64 {
        let time = time as f64;
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
                    amplitude * (peak_time - time) / (peak_time - start)
                } else if peak_time <= time && time < stop {
                    amplitude * (time - peak_time) / (stop - peak_time)
                } else {
                    f64::default()
                }
            }
            Self::Gaussian {
                mean,
                sd,
                peak_amplitude,
            } => peak_amplitude * f64::exp(-f64::powi(0.5 * (time - mean) / sd, 2)),
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

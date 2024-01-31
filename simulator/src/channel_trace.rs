use supermusr_common::{Intensity, Time};

pub(crate) enum Pulse {
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
        mean : Time,
        sd: f64,
        peak_amplitude: Intensity,
    },
    Biexp {

    }
}

impl Pulse {
    pub(crate) fn value(&self, time: Time) -> f64 {
        match self {
            &Self::Flat {start, stop, amplitude} =>
                if start <= time && time < stop {
                    amplitude as f64
                } else {
                    f64::default()
                },
            &Self::Triangular {start, peak, stop, amplitude} =>
                if start <= time && time < peak {
                    amplitude as f64*(peak - time) as f64/(peak - start) as f64
                } else if peak <= time && time < stop {
                    amplitude as f64*(time - peak) as f64/(stop - peak) as f64
                } else {
                    f64::default()
                },
            &Self::Gaussian { mean, sd, peak_amplitude } => {
                peak_amplitude as f64*f64::exp(-f64::powi(0.5*(time as f64 - mean as f64)/sd,2))
            },
            &Self::Biexp {} => f64::default(),
        }
    }
}

pub(crate) fn generate_trace(samples : Time, pulses: Vec<Pulse>, noise: Vec<f64>) -> Vec<Intensity> {
    (0..samples).map(|time|
        pulses.iter().map(|p|p.value(time)).sum::<f64>()
        + noise.iter().enumerate().map(|(_i,_n)|
            f64::default()
        ).sum::<f64>()
    )
    .map(|x : f64|x as Intensity)
    .collect()
}
use rand::Rng;
use supermusr_common::{Intensity, Time};

pub(crate) enum RandomScalar<T> {
    Constant(T),
    Uniform(T,T),
    Normal(T,f64),
}

impl RandomScalar<f64> {
    pub(crate) fn sample(&self) -> f64 {
        match self {
            RandomScalar::Constant(t) => *t,
            RandomScalar::Uniform(a, b) => rand::thread_rng().gen_range(*a..*b),
            RandomScalar::Normal(mu, _sigma) => *mu,
        }
    }
}

impl RandomScalar<Time> {
    pub(crate) fn sample(&self) -> Time {
        match self {
            RandomScalar::Constant(t) => *t,
            RandomScalar::Uniform(a, b) => rand::thread_rng().gen_range(*a..*b),
            RandomScalar::Normal(mu, _sigma) => *mu,
        }
    }
}


impl RandomScalar<Intensity> {
    pub(crate) fn sample(&self) -> Intensity {
        match self {
            RandomScalar::Constant(t) => *t,
            RandomScalar::Uniform(a, b) => rand::thread_rng().gen_range(*a..*b),
            RandomScalar::Normal(mu, _sigma) => *mu,
        }
    }
}





pub(crate) enum Pulse<TimeType, IntensityType, RealType> {
    Flat {
        start: TimeType,
        stop: TimeType,
        amplitude: IntensityType,
    },
    Triangular {
        start: TimeType,
        peak: TimeType,
        stop: TimeType,
        amplitude: IntensityType,
    },
    Gaussian {
        mean : TimeType,
        sd: RealType,
        peak_amplitude: IntensityType,
    },
    Biexp {

    }
}

pub(crate) type RandomPulse = Pulse<RandomScalar<Time>,RandomScalar<Intensity>,RandomScalar<f64>>;
impl Pulse<RandomScalar<Time>,RandomScalar<Intensity>,RandomScalar<f64>> {
    pub(crate) fn sample(&self) -> Pulse<Time,Intensity,f64> {
        match self {
            Pulse::Flat { start, stop, amplitude }
                => Pulse::Flat { start: start.sample(), stop: stop.sample(), amplitude: amplitude.sample() },
            Pulse::Triangular { start, peak, stop, amplitude }
                => Pulse::Triangular { start: start.sample(), peak: peak.sample(), stop: stop.sample(), amplitude: amplitude.sample() },
            Pulse::Gaussian { mean, sd, peak_amplitude }
                => Pulse::Gaussian { mean: mean.sample(), sd: sd.sample(), peak_amplitude: peak_amplitude.sample() },
            Pulse::Biexp {  } => Pulse::Biexp {  },
        }
    }
}

pub(crate) type FixedPulse = Pulse<Time,Intensity,f64>;
impl FixedPulse {
    pub(crate) fn value(&self, time: Time) -> f64 {
        match self {
            &Self::Flat {start, stop, amplitude} =>
                if start <= time && time < stop {
                    amplitude.into()
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

pub(crate) fn generate_trace(samples : Time, pulses: Vec<FixedPulse>, noise: Vec<f64>) -> Vec<Intensity> {
    (0..samples).map(|time|
        pulses.iter().map(|p|p.value(time)).sum::<f64>()
        + noise.iter().enumerate().map(|(_i,_n)|
            f64::default()
        ).sum::<f64>()
    )
    .map(|x : f64|x as Intensity)
    .collect()
}
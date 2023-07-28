use common::{Channel, Intensity, DigitizerId, FrameNumber};
use itertools::Itertools;
use rand::{Rng, random};

pub struct RandomInterval(pub f64,pub f64);

impl RandomInterval {
    fn sample(&self) -> f64 {
        self.0 + (self.1 - self.0)*random::<f64>()
    }
}

impl From<(f64,f64)> for RandomInterval {
    fn from(value: (f64,f64)) -> Self {
        RandomInterval(value.0,value.1)
    }
}

pub struct PulseDistribution {
    pub std_dev : RandomInterval,
    pub decay_factor : RandomInterval,
    pub time_wobble : RandomInterval,
    pub value_wobble: RandomInterval,
    pub peak : RandomInterval,
}
impl PulseDistribution {
    fn variance_sample(&self) -> f64 {
        f64::powi(self.std_dev.sample(),2)
    }
}

pub struct Pulse {
    peak_at: f64,
    peak_value : f64,
    variance: f64,
    time_wobble: f64,
    value_wobble: f64,
    decay_factor: f64,
}

impl Pulse {
    fn new(
        trace_length : usize,
        distribution : &PulseDistribution,
    ) -> Self {
        Pulse {
            peak_at: random::<f64>() * trace_length as f64,
            peak_value : distribution.peak.sample(),
            variance: distribution.variance_sample(),
            time_wobble : distribution.time_wobble.sample(),
            value_wobble : distribution.value_wobble.sample(),
            decay_factor : distribution.decay_factor.sample(),
        }
    }
    pub(crate) fn get_value_at(&self, time : usize) -> Intensity {
        let t : f64 = time as f64 - self.peak_at + self.time_wobble*(2.*random::<f64>() - 1.);
        let val = self.peak_value
            *f64::exp(-0.5*t*t/self.variance)
            *(1. + self.value_wobble*(2.*random::<f64>() - 1.));
        val as Intensity
    }
}

pub fn create_pulses(
    trace_length : usize,
    min_pulses : usize,
    max_pulses : usize,
    distribution : &PulseDistribution,
) -> Vec<Pulse> {
    let mut rng = rand::thread_rng();
    (0..rng.gen_range(min_pulses..=max_pulses))
        .map(|_|Pulse::new(trace_length, distribution))
        .collect_vec()
}



pub fn create_trace(
    trace_length : usize,
    pulses : Vec<Pulse>,
    min_voltage : Intensity,
    base_voltage : Intensity,
    max_voltage : Intensity,
    voltage_noise : Intensity,
) -> Vec<Intensity> {
(0..trace_length).into_iter()
    .map(|t| generate_intensity(&pulses, t, min_voltage, base_voltage, max_voltage, voltage_noise))
    .collect()
}

fn generate_intensity(pulses: &Vec<Pulse>, time: usize,
    min_voltage : Intensity,
    base_voltage : Intensity,
    max_voltage : Intensity,
    voltage_noise : Intensity
) -> Intensity {
    Intensity::clamp(
        base_voltage
        + pulses
            .iter()
            .map(|pulse|pulse.get_value_at(time) as Intensity)
            .sum::<Intensity>()
        + (voltage_noise as f64*(2.*random::<f64>() - 1.)) as Intensity,
    min_voltage,max_voltage)
}
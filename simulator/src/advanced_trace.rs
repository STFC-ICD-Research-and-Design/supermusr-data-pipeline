use rand::{self, Rng};
use clap::{Parser, Subcommand};
use supermusr_common::{Channel, Intensity, Time};

use crate::channel_trace::{FixedPulse, RandomPulse};

#[derive(Clone, Subcommand)]
pub(crate) enum TraceMode {
    Basic,
    Advanced(AdvancedTrace),
}

#[derive(Clone, Parser)]
pub(crate) struct AdvancedTrace {
    /// Number of channels to generate
    #[clap(long, default_value = "8")]
    pub(crate) num_channels: Channel,
    
    /// Number of pulses to generate
    #[clap(long, default_value = "100")]
    pub(crate) num_pulses: usize,

    /// Amount of noise to include
    #[clap(long, default_value = "0")]
    pub(crate) noise: usize,
}


pub(crate) fn generate_pulses(num_time_bins: Time, num_pulses: usize) -> Vec<FixedPulse> {
    let mut rng = rand::thread_rng();
    (0..num_pulses).map(|_| {
        let loc = rng.gen_range(0..num_time_bins);
        let width = rng.gen_range(0..100) as f64/10.0 + 0.5;
        let amplitude = rng.gen_range(20..80);
        match rng.gen_range(0..2) {
            0 => FixedPulse::Gaussian {mean: loc, sd: width, peak_amplitude: amplitude },
            1 => {
                let start = loc - width as Time/2;
                let stop = loc + width as Time/2;
                FixedPulse::Flat {start, stop, amplitude }
            },
            _ => {
                let start = loc - width as Time/2;
                let stop = loc + width as Time/2;
                let peak = {
                    let coef = rng.gen_range(0..100) as f64/100.0;
                    (start as f64*coef + stop as f64*(1.0 - coef)) as Time
                };
                FixedPulse::Triangular {start, peak, stop, amplitude }
            }
        }
    }).collect()
}
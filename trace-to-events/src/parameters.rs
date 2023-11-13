use std::str::FromStr;

use crate::pulse_detection::{detectors::threshold_detector::ThresholdDuration, Real};
use anyhow::{anyhow, Error};
use clap::{Parser, Subcommand};

#[derive(Default, Debug, Clone)]
pub struct ThresholdDurationWrapper(pub(crate) ThresholdDuration);

impl FromStr for ThresholdDurationWrapper {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let vals: Vec<_> = s.split(',').collect();
        if vals.len() == 3 {
            Ok(ThresholdDurationWrapper(ThresholdDuration {
                threshold: Real::from_str(vals[0])?,
                duration: i32::from_str(vals[1])?,
                cool_off: i32::from_str(vals[2])?,
            }))
        } else {
            Err(anyhow!(
                "Incorrect number of parameters in threshold, expected pattern '*,*,*', got '{s}'"
            ))
        }
    }
}

#[derive(Default, Debug, Clone, Parser)]
pub struct ConstantPhaseDiscriminatorParameters {
    #[clap(
        long,
        help = "constant phase threshold for detecting muon events, use format (threshold,duration,cool_down). See README.md."
    )]
    pub threshold_trigger: ThresholdDurationWrapper,
}

pub struct SaveOptions<'a> {
    pub save_path: &'a str,
    pub file_name: &'a str,
}

#[derive(Default, Debug, Clone, Parser)]
pub struct AdvancedMuonDetectorParameters {
    #[clap(
        long,
        help = "Differential threshold for detecting muon onset (threshold,duration,cool_down). See README.md."
    )]
    pub muon_onset: ThresholdDurationWrapper,
    #[clap(
        long,
        help = "Differential threshold for detecting muon peak (threshold,duration,cool_down). See README.md."
    )]
    pub muon_fall: ThresholdDurationWrapper,
    #[clap(
        long,
        help = "Differential threshold for detecting muon termination (threshold,duration,cool_down). See README.md."
    )]
    pub muon_termination: ThresholdDurationWrapper,
    #[clap(
        long,
        help = "Size of initial portion of the trace to use for determining the baseline. Initial portion should be event free."
    )]
    pub baseline_length: Option<usize>,
    #[clap(
        long,
        help = "Size of the moving average window to use for the lopass filter."
    )]
    pub smoothing_window_size: Option<usize>,
    #[clap(
        long,
        help = "Optional parameter which (if set) filters out events whose peak is greater than the given value."
    )]
    pub max_amplitude: Option<Real>,
    #[clap(
        long,
        help = "Optional parameter which (if set) filters out events whose peak is less than the given value."
    )]
    pub min_amplitude: Option<Real>,
}

#[derive(Subcommand, Debug)]
pub enum Mode {
    #[clap(
        about = "Detects events using a constant phase discriminator. Events consist only of a time value."
    )]
    ConstantPhaseDiscriminator(ConstantPhaseDiscriminatorParameters),
    #[clap(
        about = "Detects events using differential discriminators. Event lists consist of time and voltage values."
    )]
    AdvancedMuonDetector(AdvancedMuonDetectorParameters),
}

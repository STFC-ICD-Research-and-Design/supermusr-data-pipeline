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
pub struct SimpleParameters {
    pub threshold_trigger: ThresholdDurationWrapper,
}

pub struct SaveOptions<'a> {
    pub save_path: &'a str,
    pub file_name: &'a str,
}

#[derive(Default, Debug, Clone, Parser)]
pub struct BasicParameters {
    pub gate_size: Real,
    pub min_voltage: Real,
    pub smoothing_window_size: usize,
    pub baseline_length: usize,
    pub max_amplitude: Option<Real>,
    pub min_amplitude: Option<Real>,
    pub muon_onset: ThresholdDurationWrapper,
    pub muon_fall: ThresholdDurationWrapper,
    pub muon_termination: ThresholdDurationWrapper,
}

#[derive(Subcommand, Debug)]
pub enum Mode {
    #[clap(
        about = "Detects events using a constant phase discriminator. Events consist only of a time value."
    )]
    Simple(SimpleParameters),
    #[clap(
        about = "Detects events using differential discriminators. Event lists consist of time and voltage values."
    )]
    Basic(BasicParameters),
}

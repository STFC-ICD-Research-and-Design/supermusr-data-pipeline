use crate::pulse_detection::{detectors::threshold_detector::ThresholdDuration, Real};
use anyhow::{anyhow, Error};
use clap::{Parser, Subcommand};
use std::str::FromStr;

#[derive(Default, Debug, Clone)]
pub(crate) struct ThresholdDurationWrapper(pub(crate) ThresholdDuration);

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
pub(crate) struct ConstantPhaseDiscriminatorParameters {
    /// Constant phase threshold for detecting muon events, use format "threshold,duration,cool_down". See README.md.
    #[clap(long)]
    pub(crate) threshold_trigger: ThresholdDurationWrapper,
}

#[derive(Default, Debug, Clone, Parser)]
pub(crate) struct AdvancedMuonDetectorParameters {
    /// Differential threshold for detecting muon onset "threshold,duration,cool_down". See README.md.
    #[clap(long)]
    pub(crate) muon_onset: ThresholdDurationWrapper,

    /// Differential threshold for detecting muon peak "threshold,duration,cool_down". See README.md.
    #[clap(long)]
    pub(crate) muon_fall: ThresholdDurationWrapper,

    /// Differential threshold for detecting muon termination "threshold,duration,cool_down". See README.md.
    #[clap(long)]
    pub(crate) muon_termination: ThresholdDurationWrapper,

    /// Size of initial portion of the trace to use for determining the baseline. Initial portion should be event free.
    #[clap(long)]
    pub(crate) baseline_length: Option<usize>,

    /// Size of the moving average window to use for the lopass filter.
    #[clap(long)]
    pub(crate) smoothing_window_size: Option<usize>,

    /// Optional parameter which (if set) filters out events whose peak is greater than the given value.
    #[clap(long)]
    pub(crate) max_amplitude: Option<Real>,

    /// Optional parameter which (if set) filters out events whose peak is less than the given value.
    #[clap(long)]
    pub(crate) min_amplitude: Option<Real>,
}

#[derive(Subcommand, Debug)]
pub(crate) enum Mode {
    /// Detects events using a constant phase discriminator. Events consist only of a time value.
    ConstantPhaseDiscriminator(ConstantPhaseDiscriminatorParameters),
    /// Detects events using differential discriminators. Event lists consist of time and voltage values.
    AdvancedMuonDetector(AdvancedMuonDetectorParameters),
}

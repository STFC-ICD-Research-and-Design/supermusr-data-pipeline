use crate::traces::simulation_config::{
    Interval, NoiseSource, PulseAttributes, PulseSource, RandomDistribution
};
use serde::Deserialize;
use supermusr_common::{Channel, DigitizerId, FrameNumber, Time};


#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case", tag = "run-command")]
pub(crate) struct RunStart {
    pub(crate) name: String,
    pub(crate) instrument: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case", tag = "run-command")]
pub(crate) struct RunStop {
    pub(crate) name: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case", tag = "run-command")]
pub(crate) struct RunLogData {
    pub(crate) name: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case", tag = "run-command")]
pub(crate) struct SampleEnvLog {
    name: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case", tag = "run-command")]
pub(crate) struct Alarm {
    pub(crate) name: String,
}
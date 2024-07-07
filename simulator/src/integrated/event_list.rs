use serde::Deserialize;

use super::{muon::{MuonTemplate, MuonEvent}, noise::NoiseSource, RandomDistribution};


#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct EventListTemplate {
    pub(crate) pulses: Vec<MuonTemplate>,
    pub(crate) noises: Vec<NoiseSource>,
    pub(crate) num_pulses: RandomDistribution,
}

pub(crate) struct EventList<'a> {
    pub(crate) pulses: Vec<MuonEvent>,
    pub(crate) noises: &'a [NoiseSource],
}

use serde::Deserialize;

use crate::integrated::{
    simulation_elements::{
        muon::{MuonEvent, MuonTemplate},
        noise::NoiseSource,
    },
    RandomDistribution,
};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct EventListTemplate {
    pub(crate) pulses: Vec<MuonTemplate>,
    pub(crate) noises: Vec<NoiseSource>,
    pub(crate) num_pulses: RandomDistribution,
}

#[derive(Default)]
pub(crate) struct EventList<'a> {
    pub(crate) pulses: Vec<MuonEvent>,
    pub(crate) noises: &'a [NoiseSource],
}

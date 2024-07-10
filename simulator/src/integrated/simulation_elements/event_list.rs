use serde::Deserialize;
use tracing::error;

use crate::integrated::{
    simulation_elements::{
        muon::{MuonEvent, MuonTemplate},
        noise::NoiseSource,
    },
    RandomDistribution
};

use super::muon::MuonAttributes;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct EventListTemplate {
    pub(crate) pulses: Vec<MuonTemplate>,
    pub(crate) noises: Vec<NoiseSource>,
    pub(crate) num_pulses: RandomDistribution,
}

impl EventListTemplate {
    pub(crate) fn validate(&self, pulse_attributes: &[MuonAttributes]) -> bool {
        for pulse in &self.pulses {
            if pulse.validate(pulse_attributes.len()) {
                error!("Pulse index too large");
                return false
            }
        }
        true
    }
}

#[derive(Default)]
pub(crate) struct EventList<'a> {
    pub(crate) pulses: Vec<MuonEvent>,
    pub(crate) noises: &'a [NoiseSource],
}

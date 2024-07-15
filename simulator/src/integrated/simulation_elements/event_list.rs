use super::IntRandomDistribution;
use crate::integrated::simulation_elements::{
    muon::{MuonEvent, MuonTemplate},
    noise::NoiseSource,
};
use serde::Deserialize;
use supermusr_common::{
    spanned::{SpanOnce, SpanWrapper, Spanned},
    Intensity,
};
use tracing::error;

pub(crate) type Trace = SpanWrapper<Vec<Intensity>>;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct EventListTemplate {
    pub(crate) pulses: Vec<MuonTemplate>,
    pub(crate) noises: Vec<NoiseSource>,
    pub(crate) num_pulses: IntRandomDistribution,
}

impl EventListTemplate {
    pub(crate) fn validate(&self, num_pulse_attributes: usize) -> bool {
        for pulse in &self.pulses {
            if !pulse.validate(num_pulse_attributes) {
                error!("Pulse index too large");
                return false;
            }
        }
        true
    }
}

#[derive(Default)]
pub(crate) struct EventList<'a> {
    pub(crate) span: SpanOnce,
    pub(crate) pulses: Vec<MuonEvent>,
    pub(crate) noises: &'a [NoiseSource],
}

impl Spanned for EventList<'_> {
    fn span(&self) -> &SpanOnce {
        &self.span
    }
}

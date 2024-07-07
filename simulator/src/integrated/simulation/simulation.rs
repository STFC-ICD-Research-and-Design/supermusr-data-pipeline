use chrono::Utc;
use rand::SeedableRng;
use rand_distr::{Distribution, WeightedIndex};
use serde::Deserialize;
use supermusr_common::{FrameNumber, Intensity, Time};

use crate::integrated::{
    schedule::Action,
    simulation_elements::{
        event_list::{EventList, EventListTemplate},
        muon::{MuonAttributes, MuonEvent},
        noise::Noise,
    },
    Transformation,
};

use super::active_muons::ActiveMuons;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct Simulation {
    pub(crate) voltage_transformation: Transformation<f64>,
    pub(crate) time_bins: Time,
    pub(crate) sample_rate: u64,
    pub(crate) event_lists: Vec<EventListTemplate>,
    pub(crate) pulses: Vec<MuonAttributes>,
    pub(crate) schedule: Vec<Action>,
}

impl Simulation {
    pub(crate) fn validate(&self) -> bool {
        for event_list in &self.event_lists {
            for pulse in &event_list.pulses {
                if pulse.index >= self.pulses.len() {
                    return false;
                }
            }
        }
        true
    }

    pub(crate) fn get_random_pulse_attributes(
        &self,
        source: &EventListTemplate,
        distr: &WeightedIndex<f64>,
    ) -> &MuonAttributes {
        //  get a random index for the pulse
        let index = distr.sample(&mut rand::rngs::StdRng::seed_from_u64(
            Utc::now().timestamp_subsec_nanos() as u64,
        ));
        // Return a pointer to either a local or global pulse
        self.pulses
            .get(source.pulses.get(index).unwrap().index)
            .unwrap()
    }

    pub(crate) fn generate_event_list(&self, index: usize, frame_number: FrameNumber) -> EventList {
        let source = self.event_lists.get(index).unwrap();
        let distr = WeightedIndex::new(source.pulses.iter().map(|p| p.weight)).unwrap();
        EventList {
            pulses: {
                // Creates a unique template for each channel
                let mut pulses = (0..source.num_pulses.sample(frame_number as usize) as usize)
                    .map(|_| {
                        MuonEvent::sample(
                            self.get_random_pulse_attributes(source, &distr),
                            frame_number as usize,
                        )
                    })
                    .collect::<Vec<_>>();
                pulses.sort_by_key(|a| a.get_start());
                pulses
            },
            noises: &source.noises,
        }
    }

    pub(crate) fn generate_trace(
        &self,
        event_list: &EventList,
        frame_number: FrameNumber,
    ) -> Vec<Intensity> {
        let sample_time = 1_000_000_000.0 / self.sample_rate as f64;

        let mut noise = event_list.noises.iter().map(Noise::new).collect::<Vec<_>>();
        let mut active_muons = ActiveMuons::new(&event_list.pulses);
        (0..self.time_bins)
            .map(|time| {
                //  Remove any expired muons
                active_muons.drop_spent_muons(time);
                //  Append any new muons
                active_muons.push_new_muons(time);

                //  Sum the signal of the currenty active muons
                let signal = active_muons
                    .iter()
                    .map(|p| p.get_value_at(time as f64 * sample_time))
                    .sum::<f64>();
                noise.iter_mut().fold(signal, |signal, n| {
                    n.noisify(signal, time, frame_number as usize)
                })
            })
            .map(|x: f64| self.voltage_transformation.transform(x) as Intensity)
            .collect()
    }
}

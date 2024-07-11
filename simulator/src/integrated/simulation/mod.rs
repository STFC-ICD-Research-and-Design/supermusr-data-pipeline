use rayon as _;
pub(crate) mod active_muons;
pub(crate) mod digitiser_config;

use chrono::Utc;
use rand::SeedableRng;
use rand_distr::{Distribution, WeightedIndex};
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use serde::Deserialize;
use supermusr_common::{spanned::SpanWrapper, FrameNumber, Intensity, Time};
use tracing::{info_span, instrument};

use crate::integrated::{
    simulation_elements::{
        event_list::{EventList, EventListTemplate},
        muon::{MuonAttributes, MuonEvent},
        noise::Noise,
    },
    simulation_engine::actions::Action,
    Transformation,
};

use active_muons::ActiveMuons;
use digitiser_config::DigitiserConfig;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct Simulation {
    pub(crate) voltage_transformation: Transformation<f64>,
    pub(crate) time_bins: Time,
    pub(crate) sample_rate: u64,
    pub(crate) digitiser_config: DigitiserConfig,
    pub(crate) event_lists: Vec<EventListTemplate>, //  Need to validate
    pub(crate) pulses: Vec<MuonAttributes>,
    pub(crate) schedule: Vec<Action>,   //  Need to validate
}

impl Simulation {
    pub(crate) fn validate(&self) -> bool {
        for event_list in &self.event_lists {
            if !event_list.validate(self.pulses.len()) {
                return false
            }
        }
        for action in &self.schedule {
            if !action.validate(self.digitiser_config.get_num_digitisers(), self.digitiser_config.get_num_channels()) {
                return false
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

    pub(crate) fn generate_event_lists(
        &self,
        index: usize,
        frame_number: FrameNumber,
        repeat: usize,
    ) -> Vec<EventList> {
        let source = self.event_lists.get(index).unwrap();

        (0..repeat)
            .map(SpanWrapper::<usize>::new_with_current)
            .collect::<Vec<_>>()
            .into_par_iter()
            .map(|_| {
                let distr = WeightedIndex::new(source.pulses.iter().map(|p| p.weight)).unwrap();
                EventList {
                    pulses: {
                        // Creates a unique template for each channel
                        let mut pulses = (0..source.num_pulses.sample(frame_number as usize)
                            as usize)
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
            })
            .collect()
    }

    #[instrument(skip_all, target = "otel")]
    pub(crate) fn generate_traces(
        &self,
        event_lists: &[&EventList],
        frame_number: FrameNumber,
    ) -> Vec<Vec<Intensity>> {
        let sample_time = 1_000_000_000.0 / self.sample_rate as f64;

        event_lists
            .iter()
            .map(SpanWrapper::<_>::new_with_current)
            .collect::<Vec<_>>()
            .into_par_iter()
            .map(|event_list| {
                info_span!(target: "otel", "Generate New Trace").in_scope(|| {
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
                })
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const JSON_INPUT_1: &str = r#"
    {
        "voltage-transformation": {"scale": 1, "translate": 0 },
        "time-bins": 30000,
        "sample-rate": 1000000000,
        "digitiser-config": {
            "auto-digitisers": {
                "num-digitisers": 32,
                "num-channels-per-digitiser": 8
            }
        },
        "pulses": [{
                        "pulse-type": "biexp",
                        "height": { "random-type": "uniform", "min": { "fixed-value": 30 }, "max": { "fixed-value": 70 } },
                        "start":  { "random-type": "exponential", "lifetime": { "fixed-value": 2200 } },
                        "rise":   { "random-type": "uniform", "min": { "fixed-value": 20 }, "max": { "fixed-value": 30 } },
                        "decay":  { "random-type": "uniform", "min": { "fixed-value": 5 }, "max": { "fixed-value": 10 } }
                    },
                    {
                        "pulse-type": "flat",
                        "start":  { "random-type": "exponential", "lifetime": { "fixed-value": 2200 } },
                        "width":  { "random-type": "uniform", "min": { "fixed-value": 20 }, "max": { "fixed-value": 50 } },
                        "height": { "random-type": "uniform", "min": { "fixed-value": 30 }, "max": { "fixed-value": 70 } }
                    },
                    {
                        "pulse-type": "triangular",
                        "start":     { "random-type": "exponential", "lifetime": { "fixed-value": 2200 } },
                        "width":     { "random-type": "uniform", "min": { "fixed-value": 20 }, "max": { "fixed-value": 50 } },
                        "peak_time": { "random-type": "uniform", "min": { "fixed-value": 0.25 }, "max": { "fixed-value": 0.75 } },
                        "height":    { "random-type": "uniform", "min": { "fixed-value": 30 }, "max": { "fixed-value": 70 } }
                    }],
        "traces": [
            {
                "sample-rate": 100000000,
                "pulses": [
                    {"weight": 1, "attributes": {"create-from-index": 0}},
                    {"weight": 1, "attributes": {"create-from-index": 1}},
                    {"weight": 1, "attributes": {"create-from-index": 2}}
                ],
                "noises": [
                    {
                        "attributes": { "noise-type" : "gaussian", "mean" : { "fixed-value": 0 }, "sd" : { "fixed-value": 20 } },
                        "smoothing-factor" : { "fixed-value": 0.975 },
                        "bounds" : { "min": 0, "max": 30000 }
                    },
                    {
                        "attributes": { "noise-type" : "gaussian", "mean" : { "fixed-value": 0 }, "sd" : { "frame-transform": { "scale": 50, "translate": 50 } } },
                        "smoothing-factor" : { "fixed-value": 0.995 },
                        "bounds" : { "min": 0, "max": 30000 }
                    }
                ],
                "num-pulses": { "random-type": "constant", "value": { "fixed-value": 500 } },
                "time-bins": 30000,
                "timestamp": "now",
                "frame-delay-us": 20000
            }
        ],
        "schedule" [
            { "action": { "run-command": "run-start", "name": "MyRun", "instrument": "MuSR" } },
            { "action": { "wait_ms": 100 } },
            { "loop" : {
                    
                }
            }
        ]
    }
    "#;
    #[test]
    fn test1() {
        let simulation: Simulation = serde_json::from_str(JSON_INPUT_1).unwrap();

        assert!(simulation.validate());
        assert_eq!(simulation.pulses.len(), 0);
        assert_eq!(simulation.voltage_transformation.scale, 1.0);
        assert_eq!(simulation.voltage_transformation.translate, 0.0);
    }
}

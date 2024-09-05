use crate::integrated::{
    simulation_elements::{
        event_list::{EventList, EventListTemplate, Trace},
        pulses::PulseTemplate,
        DigitiserConfig, Transformation,
    },
    simulation_engine::actions::Action,
};
use chrono::Utc;
use rand::SeedableRng;
use rand_distr::{Distribution, WeightedIndex};
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use serde::Deserialize;
use supermusr_common::{
    spanned::{SpanWrapper, Spanned},
    FrameNumber, Time,
};
use tracing::instrument;

///
/// This struct is created from the configuration JSON file.
///
#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct Simulation {
    // Is applied to all voltages when traces are created
    pub(crate) voltage_transformation: Transformation<f64>,
    //  The length of each trace
    pub(crate) time_bins: Time,
    //  Number of samples (time_bins) per second
    pub(crate) sample_rate: u64,
    pub(crate) digitiser_config: DigitiserConfig,
    pub(crate) event_lists: Vec<EventListTemplate>,
    pub(crate) pulses: Vec<PulseTemplate>,
    pub(crate) schedule: Vec<Action>,
}

impl Simulation {
    /// Checks that all Pulse, Digitiser and EventList indices are valid
    pub(crate) fn validate(&self) -> bool {
        for event_list in &self.event_lists {
            if !event_list.validate(self.pulses.len()) {
                return false;
            }
        }
        for action in &self.schedule {
            if !action.validate(
                self.digitiser_config.get_num_digitisers(),
                self.digitiser_config.get_num_channels(),
            ) {
                return false;
            }
        }
        true
    }

    pub(crate) fn get_random_pulse_template(
        &self,
        source: &EventListTemplate,
        distr: &WeightedIndex<f64>,
    ) -> &PulseTemplate {
        //  get a random index for the pulse
        let index = distr.sample(&mut rand::rngs::StdRng::seed_from_u64(
            Utc::now().timestamp_subsec_nanos() as u64,
        ));
        // Return a pointer to either a local or global pulse
        self.pulses
            .get(source.pulses.get(index).unwrap().pulse_index)
            .unwrap() //  This will never panic as long as validate is called
    }

    #[instrument(skip_all, target = "otel")]
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
            .map(|span_wrapper| {
                span_wrapper
                    .span()
                    .get()
                    .expect("Span is initialised")
                    .in_scope(|| EventList::new(self, frame_number, source))
            })
            .collect()
    }

    #[instrument(skip_all, target = "otel")]
    pub(crate) fn generate_traces<'a>(
        &'a self,
        event_lists: &'a [EventList],
        frame_number: FrameNumber,
    ) -> Vec<Trace> {
        event_lists
            .iter()
            .map(SpanWrapper::<_>::new_with_current)
            .collect::<Vec<_>>()
            .into_par_iter()
            .map(|event_list| {
                let current_span = event_list.span().get().unwrap(); //  This is the span of this method
                let event_list: &EventList = *event_list; //  This is the spanned event list
                current_span.in_scope(|| Trace::new(self, frame_number, event_list))
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
                "num-digitisers": { "int" : 32 },
                "num-channels-per-digitiser": { "int" : 8 }
            }
        },
        "pulses": [{
                        "pulse-type": "biexp",
                        "height": { "random-type": "uniform", "min": { "float": 30 }, "max": { "float": 70 } },
                        "start":  { "random-type": "exponential", "lifetime": { "float": 2200 } },
                        "rise":   { "random-type": "uniform", "min": { "float": 20 }, "max": { "float": 30 } },
                        "decay":  { "random-type": "uniform", "min": { "float": 5 }, "max": { "float": 10 } }
                    },
                    {
                        "pulse-type": "flat",
                        "start":  { "random-type": "exponential", "lifetime": { "float": 2200 } },
                        "width":  { "random-type": "uniform", "min": { "float": 20 }, "max": { "float": 50 } },
                        "height": { "random-type": "uniform", "min": { "float": 30 }, "max": { "float": 70 } }
                    },
                    {
                        "pulse-type": "triangular",
                        "start":     { "random-type": "exponential", "lifetime": { "float": 2200 } },
                        "width":     { "random-type": "uniform", "min": { "float": 20 }, "max": { "float": 50 } },
                        "peak_time": { "random-type": "uniform", "min": { "float": 0.25 }, "max": { "float": 0.75 } },
                        "height":    { "random-type": "uniform", "min": { "float": 30 }, "max": { "float": 70 } }
                    }],
        "event-lists": [
            {
                "pulses": [
                    {"weight": 1, "pulse-index": 0},
                    {"weight": 1, "pulse-index": 1},
                    {"weight": 1, "pulse-index": 2}
                ],
                "noises": [
                    {
                        "attributes": { "noise-type" : "gaussian", "mean" : { "float": 0 }, "sd" : { "float": 20 } },
                        "smoothing-factor" : { "float": 0.975 },
                        "bounds" : { "min": 0, "max": 30000 }
                    },
                    {
                        "attributes": { "noise-type" : "gaussian", "mean" : { "float": 0 }, "sd" : { "float-func": { "scale": 50, "translate": 50 } } },
                        "smoothing-factor" : { "float": 0.995 },
                        "bounds" : { "min": 0, "max": 30000 }
                    }
                ],
                "num-pulses": { "random-type": "constant", "value": { "int": 500 } }
            }
        ],
        "schedule": [
            { "send-run-start": { "name": { "text": "MyRun" }, "instrument": { "text": "MuSR" } } },
            { "wait-ms": 100 },
            { "frame-loop": {
                    "start": { "int": 0 },
                    "end": { "int": 99 },
                    "schedule": [
                    ]
                }
            }
        ]
    }
    "#;
    #[test]
    fn test1() {
        let simulation: Simulation = serde_json::from_str(JSON_INPUT_1).unwrap();

        assert!(simulation.validate());
        assert_eq!(simulation.pulses.len(), 3);
        assert_eq!(simulation.voltage_transformation.scale, 1.0);
        assert_eq!(simulation.voltage_transformation.translate, 0.0);
    }
}

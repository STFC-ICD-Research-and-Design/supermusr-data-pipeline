pub(crate) mod run_messages;
pub(crate) mod schedule;
pub(crate) mod engine;
pub(crate) mod event_list;
pub(crate) mod muon;
pub(crate) mod noise;

use std::ops::RangeInclusive;

use chrono::Utc;
use event_list::EventListTemplate;
use muon::{MuonAttributes, MuonTemplate};
use rand::{Rng, SeedableRng};
use rand_distr::{Distribution, Exp, Normal};
use run_messages::{Alarm, RunLogData, RunStart, RunStop, SampleEnvLog};
use schedule::Action;
use serde::Deserialize;
use supermusr_common::{Channel, DigitizerId, FrameNumber, Intensity, Time};
use supermusr_streaming_types::{frame_metadata_v2_generated::FrameMetadataV2, FrameMetadata};




#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum Expression {
    FixedValue(f64),
    FrameTransform(Transformation<f64>),
}

impl Expression {
    fn value(&self, frame_index: usize) -> f64 {
        match self {
            Expression::FixedValue(v) => *v,
            Expression::FrameTransform(frame_function) => {
                frame_function.transform(frame_index as f64)
            }
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "kebab-case", tag = "random-type")]
pub(crate) enum RandomDistribution {
    Constant { value: Expression },
    Uniform { min: Expression, max: Expression },
    Normal { mean: Expression, sd: Expression },
    Exponential { lifetime: Expression },
}

impl RandomDistribution {
    pub(crate) fn sample(&self, frame_index: usize) -> f64 {
        match self {
            Self::Constant { value } => value.value(frame_index),
            Self::Uniform { min, max } => {
                rand::rngs::StdRng::seed_from_u64(Utc::now().timestamp_subsec_nanos() as u64)
                    .gen_range(min.value(frame_index)..max.value(frame_index))
            }
            Self::Normal { mean, sd } => {
                Normal::new(mean.value(frame_index), sd.value(frame_index))
                    .unwrap()
                    .sample(&mut rand::rngs::StdRng::seed_from_u64(
                        Utc::now().timestamp_subsec_nanos() as u64,
                    ))
            }
            Self::Exponential { lifetime } => Exp::new(1.0 / lifetime.value(frame_index))
                .unwrap()
                .sample(&mut rand::rngs::StdRng::seed_from_u64(
                    Utc::now().timestamp_subsec_nanos() as u64,
                )),
        }
    }
}


#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct Interval<T> where T : Clone {
    pub(crate) min: T,
    pub(crate) max: T,
}

impl<T: PartialOrd + Copy> Interval<T> {
    fn range_inclusive(&self) -> RangeInclusive<T> {
        self.min..=self.max
    }

    fn is_in(&self, value: T) -> bool {
        self.range_inclusive().contains(&value)
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct Transformation<T> {
    pub(crate) scale: T,
    pub(crate) translate: T,
}

impl Transformation<f64> {
    pub(crate) fn transform(&self, x: f64) -> f64 {
        x * self.scale + self.translate
    }
}











pub(crate) type TraceSourceId = usize;


#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum FrameSource {
    AggregatedFrame {
        num_channels: usize,
        source: TraceSourceId,
    },
    AutoDigitisers {
        num_digitiser: usize,
        num_channels_per_digitiser: usize,
        source: TraceSourceId,
    },
    DigitiserList(Vec<Digitiser>),
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct Digitiser {
    pub(crate) id: DigitizerId,
    pub(crate) channels: Interval<Channel>,
    pub(crate) source: TraceSourceId,
}

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
}

const JSON_INPUT_1: &str = r#"
{
    "voltage-transformation": {"scale": 1, "translate": 0 },
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

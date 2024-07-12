pub(crate) mod build_messages;
pub(crate) mod send_messages;
pub(crate) mod simulation;
pub(crate) mod simulation_elements;
pub(crate) mod simulation_engine;

use std::{fs::File, ops::RangeInclusive};

use crate::{Cli, Defined};
use chrono::Utc;
use rand::{Rng, SeedableRng};
use rand_distr::{Distribution, Exp, Normal};
use rdkafka::producer::FutureProducer;
use serde::Deserialize;
use simulation::Simulation;
use simulation_engine::{run_schedule, SimulationEngine, SimulationEngineExternals};
use tokio::task::JoinSet;

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
pub(crate) struct Interval<T>
where
    T: Clone,
{
    pub(crate) min: T,
    pub(crate) max: T,
}

impl<T: PartialOrd + Copy> Interval<T> {
    pub(crate) fn range_inclusive(&self) -> RangeInclusive<T> {
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

pub(crate) struct Topics<'a> {
    pub(crate) traces: Option<&'a str>,
    pub(crate) events: Option<&'a str>,
    pub(crate) frame_events: Option<&'a str>,
    pub(crate) run_controls: Option<&'a str>,
}

pub(crate) async fn run_configured_simulation(
    use_otel: bool,
    cli: &Cli,
    producer: &FutureProducer,
    defined: Defined,
) {
    let Defined { file, .. } = defined;

    let simulation: Simulation = serde_json::from_reader(File::open(file).unwrap()).unwrap();
    let mut kafka_producer_thread_set = JoinSet::<()>::new();
    let mut engine = SimulationEngine::new(
        SimulationEngineExternals {
            use_otel,
            producer,
            kafka_producer_thread_set: &mut kafka_producer_thread_set,
        },
        Topics {
            traces: cli.trace_topic.as_deref(),
            events: cli.event_topic.as_deref(),
            frame_events: cli.frame_event_topic.as_deref(),
            run_controls: cli.control_topic.as_deref(),
        },
        &simulation
    );
    run_schedule(&mut engine);
}

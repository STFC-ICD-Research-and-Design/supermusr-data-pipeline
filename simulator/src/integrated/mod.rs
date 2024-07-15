pub(crate) mod build_messages;
pub(crate) mod send_messages;
pub(crate) mod simulation;
pub(crate) mod simulation_elements;
pub(crate) mod simulation_engine;

use crate::{Cli, Defined};
use rdkafka::producer::FutureProducer;
use simulation::Simulation;
use simulation_engine::{run_schedule, SimulationEngine, SimulationEngineExternals};
use std::fs::File;
use tokio::task::JoinSet;

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
        &simulation,
    );
    run_schedule(&mut engine);
}

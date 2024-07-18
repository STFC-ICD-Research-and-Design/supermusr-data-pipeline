pub(crate) mod simulation;
pub(crate) mod simulation_elements;
pub(crate) mod simulation_engine;

use crate::{Cli, Defined};
use rdkafka::producer::FutureProducer;
use simulation::Simulation;
use simulation_engine::{run_schedule, SimulationEngine, SimulationEngineExternals};
use std::fs::File;
use tokio::task::JoinSet;
use tracing::{error, trace};

pub(crate) struct Topics<'a> {
    pub(crate) traces: &'a str,
    pub(crate) events: &'a str,
    pub(crate) frame_events: &'a str,
    pub(crate) run_controls: &'a str,
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
            traces: cli.trace_topic.as_deref().unwrap(),
            events: cli.event_topic.as_deref().unwrap(),
            frame_events: cli.frame_event_topic.as_deref().unwrap(),
            run_controls: cli.control_topic.as_deref().unwrap(),
        },
        &simulation,
    );
    run_schedule(&mut engine);

    trace!("Waiting for delivery threads to finish.");
    while let Some(result) = kafka_producer_thread_set.join_next().await {
        if let Err(e) = result {
            error!("{e}");
        }
    }
    trace!("All finished.");
}

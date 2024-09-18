pub(crate) mod active_pulses;
pub(crate) mod build_messages;
pub(crate) mod send_messages;
pub(crate) mod simulation;
pub(crate) mod simulation_elements;
pub(crate) mod simulation_engine;

use crate::Defined;
use rdkafka::producer::FutureProducer;
use simulation::{Simulation, SimulationError};
use simulation_engine::{
    engine::SimulationEngineError, run_schedule, SimulationEngine, SimulationEngineExternals,
};
use std::fs::File;
use thiserror::Error;
use tokio::task::JoinSet;
use tracing::{error, trace};

pub(crate) struct Topics<'a> {
    pub(crate) traces: &'a str,
    pub(crate) events: &'a str,
    pub(crate) frame_events: &'a str,
    pub(crate) run_controls: &'a str,
    pub(crate) runlog: &'a str,
    pub(crate) selog: &'a str,
    pub(crate) alarm: &'a str,
}

#[derive(Debug, Error)]
pub(crate) enum ConfiguredError {
    #[error("Simulation Error: {0}")]
    Simulation(#[from] SimulationError),
    #[error("Simulation Engine Error: {0}")]
    SimulationEngine(#[from] SimulationEngineError),
    #[error("Json Error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("File Error: {0}")]
    IO(#[from] std::io::Error),
}

#[tracing::instrument(skip_all, target = "otel", err(level = "error"))]
pub(crate) async fn run_configured_simulation(
    use_otel: bool,
    producer: &FutureProducer,
    defined: Defined,
) -> Result<(), ConfiguredError> {
    let simulation: Simulation = serde_json::from_reader(File::open(defined.file)?)?;
    let mut kafka_producer_thread_set = JoinSet::<()>::new();
    let mut engine = SimulationEngine::new(
        SimulationEngineExternals {
            use_otel,
            producer,
            kafka_producer_thread_set: &mut kafka_producer_thread_set,
            topics: Topics {
                traces: &defined.digitiser_trace_topic,
                events: &defined.digitiser_event_topic,
                frame_events: &defined.frame_event_topic,
                run_controls: &defined.control_topic,
                runlog: &defined.runlog_topic,
                selog: &defined.selog_topic,
                alarm: &defined.alarm_topic,
            },
        },
        &simulation,
    )?;

    if let Err(e) = run_schedule(&mut engine) {
        error!("Critical Error: {e}");
    }

    trace!("Waiting for delivery threads to finish.");
    while let Some(result) = kafka_producer_thread_set.join_next().await {
        if let Err(e) = result {
            error!("{e}");
        }
    }

    trace!("All finished.");
    Ok(())
}

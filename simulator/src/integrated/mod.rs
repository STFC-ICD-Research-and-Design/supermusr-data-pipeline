pub(crate) mod active_pulses;
pub(crate) mod build_messages;
pub(crate) mod send_messages;
pub(crate) mod simulation;
pub(crate) mod simulation_elements;
pub(crate) mod simulation_engine;

use crate::{Cli, Defined};
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
    #[error("Topic {0} not given at command line")]
    MissingTopic(&'static str),
}

#[tracing::instrument(skip_all, target = "otel", err(level = "error"))]
pub(crate) async fn run_configured_simulation(
    use_otel: bool,
    cli: &Cli,
    producer: &FutureProducer,
    defined: Defined,
) -> Result<(), ConfiguredError> {
    let Defined { file, .. } = defined;

    let simulation: Simulation = serde_json::from_reader(File::open(file)?)?;
    let mut kafka_producer_thread_set = JoinSet::<()>::new();
    let mut engine = SimulationEngine::new(
        SimulationEngineExternals {
            use_otel,
            producer,
            kafka_producer_thread_set: &mut kafka_producer_thread_set,
            topics: Topics {
                traces: cli
                    .trace_topic
                    .as_deref()
                    .ok_or(ConfiguredError::MissingTopic("trace-topic"))?,
                events: cli
                    .event_topic
                    .as_deref()
                    .ok_or(ConfiguredError::MissingTopic("event-topic"))?,
                frame_events: cli
                    .frame_event_topic
                    .as_deref()
                    .ok_or(ConfiguredError::MissingTopic("frame-event-topic"))?,
                run_controls: cli
                    .control_topic
                    .as_deref()
                    .ok_or(ConfiguredError::MissingTopic("control-topic"))?,
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

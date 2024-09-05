use super::actions::{
    Action, DigitiserAction, FrameAction, GenerateEventList, GenerateTrace, SelectionModeOptions,
    Timestamp, TracingEvent, TracingLevel,
};
use crate::integrated::{
    build_messages::build_trace_message,
    send_messages::{
        send_aggregated_frame_event_list_message, send_alarm_command,
        send_digitiser_event_list_message, send_digitiser_trace_message, send_run_log_command,
        send_run_start_command, send_run_stop_command, send_se_log_command,
    },
    simulation::Simulation,
    simulation_elements::event_list::{EventList, Trace},
    Topics,
};
use anyhow::Result;
use chrono::{DateTime, TimeDelta, Utc};
use rdkafka::producer::FutureProducer;
use std::{collections::VecDeque, thread::sleep, time::Duration};
use supermusr_common::{Channel, DigitizerId, FrameNumber};
use supermusr_streaming_types::{flatbuffers::FlatBufferBuilder, FrameMetadata};
use tokio::task::JoinSet;
use tracing::{debug, error, info, instrument};

#[derive(Clone, Debug)]
pub(crate) struct SimulationEngineState {
    pub(super) metadata: FrameMetadata,
    pub(super) digitiser_index: usize,
    pub(super) delay_from: DateTime<Utc>,
}

impl Default for SimulationEngineState {
    fn default() -> Self {
        Self {
            metadata: FrameMetadata {
                timestamp: Utc::now(),
                period_number: 0,
                protons_per_pulse: 0,
                running: true,
                frame_number: 0,
                veto_flags: 0,
            },
            digitiser_index: Default::default(),
            delay_from: Utc::now(),
        }
    }
}

pub(crate) struct SimulationEngineDigitiser {
    pub(crate) id: DigitizerId,
    pub(crate) channel_indices: Vec<usize>,
}

impl SimulationEngineDigitiser {
    #[instrument(target = "otel", name = "digitiser", skip(channel_indices))]
    pub(crate) fn new(id: DigitizerId, channel_indices: Vec<usize>) -> Self {
        SimulationEngineDigitiser {
            id,
            channel_indices,
        }
    }
}

pub(crate) struct SimulationEngineExternals<'a> {
    pub(crate) use_otel: bool,
    pub(crate) producer: &'a FutureProducer,
    pub(crate) kafka_producer_thread_set: &'a mut JoinSet<()>,
    pub(crate) topics: Topics<'a>,
}

pub(crate) struct SimulationEngine<'a> {
    externals: SimulationEngineExternals<'a>,
    state: SimulationEngineState,
    trace_cache: VecDeque<Trace>,
    event_list_cache: VecDeque<EventList<'a>>,
    simulation: &'a Simulation,
    channels: Vec<Channel>,
    digitiser_ids: Vec<SimulationEngineDigitiser>,
}

impl<'a> SimulationEngine<'a> {
    pub(crate) fn new(
        externals: SimulationEngineExternals<'a>,
        simulation: &'a Simulation,
    ) -> Self {
        if !simulation.validate() {
            error!("Invalid Simulation Object");
        }

        debug!("Creating Simulation Engine");
        Self {
            externals,
            simulation,
            state: Default::default(),
            trace_cache: Default::default(),
            event_list_cache: Default::default(),
            digitiser_ids: simulation.digitiser_config.generate_digitisers(),
            channels: simulation.digitiser_config.generate_channels(),
        }
    }
}

#[instrument(skip_all, level = "debug", target = "otel")]
fn set_timestamp(engine: &mut SimulationEngine, timestamp: &Timestamp) {
    match timestamp {
        Timestamp::Now => engine.state.metadata.timestamp = Utc::now(),
        Timestamp::AdvanceByMs(ms) => {
            engine.state.metadata.timestamp = engine
                .state
                .metadata
                .timestamp
                .checked_add_signed(TimeDelta::milliseconds(*ms as i64))
                .unwrap()
        }
    }
}

#[instrument(skip_all, level = "debug", target = "otel")]
fn wait_ms(ms: usize) {
    sleep(Duration::from_millis(ms as u64));
}

#[instrument(skip_all, level = "debug", target = "otel")]
fn ensure_delay_ms(ms: usize, delay_from: &mut DateTime<Utc>) {
    let duration = TimeDelta::milliseconds(ms as i64);
    if Utc::now() - *delay_from < duration {
        sleep(Duration::from_millis(duration.num_milliseconds() as u64));
    }
    *delay_from = Utc::now();
}

#[instrument(skip_all, level = "debug", target = "otel")]
fn generate_trace_push_to_cache(engine: &mut SimulationEngine, generate_trace: &GenerateTrace) {
    let event_lists = engine.simulation.generate_event_lists(
        generate_trace.event_list_index,
        engine.state.metadata.frame_number,
        generate_trace.repeat,
    );
    let traces = engine
        .simulation
        .generate_traces(event_lists.as_slice(), engine.state.metadata.frame_number);
    engine.trace_cache.extend(traces);
}

#[instrument(skip_all, level = "debug", target = "otel")]
fn generate_trace_fbb_push_to_cache(
    sample_rate: u64,
    trace_cache_fbb: &mut VecDeque<FlatBufferBuilder<'_>>,
    metadata: &FrameMetadata,
    digitizer_id: DigitizerId,
    channels: &[Channel],
    simulation: &Simulation,
    generate_trace: &GenerateTrace,
) {
    let event_lists = simulation.generate_event_lists(
        generate_trace.event_list_index,
        metadata.frame_number,
        generate_trace.repeat,
    );
    let mut traces =
        VecDeque::from(simulation.generate_traces(event_lists.as_slice(), metadata.frame_number));

    let mut fbb = FlatBufferBuilder::new();

    build_trace_message(
        &mut fbb,
        sample_rate,
        &mut traces,
        metadata,
        digitizer_id,
        channels,
        SelectionModeOptions::PopFront,
    )
    .unwrap();

    trace_cache_fbb.push_back(fbb);
}

#[instrument(skip_all, level = "debug", target = "otel")]
fn generate_event_lists_push_to_cache(
    engine: &mut SimulationEngine,
    generate_event: &GenerateEventList,
) {
    let event_lists = engine.simulation.generate_event_lists(
        generate_event.event_list_index,
        engine.state.metadata.frame_number,
        generate_event.repeat,
    );
    engine.event_list_cache.extend(event_lists);
}

fn tracing_event(event: &TracingEvent) {
    match event.level {
        TracingLevel::Info => info!(event.message),
        TracingLevel::Debug => debug!(event.message),
    }
}

#[tracing::instrument(skip_all, level = "debug", target = "otel", fields(num_actions = engine.simulation.schedule.len()))]
pub(crate) fn run_schedule(engine: &mut SimulationEngine) -> Result<()> {
    for action in engine.simulation.schedule.iter() {
        match action {
            Action::WaitMs(ms) => wait_ms(*ms),
            Action::EnsureDelayMs(ms) => ensure_delay_ms(*ms, &mut engine.state.delay_from),
            Action::TracingEvent(event) => tracing_event(event),
            Action::SendRunStart(run_start) => send_run_start_command(
                &mut engine.externals,
                run_start,
                &engine.state.metadata.timestamp,
            )?,
            Action::SendRunStop(run_stop) => send_run_stop_command(
                &mut engine.externals,
                run_stop,
                &engine.state.metadata.timestamp,
            )?,
            Action::SendRunLogData(run_log_data) => send_run_log_command(
                &mut engine.externals,
                &engine.state.metadata.timestamp,
                run_log_data,
            )?,
            Action::SendSampleEnvLog(sample_env_log) => {
                send_se_log_command(
                    &mut engine.externals,
                    &engine.state.metadata.timestamp,
                    sample_env_log,
                )
                .unwrap();
            }
            Action::SendAlarm(alarm) => {
                send_alarm_command(
                    &mut engine.externals,
                    &engine.state.metadata.timestamp,
                    alarm,
                )?;
            }
            Action::SetVetoFlags(vetoes) => {
                engine.state.metadata.veto_flags = *vetoes;
            }
            Action::SetPeriod(period) => {
                engine.state.metadata.period_number = *period;
            }
            Action::SetProtonsPerPulse(ppp) => {
                engine.state.metadata.protons_per_pulse = *ppp;
            }
            Action::SetRunning(running) => {
                engine.state.metadata.running = *running;
            }
            Action::GenerateTrace(generate_trace) => {
                generate_trace_push_to_cache(engine, generate_trace)
            }
            Action::GenerateEventList(generate_event) => {
                generate_event_lists_push_to_cache(engine, generate_event)
            }
            Action::SetTimestamp(timestamp) => set_timestamp(engine, timestamp),
            Action::FrameLoop(frame_loop) => {
                for frame in frame_loop.start.value()..=frame_loop.end.value() {
                    engine.state.metadata.frame_number = frame as FrameNumber;
                    run_frame(engine, frame_loop.schedule.as_slice())?;
                }
            }
            Action::Comment(_) => (),
        }
    }
    Ok(())
}

#[tracing::instrument(
    skip_all, level = "debug", target = "otel",
    fields(
        frame_number = engine.state.metadata.frame_number,
        num_actions = frame_actions.len()
    )
)]
fn run_frame(engine: &mut SimulationEngine, frame_actions: &[FrameAction]) -> Result<()> {
    for action in frame_actions {
        match action {
            FrameAction::WaitMs(ms) => wait_ms(*ms),
            FrameAction::EnsureDelayMs(ms) => ensure_delay_ms(*ms, &mut engine.state.delay_from),
            FrameAction::TracingEvent(event) => tracing_event(event),
            FrameAction::SendAggregatedFrameEventList(source) => {
                send_aggregated_frame_event_list_message(
                    &mut engine.externals,
                    &mut engine.event_list_cache,
                    &engine.state.metadata,
                    &source
                        .channel_indices
                        .range_inclusive()
                        .map(|i| *engine.channels.get(i).unwrap())
                        .collect::<Vec<_>>(),
                    &source.source_options,
                )?
            }
            FrameAction::GenerateTrace(generate_trace) => {
                generate_trace_push_to_cache(engine, generate_trace)
            }
            FrameAction::GenerateEventList(generate_event) => {
                generate_event_lists_push_to_cache(engine, generate_event)
            }
            FrameAction::SetTimestamp(timestamp) => set_timestamp(engine, timestamp),
            FrameAction::DigitiserLoop(digitiser_loop) => {
                for digitiser in digitiser_loop.start.value()..=digitiser_loop.end.value() {
                    engine.state.digitiser_index = digitiser as usize;
                    run_digitiser(engine, &digitiser_loop.schedule)?;
                }
            }
            FrameAction::Comment(_) => (),
        }
    }
    Ok(())
}

#[tracing::instrument(skip_all, level = "debug", target = "otel",
    fields(
        frame_number = engine.state.metadata.frame_number,
        digitiser_id = engine.digitiser_ids[engine.state.digitiser_index].id,
        num_actions = digitiser_actions.len()
    )
)]
pub(crate) fn run_digitiser<'a>(
    engine: &'a mut SimulationEngine,
    digitiser_actions: &[DigitiserAction],
) -> anyhow::Result<()> {
    for action in digitiser_actions {
        match action {
            DigitiserAction::WaitMs(ms) => wait_ms(*ms),
            DigitiserAction::EnsureDelayMs(ms) => {
                ensure_delay_ms(*ms, &mut engine.state.delay_from)
            }
            DigitiserAction::TracingEvent(event) => tracing_event(event),
            DigitiserAction::SendDigitiserTrace(source) => {
                send_digitiser_trace_message(
                    &mut engine.externals,
                    engine.simulation.sample_rate,
                    &mut engine.trace_cache,
                    &engine.state.metadata,
                    engine
                        .digitiser_ids
                        .get(engine.state.digitiser_index)
                        .unwrap()
                        .id,
                    &engine
                        .digitiser_ids
                        .get(engine.state.digitiser_index)
                        .unwrap()
                        .channel_indices
                        .iter()
                        .map(|idx| engine.channels[*idx])
                        .collect::<Vec<_>>(),
                    source.0,
                )
                .unwrap();
            }
            DigitiserAction::SendDigitiserEventList(source) => {
                send_digitiser_event_list_message(
                    &mut engine.externals,
                    &mut engine.event_list_cache,
                    &engine.state.metadata,
                    engine
                        .digitiser_ids
                        .get(engine.state.digitiser_index)
                        .unwrap()
                        .id,
                    &engine
                        .digitiser_ids
                        .get(engine.state.digitiser_index)
                        .unwrap()
                        .channel_indices
                        .iter()
                        .map(|idx| engine.channels[*idx])
                        .collect::<Vec<_>>(),
                    &source.0,
                )
                .unwrap();
            }
            DigitiserAction::GenerateTrace(generate_trace) => {
                generate_trace_push_to_cache(engine, generate_trace)
            }
            DigitiserAction::GenerateEventList(generate_event) => {
                generate_event_lists_push_to_cache(engine, generate_event)
            }
            DigitiserAction::Comment(_) => (),
        }
    }
    Ok(())
}

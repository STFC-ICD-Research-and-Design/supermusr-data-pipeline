use super::cache::SimulationEngineCache;
use super::send_messages::{
    send_aggregated_frame_event_list_message, send_alarm_command,
    send_digitiser_event_list_message, send_run_log_command, send_run_start_command,
    send_run_stop_command, send_se_log_command, send_trace_message,
};
use chrono::{TimeDelta, Utc};
use rdkafka::producer::FutureProducer;
use std::collections::VecDeque;
use tokio::task::JoinSet;
use tracing::debug;

use super::scheduler::{
    Action, DigitiserAction, FrameAction, GenerateEventList, GenerateTrace, Timestamp,
};
use super::simulation::Simulation;
use super::simulation_elements::event_list::EventList;
use super::Topics;
use supermusr_common::{Channel, DigitizerId, FrameNumber, Intensity};
use supermusr_streaming_types::FrameMetadata;

/*
#[derive(Default)]
pub(crate) struct SimulationEngineCache<'a> {
    trace_cache: VecDeque<Vec<Intensity>>,
    event_list_cache: VecDeque<EventList<'a>>,
}

impl<'a> SimulationEngineCache<'a> {
    pub(crate) fn get_trace(&self, selection_mode: SelectionModeOptions) -> &Vec<Intensity> {
        match selection_mode {
            SelectionModeOptions::PopFront => self.trace_cache.front(),
            SelectionModeOptions::ReplaceRandom => self.trace_cache.get(0),
        }
        .unwrap()
    }
    fn push_back_trace(&mut self, value: Vec<Vec<Intensity>>) {
        self.trace_cache.extend(value)
    }
    pub(crate) fn finish_trace(&mut self, selection_mode: SelectionModeOptions) {
        match selection_mode {
            SelectionModeOptions::PopFront => {
                self.trace_cache.pop_front();
            }
            SelectionModeOptions::ReplaceRandom => (),
        };
    }

    pub(crate) fn get_event_lists(
        &self,
        selection_mode: SelectionModeOptions,
        amount: usize,
    ) -> Vec<&EventList<'a>> {
        match selection_mode {
            SelectionModeOptions::PopFront => self.event_list_cache.iter().take(amount).collect(),
            SelectionModeOptions::ReplaceRandom => {
                let mut rng =
                    rand::rngs::StdRng::seed_from_u64(Utc::now().timestamp_subsec_nanos() as u64);
                let mut indices = (0..self.trace_cache.len()).collect::<Vec<_>>();
                let (random_indices, _) = indices.partial_shuffle(&mut rng, amount);
                random_indices
                    .into_iter()
                    .map(|i| self.event_list_cache.get(*i).unwrap())
                    .collect()
            }
        }
    }
    fn push_back_event_lists(&mut self, value: Vec<EventList<'a>>) {
        self.event_list_cache.extend(value)
    }
    pub(crate) fn finish_event_lists(
        &mut self,
        selection_mode: SelectionModeOptions,
        amount: usize,
    ) {
        match selection_mode {
            SelectionModeOptions::PopFront => {
                self.event_list_cache.drain(0..amount);
            }
            SelectionModeOptions::ReplaceRandom => (),
        };
    }
} */

#[derive(Clone, Debug)]
pub(crate) struct SimulationEngineState {
    pub(super) metadata: FrameMetadata,
    pub(super) digitiser_index: usize,
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
        }
    }
}
pub(crate) struct SimulationEngineDigitiser {
    pub(crate) id: DigitizerId,
    pub(crate) channel_indices: Vec<usize>,
}

pub(crate) struct SimulationEngineExternals<'a> {
    pub(crate) use_otel: bool,
    pub(crate) producer: &'a FutureProducer,
    pub(crate) kafka_producer_thread_set: &'a mut JoinSet<()>,
}

pub(crate) struct SimulationEngine<'a> {
    topics: Topics<'a>,

    externals: SimulationEngineExternals<'a>,
    state: SimulationEngineState,
    trace_cache: VecDeque<Vec<Intensity>>,
    event_list_cache: VecDeque<EventList<'a>>,
    //cache: SimulationEngineCache<'a>,
    simulation: &'a Simulation,
    channels: Vec<Channel>,
    digitiser_ids: Vec<SimulationEngineDigitiser>,
}

impl<'a> SimulationEngine<'a> {
    pub(crate) fn new(
        externals: SimulationEngineExternals<'a>,
        topics: Topics<'a>,
        simulation: &'a Simulation,
    ) -> Self {
        debug!("Creating Simulation Engine");
        Self {
            externals,
            topics,
            simulation,
            state: Default::default(),
            trace_cache: Default::default(),
            event_list_cache: Default::default(),
            digitiser_ids: simulation.digitiser_config.generate_digitisers(),
            channels: simulation.digitiser_config.generate_channels(),
            //actions: Self::unfold_actions(&simulation.schedule, Vec::<Action>::new()),
        }
        //debug!("Creating Simulation has {0} actions", me.actions.len());
    }
    /*
    fn unfold_actions(schedule: &[Action], mut actions: Vec<Action>) -> Vec<Action> {
        for action in schedule {
            match action {
                Action::Loop(lp) => {
                    for i in lp.start..=lp.end {
                        match lp.variable {
                            super::schedule::LoopVariable::Frame => {
                                actions.push(Action::SetFrame(i as FrameNumber))
                            }
                            super::schedule::LoopVariable::Digitiser => {
                                actions.push(Action::SetDigitiserIndex(i))
                            }
                            super::schedule::LoopVariable::Channel => {
                                actions.push(Action::SetChannelIndex(i))
                            }
                            _ => (),
                        }
                        actions = Self::unfold_actions(&lp.schedule, actions);
                    }
                }
                Action::Comment(_) => (),
                other => actions.push(other.clone()),
            }
        }
        actions
    } */
}

fn set_timestamp<'a>(engine: &'a mut SimulationEngine, timestamp: &Timestamp) {
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

fn wait_ms<'a>(engine: &'a mut SimulationEngine, ms: usize) {
    while Utc::now()
        .signed_duration_since(&engine.state.metadata.timestamp)
        .num_milliseconds()
        < ms as i64
    {}
}

fn generate_trace_push_to_cache<'a>(
    engine: &'a mut SimulationEngine,
    generate_trace: &GenerateTrace,
) {
    let events = engine
        .event_list_cache
        .extract(generate_trace.selection_mode, generate_trace.repeat);
    let traces = engine
        .simulation
        .generate_traces(&events, engine.state.metadata.frame_number);
    engine.trace_cache.extend(traces);
    engine
        .event_list_cache
        .finish(generate_trace.selection_mode, generate_trace.repeat);
}

fn generate_event_lists_push_to_cache<'a>(
    engine: &'a mut SimulationEngine,
    generate_event: &GenerateEventList,
) {
    {
        let event_lists = engine.simulation.generate_event_lists(
            generate_event.template_index,
            engine.state.metadata.frame_number,
            generate_event.repeat,
        );
        engine.event_list_cache.extend(event_lists);
    }
}

#[tracing::instrument(skip_all, target = "otel", fields(num_actions = engine.simulation.schedule.len()))]
pub(crate) fn run_schedule<'a>(engine: &'a mut SimulationEngine) {
    for action in engine.simulation.schedule.iter() {
        match action {
            Action::WaitMs(ms) => wait_ms(engine, *ms),
            Action::SendRunStart(run_start) => {
                send_run_start_command(
                    &mut engine.externals,
                    run_start,
                    engine.topics.run_controls.unwrap(),
                    &engine.state.metadata.timestamp,
                )
                .unwrap();
            }
            Action::SendRunStop(run_stop) => {
                send_run_stop_command(
                    &mut engine.externals,
                    run_stop,
                    engine.topics.run_controls.unwrap(),
                    &engine.state.metadata.timestamp,
                )
                .unwrap();
            }
            Action::SendRunLogData(run_log_data) => {
                send_run_log_command(
                    &mut engine.externals,
                    &engine.state.metadata.timestamp,
                    run_log_data,
                    engine.topics.run_controls.unwrap(),
                )
                .unwrap();
            }
            Action::SendSampleEnvLog(sample_env_log) => {
                send_se_log_command(
                    &mut engine.externals,
                    &engine.state.metadata.timestamp,
                    sample_env_log,
                    engine.topics.run_controls.unwrap(),
                )
                .unwrap();
            }
            Action::SendAlarm(alarm) => {
                send_alarm_command(
                    &mut engine.externals,
                    &engine.state.metadata.timestamp,
                    alarm,
                    engine.topics.run_controls.unwrap(),
                )
                .unwrap();
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
                for frame in frame_loop.start..frame_loop.end {
                    engine.state.metadata.frame_number = frame as FrameNumber;
                    run_frame(engine, frame_loop.schedule.as_slice());
                }
            }
            Action::Comment(_) => (),
        }
    }
}

#[tracing::instrument(skip_all, target = "otel", fields(
    frame = engine.state.metadata.frame_number,
    num_actions = frame_actions.len()
))]
fn run_frame<'a>(engine: &'a mut SimulationEngine, frame_actions: &[FrameAction]) {
    for action in frame_actions {
        match action {
            FrameAction::WaitMs(ms) => wait_ms(engine, *ms),
            FrameAction::SendAggregatedFrameEventList(source) => {
                send_aggregated_frame_event_list_message(
                    &mut engine.externals,
                    engine.topics.frame_events.unwrap(),
                    &mut engine.event_list_cache,
                    &engine.state.metadata,
                    &source
                        .channel_indices
                        .range_inclusive()
                        .map(|i| *engine.channels.get(i).unwrap())
                        .collect::<Vec<_>>(),
                    &source.source_options,
                )
                .unwrap();
            }
            FrameAction::GenerateTrace(generate_trace) => {
                generate_trace_push_to_cache(engine, generate_trace)
            }
            FrameAction::GenerateEventList(generate_event) => {
                generate_event_lists_push_to_cache(engine, generate_event)
            }
            FrameAction::SetTimestamp(timestamp) => set_timestamp(engine, timestamp),
            FrameAction::DigitiserLoop(digitiser_loop) => {
                for digitiser in digitiser_loop.start..digitiser_loop.end {
                    engine.state.digitiser_index = digitiser;
                    run_digitiser(engine, &digitiser_loop.schedule);
                }
            }
            FrameAction::Comment(_) => (),
        }
    }
}

#[tracing::instrument(skip_all, fields(
    digitiser = engine.digitiser_ids[engine.state.digitiser_index].id,
    num_actions = digitiser_actions.len()
))]
pub(crate) fn run_digitiser<'a>(
    engine: &'a mut SimulationEngine,
    digitiser_actions: &[DigitiserAction],
) {
    for action in digitiser_actions {
        match action {
            DigitiserAction::WaitMs(ms) => wait_ms(engine, *ms),
            DigitiserAction::SendDigitiserTrace(source) => {
                send_trace_message(
                    &mut engine.externals,
                    engine.topics.traces.unwrap(),
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
                    engine.topics.events.unwrap(),
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
}

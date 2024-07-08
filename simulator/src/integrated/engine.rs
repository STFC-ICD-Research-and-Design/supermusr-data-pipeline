use crate::send_messages::{
    send_aggregated_frame_event_list_message, send_digitiser_event_list_message,
    send_run_start_command, send_run_stop_command, send_trace_message,
};
use chrono::{DateTime, TimeDelta, Utc};
use rdkafka::producer::FutureProducer;
use std::collections::VecDeque;
use tokio::task::JoinSet;
use tracing::debug;

use super::schedule::{Action, SelectionModeOptions, Timestamp};
use super::simulation::Simulation;
use super::simulation_elements::event_list::EventList;
use super::Topics;
use supermusr_common::{Channel, DigitizerId, FrameNumber, Intensity};
use supermusr_streaming_types::FrameMetadata;

#[derive(Default)]
pub(crate) struct SimulationEngineCache<'a> {
    trace_cache: VecDeque<Vec<Intensity>>,
    event_list_cache: VecDeque<EventList<'a>>,
}

impl<'a> SimulationEngineCache<'a> {
    pub(crate) fn get_trace(&self, selection_mode: SelectionModeOptions) -> &Vec<Intensity> {
        match selection_mode {
            super::schedule::SelectionModeOptions::PopFront => self.trace_cache.front(),
        }
        .unwrap()
    }
    fn push_back_trace(&mut self, value: Vec<Vec<Intensity>>) {
        self.trace_cache.extend(value)
    }
    pub(crate) fn finish_trace(&mut self, selection_mode: SelectionModeOptions) {
        match selection_mode {
            super::schedule::SelectionModeOptions::PopFront => self.trace_cache.pop_front(),
        }
        .unwrap();
    }

    pub(crate) fn get_event_lists(
        &self,
        selection_mode: SelectionModeOptions,
        amount: usize,
    ) -> Vec<&EventList<'a>> {
        match selection_mode {
            super::schedule::SelectionModeOptions::PopFront => {
                self.event_list_cache.iter().take(amount).collect()
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
            super::schedule::SelectionModeOptions::PopFront => {
                self.event_list_cache.drain(0..amount)
            }
        };
    }
}

#[derive(Clone, Debug)]
pub(crate) struct SimulationEngineState {
    pub(super) metadata: FrameMetadata,
    pub(super) digitiser_index: usize,
    pub(super) channel_index: usize,
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
            channel_index: Default::default(),
        }
    }
}
pub(crate) struct SimulationEngineDigitiser {
    pub(crate) id: DigitizerId,
    pub(crate) channel_indices: Vec<usize>,
}

pub(crate) struct SimulationEngineImmutableProperties<'a> {
    pub(crate) use_otel: bool,
    pub(crate) producer: &'a FutureProducer,
    pub(crate) kafka_producer_thread_set: &'a mut JoinSet<()>,
}

pub(crate) struct SimulationEngine<'a> {
    topics: Topics<'a>,

    immutable: SimulationEngineImmutableProperties<'a>,

    simulation: &'a Simulation,
    channels: Vec<Channel>,
    digitiser_ids: Vec<SimulationEngineDigitiser>,
    actions: Vec<Action>,
}

impl<'a> SimulationEngine<'a> {
    pub(crate) fn new(
        immutable: SimulationEngineImmutableProperties<'a>,
        topics: Topics<'a>,
        simulation: &'a Simulation,
    ) -> Self {
        debug!("Creating Simulation Engine");
        let me = Self {
            immutable,
            topics,
            simulation,
            digitiser_ids: simulation.digitiser_config.generate_digitisers(),
            channels: simulation.digitiser_config.generate_channels(),
            actions: Self::unfold_actions(&simulation.schedule, Vec::<Action>::new()),
        };
        debug!("Creating Simulation has {0} actions", me.actions.len());
        me
    }

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
    }

    pub(crate) fn run(
        &'a mut self,
        state: &mut SimulationEngineState,
        cache: &'a mut SimulationEngineCache<'a>,
    ) {
        for action in self.actions.iter() {
            match action {
                Action::WaitMs(ms) => {
                    while Utc::now()
                        .signed_duration_since(&state.metadata.timestamp)
                        .num_milliseconds()
                        < *ms as i64
                    {}
                }
                Action::SendRunStart(run_start) => {
                    send_run_start_command(
                        &mut self.immutable,
                        run_start,
                        self.topics.run_controls.unwrap(),
                        &state.metadata.timestamp,
                    )
                    .unwrap();
                }
                Action::SendRunStop(run_stop) => {
                    send_run_stop_command(
                        &mut self.immutable,
                        run_stop,
                        self.topics.run_controls.unwrap(),
                        &state.metadata.timestamp,
                    )
                    .unwrap();
                }
                Action::SendRunLogData(run_log_data) => {}
                Action::SendSampleEnvLog(sample_env_log) => {}
                Action::SendAlarm(alarm) => {}
                Action::SendDigitiserTrace(source) => {
                    send_trace_message(
                        &mut self.immutable,
                        self.topics.traces.unwrap(),
                        self.simulation.sample_rate,
                        cache,
                        &state.metadata,
                        self.digitiser_ids[state.digitiser_index].id,
                        &self.digitiser_ids[state.digitiser_index]
                            .channel_indices
                            .iter()
                            .map(|idx| self.channels[*idx])
                            .collect::<Vec<_>>(),
                        source.0,
                    )
                    .unwrap();
                }
                Action::SendDigitiserEventList(source) => {
                    send_digitiser_event_list_message(
                        &mut self.immutable,
                        self.topics.events.unwrap(),
                        cache,
                        &state.metadata,
                        self.digitiser_ids[state.digitiser_index].id,
                        &self.digitiser_ids[state.digitiser_index]
                            .channel_indices
                            .iter()
                            .map(|idx| self.channels[*idx])
                            .collect::<Vec<_>>(),
                        &source.0,
                    )
                    .unwrap();
                }
                Action::SendAggregatedFrameEventList(source) => {
                    send_aggregated_frame_event_list_message(
                        &mut self.immutable,
                        self.topics.frame_events.unwrap(),
                        cache,
                        &state.metadata,
                        &source
                            .channel_indices
                            .range_inclusive()
                            .map(|i| *self.channels.get(i).unwrap())
                            .collect::<Vec<_>>(),
                        &source.source_options,
                    )
                    .unwrap();
                }
                Action::SetVetoFlags(vetoes) => {
                    state.metadata.veto_flags = *vetoes;
                }
                Action::SetFrame(frame) => {
                    state.metadata.frame_number = *frame;
                }
                Action::SetDigitiserIndex(dig_index) => {
                    state.digitiser_index = *dig_index;
                }
                Action::SetChannelIndex(channel_index) => {
                    state.channel_index = *channel_index;
                }
                Action::SetPeriod(period) => {
                    state.metadata.period_number = *period;
                }
                Action::SetProtonsPerPulse(ppp) => {
                    state.metadata.protons_per_pulse = *ppp;
                }
                Action::SetRunning(running) => {
                    state.metadata.running = *running;
                }
                Action::GenerateTrace(generate_trace) => {
                    let events =
                        cache.get_event_lists(generate_trace.selection_mode, generate_trace.repeat);
                    let traces = self
                        .simulation
                        .generate_traces(&events, state.metadata.frame_number);
                    cache.push_back_trace(traces);
                    cache.finish_event_lists(generate_trace.selection_mode, generate_trace.repeat);
                }
                Action::GenerateEventList(generate_event) => {
                    let event_lists = self.simulation.generate_event_lists(
                        generate_event.template_index,
                        state.metadata.frame_number,
                        generate_event.repeat,
                    );
                    cache.push_back_event_lists(event_lists);
                }
                Action::SetTimestamp(timestamp) => match timestamp {
                    Timestamp::Now => state.metadata.timestamp = Utc::now(),
                    Timestamp::AdvanceByMs(ms) => {
                        state.metadata.timestamp = state
                            .metadata
                            .timestamp
                            .checked_add_signed(TimeDelta::milliseconds(*ms as i64))
                            .unwrap()
                    }
                },
                _ => unreachable!(),
            }
        }
    }
}

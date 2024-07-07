use crate::send_messages::send_trace_message;
use chrono::{DateTime, TimeDelta, Utc};
use rand::SeedableRng;
use rand_distr::{Distribution, WeightedIndex};
use rdkafka::producer::FutureProducer;
use rdkafka::statistics::Topic;
use std::collections::VecDeque;
use tokio::task::JoinSet;

use super::schedule::{Action, Source};
use super::simulation::Simulation;
use super::simulation_elements::{
    event_list::{EventList, EventListTemplate},
    muon::{MuonAttributes, MuonEvent},
    noise::Noise,
};
use super::Topics;
use supermusr_common::{Channel, DigitizerId, FrameNumber, Intensity};
use supermusr_streaming_types::FrameMetadata;

#[derive(Default)]
pub(crate) struct Cache<'a> {
    trace_cache: VecDeque<Vec<Intensity>>,
    event_list_cache: VecDeque<EventList<'a>>,
}

impl<'a> Cache<'a> {
    pub(crate) fn get_trace(&self, source: &Source) -> &Vec<Intensity> {
        match source.selection_mode {
            super::schedule::SelectionModeOptions::PopFront => self.trace_cache.front(),
        }
        .unwrap()
    }
    fn push_back_trace(&mut self, value: Vec<Intensity>) {
        self.trace_cache.push_back(value)
    }
    pub(crate) fn finish_trace(&mut self, source: &Source) {
        match source.selection_mode {
            super::schedule::SelectionModeOptions::PopFront => self.trace_cache.pop_front(),
        }
        .unwrap();
    }

    pub(crate) fn get_event_list(&self, source: &Source) -> &EventList<'a> {
        match source.selection_mode {
            super::schedule::SelectionModeOptions::PopFront => self.event_list_cache.front(),
        }
        .unwrap()
    }
    fn push_back_event_list(&mut self, value: EventList<'a>) {
        self.event_list_cache.push_back(value)
    }
    pub(crate) fn finish_event_list(&mut self, source: &Source) {
        match source.selection_mode {
            super::schedule::SelectionModeOptions::PopFront => self.event_list_cache.pop_front(),
        }
        .unwrap();
    }
}

#[derive(Clone)]
pub(crate) struct SimulationEngineState {
    pub(super) current_time: DateTime<Utc>,
    pub(super) metadata: FrameMetadata,
    pub(super) digitiser_index: usize,
    pub(super) channel_index: usize,
}

impl Default for SimulationEngineState {
    fn default() -> Self {
        Self {
            current_time: Utc::now(),
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

pub(crate) struct SimulationEngine<'a> {
    use_otel: bool,
    producer: &'a FutureProducer,
    simulation: &'a Simulation,
    topics: &'a Topics<'a>,

    channels: Vec<Channel>,
    digitiser_ids: Vec<(DigitizerId, Vec<usize>)>,
    actions: Vec<Action>,
}

impl<'a> SimulationEngine<'a> {
    pub(crate) fn new(
        use_otel: bool,
        producer: &'a FutureProducer,
        topics: &'a Topics<'a>,
        simulation: &'a Simulation,
    ) -> Self {
        Self {
            use_otel,
            producer,
            topics,
            simulation,
            digitiser_ids: Default::default(),
            channels: Default::default(),
            actions: Self::unfold_actions(&simulation.schedule, Vec::<Action>::new()),
        }
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
                other => actions.push(other.clone()),
            }
        }
        actions
    }
}

pub(crate) fn run<'a>(
    engine: &'a mut SimulationEngine,
    kafka_producer_thread_set: &mut JoinSet<()>,
    state: &mut SimulationEngineState,
    cache: &'a mut Cache<'a>,
) {
    run_schedule(
        engine,
        kafka_producer_thread_set,
        state,
        cache,
        &engine.actions,
    );
}

fn run_schedule<'a>(
    engine: &'a SimulationEngine,
    kafka_producer_thread_set: &mut JoinSet<()>,
    state: &mut SimulationEngineState,
    cache: &'a mut Cache<'a>,
    schedule: &'a [Action],
) {
    for action in schedule {
        match action {
            Action::WaitMs(ms) => todo!(),
            Action::SendRunStart(run_start) => {
                //create_run_start_command()
            }
            Action::SendRunStop(run_stop) => {
                //create_run_start_command()
            }
            Action::SendRunLogData(run_log_data) => {}
            Action::SendSampleEnvLog(sample_env_log) => {}
            Action::SendAlarm(alarm) => {}
            Action::SendDigitiserTrace(source) => {
                send_trace_message(
                    engine.use_otel,
                    engine.producer,
                    kafka_producer_thread_set,
                    engine.topics.traces.unwrap(),
                    engine.simulation,
                    cache,
                    state.current_time.try_into().unwrap(),
                    &state.metadata,
                    engine.digitiser_ids[state.digitiser_index].0,
                    &engine.digitiser_ids[state.digitiser_index]
                        .1
                        .iter()
                        .map(|idx| engine.channels[*idx])
                        .collect::<Vec<_>>(),
                    source,
                )
                .unwrap();
            }
            Action::SendDigitiserEventList(source) => {}
            Action::SendAggregatedFrameEventList(source) => {}
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
            Action::GenerateTrace(source) => {
                let event = cache.get_event_list(source);
                let trace = engine
                    .simulation
                    .generate_trace(event, state.metadata.frame_number);
                cache.push_back_trace(trace);
                cache.finish_event_list(source);
            }
            Action::GenerateEventList(idx) => {
                let event_list = engine
                    .simulation
                    .generate_event_list(*idx, state.metadata.frame_number);
                cache.push_back_event_list(event_list);
            }
            Action::Loop(_) => unreachable!(),
            Action::SetTimestampToNow() => state.current_time = Utc::now(),
            Action::AdvanceTimestampByMs(ms) => {
                state.current_time = state
                    .current_time
                    .checked_add_signed(TimeDelta::milliseconds(*ms as i64))
                    .unwrap()
            }
        }
    }
}

use std::collections::VecDeque;

use chrono::Utc;
use rand::SeedableRng;
use rand_distr::{Distribution, WeightedIndex};
use rdkafka::client::DefaultClientContext;
use super::event_list::{EventList, EventListTemplate};
use super::muon::{MuonAttributes,MuonEvent};
use super::noise::Noise;
use super::run_messages::{Alarm, RunLogData, RunStart, RunStop, SampleEnvLog};
use super::schedule::{Action, Source};
use super::{Simulation};
use serde::Deserialize;
use supermusr_common::{Channel, DigitizerId, FrameNumber, Intensity, Time};
use supermusr_streaming_types::{frame_metadata_v2_generated::FrameMetadataV2, FrameMetadata};

struct SimulationEngine<'a> {
    simulation: &'a Simulation,
    current_time_ns : Time,
    metadata : FrameMetadata,
    digitiser_index : usize,

    channel_index : usize,

    channels : Vec<Channel>,
    digitiser_ids : Vec<(DigitizerId,Vec<usize>)>,

    trace_cache : VecDeque<Vec<Intensity>>,
    event_list_cache : VecDeque<EventList<'a>>,
}

impl<'a> SimulationEngine<'a> {
    fn new(simulation : &'a Simulation) -> Self {
        Self {
            simulation,
            current_time_ns: 0,
            metadata: FrameMetadata {
                timestamp: Utc::now(),
                period_number: 0,
                protons_per_pulse: 0,
                running: true,
                frame_number: 0,
                veto_flags: 0,
            },
            digitiser_index: 0,
            channel_index: 0,
            digitiser_ids: Default::default(),
            channels: Default::default(),
            trace_cache: Default::default(),
            event_list_cache: Default::default(),
        }
    }
    fn run(&'a mut self) {
        run_schedule(self, &self.simulation.schedule);
    }

    fn get_random_pulse_attributes(
        &'a self,
        source: &EventListTemplate,
        distr: &WeightedIndex<f64>
    ) -> &MuonAttributes {
        //  get a random index for the pulse
        let index = distr.sample(&mut rand::rngs::StdRng::seed_from_u64(
            Utc::now().timestamp_subsec_nanos() as u64,
        ));
        // Return a pointer to either a local or global pulse
        self.simulation.pulses.get(source.pulses.get(index).unwrap().index).unwrap()
    }

    fn generate_event_list(&self, index: usize) -> EventList {
        let source = self.simulation.event_lists.get(index).unwrap();
        let distr = WeightedIndex::new(source.pulses.iter().map(|p| p.weight)).unwrap();
        EventList {
            pulses: {
                // Creates a unique template for each channel
                let mut pulses = (0..source.num_pulses.sample(self.metadata.frame_number as usize) as usize)
                    .map(|_| {
                        MuonEvent::sample(
                            self.get_random_pulse_attributes(source, &distr),
                            self.metadata.frame_number as usize,
                        )
                    })
                    .collect::<Vec<_>>();
                pulses.sort_by_key(|a| a.get_start());
                pulses
            },
            noises: &source.noises,
        }
    }

    fn get_cached_event_list(&self, source: &Source) -> &EventList {
        match source.selection_mode {
            super::schedule::SelectionModeOptions::PopFront => 
                self.event_list_cache.front()
        }.unwrap()
    }
    fn process_cached_event_list(&mut self, source: &Source) {
        match source.selection_mode {
            super::schedule::SelectionModeOptions::PopFront => 
                self.event_list_cache.pop_front()
        }.unwrap();
    }

    fn generate_trace(&self, event_list: &EventList) -> Vec<Intensity> {
        
        let sample_time = 1_000_000_000.0 / self.simulation.sample_rate as f64;

        let mut noise = event_list.noises.iter().map(Noise::new).collect::<Vec<_>>();
        let mut active_muons = VecDeque::<&'a MuonEvent>::new();
        let mut muon_iter = event_list.pulses.iter();
        (0..self.simulation.time_bins)
            .map(|time| {
                //  Remove any expired muons
                while active_muons
                    .front()
                    .and_then(|m| (m.get_end() < time).then_some(m))
                    .is_some()
                {
                    active_muons.pop_front();
                }
                //  Append any new muons
                while let Some(iter) = muon_iter
                    .next()
                    .and_then(|iter| (iter.get_start() > time).then_some(iter))
                {
                    active_muons.push_back(iter)
                }

                //  Sum the signal of the currenty active muons
                let signal = active_muons
                    .iter()
                    .map(|p| p.get_value_at(time as f64 * sample_time))
                    .sum::<f64>();
                noise.iter_mut().fold(signal, |signal, n| {
                    n.noisify(signal, time, self.metadata.frame_number as usize)
                })
            })
            .map(|x: f64| self.simulation.voltage_transformation.transform(x) as Intensity)
            .collect()
    }
}


fn run_schedule<'a>(engine : &'a mut SimulationEngine<'a>, schedule : &[Action]) {
    for action in schedule {
        match action {
            Action::WaitMs(ms) => todo!(),
            Action::RunStart(run_start) => todo!(),
            Action::RunStop(run_stop) => todo!(),
            Action::RunLogData(run_log_data) => todo!(),
            Action::SampleEnvLog(sample_env_log) => todo!(),
            Action::Alarm(alarm) => todo!(),
            Action::DigitiserTrace() => todo!(),
            Action::DigitiserEventList() => todo!(),
            Action::AggregatedFrameEventList() => todo!(),
            Action::EmitFrameEventList() => todo!(),
            Action::EmitDigitiserEventList() => todo!(),
            Action::EmitDigitiserTrace() => todo!(),
            Action::Loop(lp) => todo!(),
            Action::SetVetoFlags(vetoes) => { engine.metadata.veto_flags = *vetoes; },
            Action::SetFrame(frame) => { engine.metadata.frame_number = *frame; },
            Action::SetPeriod(period) => { engine.metadata.period_number = *period; },
            Action::SetProtonsPerPulse(ppp) => { engine.metadata.protons_per_pulse = *ppp; },
            Action::SetRunning(running) => { engine.metadata.running = *running; },
            Action::GenerateTrace(source) => {
                let event = match source.selection_mode {
                    super::schedule::SelectionModeOptions::PopFront => 
                        engine.event_list_cache.front()
                }.unwrap();
                let trace = engine.generate_trace(event);
                engine.trace_cache.push_back(trace);
                
                match source.selection_mode {
                    super::schedule::SelectionModeOptions::PopFront => 
                        engine.event_list_cache.pop_front()
                }.unwrap();
            },
            Action::GenerateEventList(idx) => {
                let event_list = engine.generate_event_list(*idx);
                engine.event_list_cache.push_back(event_list);
            },
        }
    }
}
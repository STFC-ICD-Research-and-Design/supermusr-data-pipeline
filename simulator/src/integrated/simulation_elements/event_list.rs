use super::{noise::Noise, IntRandomDistribution};
use crate::integrated::{
    active_pulses::ActivePulses,
    simulation::Simulation,
    simulation_elements::{noise::NoiseSource, pulses::PulseEvent},
};
use rand_distr::WeightedIndex;
use serde::Deserialize;
use supermusr_common::{
    spanned::{SpanOnce, Spanned},
    FrameNumber, Intensity,
};
use tracing::{error, instrument};

pub(crate) struct TraceMetadata {
    expected_pulses: usize,
}

impl TraceMetadata {
    pub(crate) fn get_expected_pulses(&self) -> usize {
        self.expected_pulses
    }
}

pub(crate) struct Trace {
    span: SpanOnce,
    metadata: TraceMetadata,
    intensities: Vec<Intensity>,
}

impl Trace {
    #[instrument(
        skip_all,
        level = "debug",
        follows_from = [event_list
            .span()
            .get()
            .expect("Span should be initialised")
        ],
        target = "otel",
        name = "New Trace"
    )]
    pub(crate) fn new(
        simulation: &Simulation,
        frame_number: FrameNumber,
        event_list: &EventList<'_>,
    ) -> Self {
        let mut noise = event_list.noises.iter().map(Noise::new).collect::<Vec<_>>();
        let mut active_pulses = ActivePulses::new(&event_list.pulses);
        let sample_time = 1_000_000_000.0 / simulation.sample_rate as f64;
        Self {
            span: SpanOnce::Spanned(tracing::Span::current()),
            metadata: TraceMetadata {
                expected_pulses: event_list.pulses.len(),
            },
            intensities: (0..simulation.time_bins)
                .map(|time| {
                    //  Remove any expired muons
                    active_pulses.drop_spent_muons(time);
                    //  Append any new muons
                    active_pulses.push_new_muons(time);

                    //  Sum the signal of the currenty active muons
                    let signal = active_pulses
                        .iter()
                        .map(|p| p.get_value_at(time as f64 * sample_time))
                        .sum::<f64>();
                    noise.iter_mut().fold(signal, |signal, n| {
                        n.noisify(signal, time, frame_number as usize)
                    })
                })
                .map(|x: f64| simulation.voltage_transformation.transform(x) as Intensity)
                .collect(),
        }
    }

    pub(crate) fn get_metadata(&self) -> &TraceMetadata {
        &self.metadata
    }

    pub(crate) fn get_intensities(&self) -> &[Intensity] {
        &self.intensities
    }
}

impl Spanned for Trace {
    fn span(&self) -> &SpanOnce {
        &self.span
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "kebab-case", tag = "pulse-type")]
pub(crate) struct EventPulseTemplate {
    pub(crate) weight: f64,
    pub(crate) pulse_index: usize,
}

impl EventPulseTemplate {
    pub(crate) fn validate(&self, num_pulses: usize) -> bool {
        self.pulse_index < num_pulses
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct EventListTemplate {
    pub(crate) pulses: Vec<EventPulseTemplate>,
    pub(crate) noises: Vec<NoiseSource>,
    pub(crate) num_pulses: IntRandomDistribution,
}

impl EventListTemplate {
    pub(crate) fn validate(&self, num_pulse_attributes: usize) -> bool {
        for pulse in &self.pulses {
            if !pulse.validate(num_pulse_attributes) {
                error!("Pulse index too large");
                return false;
            }
        }
        true
    }
}

#[derive(Default)]
pub(crate) struct EventList<'a> {
    pub(crate) span: SpanOnce,
    pub(crate) pulses: Vec<PulseEvent>,
    pub(crate) noises: &'a [NoiseSource],
}

impl<'a> EventList<'a> {
    #[instrument(skip_all, level = "debug", target = "otel", "New Event List")]
    pub(crate) fn new(
        simulator: &Simulation,
        frame_number: FrameNumber,
        source: &'a EventListTemplate,
    ) -> Self {
        let distr = WeightedIndex::new(source.pulses.iter().map(|p| p.weight)).unwrap();
        let pulses = {
            // Creates a unique template for each channel
            let mut pulses = (0..source.num_pulses.sample(frame_number as usize) as usize)
                .map(|_| {
                    PulseEvent::sample(
                        simulator.get_random_pulse_template(source, &distr),
                        frame_number as usize,
                    )
                })
                .collect::<Vec<_>>();
            pulses.sort_by_key(|a| a.get_start());
            pulses
        };
        Self {
            span: SpanOnce::Spanned(tracing::Span::current()),
            pulses,
            noises: &source.noises,
        }
    }
}

impl Spanned for EventList<'_> {
    fn span(&self) -> &SpanOnce {
        &self.span
    }
}

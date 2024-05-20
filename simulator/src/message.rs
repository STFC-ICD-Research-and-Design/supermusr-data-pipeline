use crate::{
    muon_event::MuonEvent,
    noise::Noise,
    simulation_config::{NoiseSource, PulseAttributes, TraceMessage, Transformation},
};
use anyhow::Result;
use chrono::{DateTime, Utc};
use rand::{
    distributions::{Distribution, WeightedIndex},
    SeedableRng,
};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use std::time::Duration;
use supermusr_common::{Channel, DigitizerId, FrameNumber, Intensity, Time};
use supermusr_streaming_types::{
    dat2_digitizer_analog_trace_v2_generated::{
        finish_digitizer_analog_trace_message_buffer, ChannelTrace, ChannelTraceArgs,
        DigitizerAnalogTraceMessage, DigitizerAnalogTraceMessageArgs,
    },
    dev2_digitizer_event_v2_generated::{
        finish_digitizer_event_list_message_buffer, DigitizerEventListMessage,
        DigitizerEventListMessageArgs,
    },
    flatbuffers::FlatBufferBuilder,
    frame_metadata_v2_generated::{FrameMetadataV2, FrameMetadataV2Args, GpsTime},
};

impl<'a> TraceMessage {
    fn get_random_pulse_attributes(&self, distr: &WeightedIndex<f64>) -> &PulseAttributes {
        &self.pulses[distr.sample(&mut rand::rngs::StdRng::seed_from_u64(
            Utc::now().timestamp_subsec_nanos() as u64,
        ))]
        .attributes
    }

    pub(crate) fn create_frame_templates(
        &'a self,
        frame_index: usize,
        frame_number: FrameNumber,
        timestamp: &'a GpsTime,
    ) -> Result<Vec<TraceTemplate>> {
        let distr = WeightedIndex::new(self.pulses.iter().map(|p| p.weight))?;

        Ok(self
            .digitizers
            .iter()
            .map(|digitizer| {
                //  Unfortunately we can't clone these
                let metadata = FrameMetadataV2Args {
                    frame_number,
                    period_number: 0,
                    protons_per_pulse: 0,
                    running: true,
                    timestamp: Some(timestamp),
                    veto_flags: 0,
                };

                let channels = digitizer
                    .get_channels()
                    .map(|channel| {
                        // Creates a unique template for each channel
                        let pulses: Vec<_> = (0..self.num_pulses.sample(frame_index) as usize)
                            .map(|_| {
                                MuonEvent::sample(
                                    self.get_random_pulse_attributes(&distr),
                                    frame_index,
                                )
                            })
                            .collect();
                        (channel, pulses)
                    })
                    .collect();

                TraceTemplate {
                    frame_index,
                    time_bins: self.time_bins,
                    digitizer_id: digitizer.id,
                    sample_rate: self.sample_rate.unwrap_or(1_000_000_000),
                    metadata,
                    channels,
                    noises: &self.noises,
                }
            })
            .collect())
    }

    pub(crate) fn create_time_stamp(&self, now: &DateTime<Utc>, frame_index: usize) -> GpsTime {
        match self.timestamp {
            crate::simulation_config::Timestamp::Now => {
                *now + Duration::from_micros(frame_index as u64 * self.frame_delay_us)
            }
            crate::simulation_config::Timestamp::From(now) => {
                now + Duration::from_micros(frame_index as u64 * self.frame_delay_us)
            }
        }
        .into()
    }
}

pub(crate) struct TraceTemplate<'a> {
    frame_index: usize,
    digitizer_id: DigitizerId,
    time_bins: Time,
    sample_rate: u64,
    metadata: FrameMetadataV2Args<'a>,
    channels: Vec<(Channel, Vec<MuonEvent>)>,
    noises: &'a [NoiseSource],
}

impl TraceTemplate<'_> {
    fn generate_trace(
        &self,
        muons: &[MuonEvent],
        noise: &[NoiseSource],
        sample_time: f64,
        voltage_transformation: &Transformation<f64>,
    ) -> Vec<Intensity> {
        let mut noise = noise.iter().map(Noise::new).collect::<Vec<_>>();
        (0..self.time_bins)
            .map(|time| {
                let signal = muons
                    .iter()
                    .map(|p| p.get_value_at(time as f64 * sample_time))
                    .sum::<f64>();
                noise.iter_mut().fold(signal, |signal, n| {
                    n.noisify(signal, time, self.frame_index)
                })
            })
            .map(|x: f64| voltage_transformation.transform(x) as Intensity)
            .collect()
    }

    pub(crate) async fn send_trace_messages(
        &self,
        fbb: &mut FlatBufferBuilder<'_>,
        voltage_transformation: &Transformation<f64>,
    ) -> Result<()> {
        let sample_time = 1_000_000_000.0 / self.sample_rate as f64;
        let channels = self
            .channels
            .par_iter()
            .map(|(channel, pulses)| {
                //  This line creates the actual trace for the channel
                let trace =
                    self.generate_trace(pulses, self.noises, sample_time, voltage_transformation);
                (*channel, trace)
            })
            .collect::<Vec<_>>()
            .into_iter()
            .map(|(channel, trace)| {
                let voltage = Some(fbb.create_vector::<Intensity>(&trace));
                ChannelTrace::create(fbb, &ChannelTraceArgs { channel, voltage })
            })
            .collect::<Vec<_>>();

        let message = DigitizerAnalogTraceMessageArgs {
            digitizer_id: self.digitizer_id,
            metadata: Some(FrameMetadataV2::create(fbb, &self.metadata)),
            sample_rate: self.sample_rate,
            channels: Some(fbb.create_vector(&channels)),
        };
        let message = DigitizerAnalogTraceMessage::create(fbb, &message);
        finish_digitizer_analog_trace_message_buffer(fbb, message);

        Ok(())
    }

    pub(crate) async fn send_event_messages(
        &self,
        fbb: &mut FlatBufferBuilder<'_>,
        voltage_transformation: &Transformation<f64>,
    ) -> Result<()> {
        let sample_time_ns = 1_000_000_000.0 / self.sample_rate as f64;
        let mut channels = Vec::<Channel>::new();
        let mut time = Vec::<Time>::new();
        let mut voltage = Vec::<Intensity>::new();
        for (c, events) in &self.channels {
            for event in events {
                time.push((event.time() as f64 * sample_time_ns) as Time);
                voltage
                    .push(voltage_transformation.transform(event.intensity() as f64) as Intensity);
                channels.push(*c)
            }
        }

        let message = DigitizerEventListMessageArgs {
            digitizer_id: self.digitizer_id,
            metadata: Some(FrameMetadataV2::create(fbb, &self.metadata)),
            time: Some(fbb.create_vector(&time)),
            voltage: Some(fbb.create_vector(&voltage)),
            channel: Some(fbb.create_vector(&channels)),
        };
        let message = DigitizerEventListMessage::create(fbb, &message);
        finish_digitizer_event_list_message_buffer(fbb, message);
        Ok(())
    }

    pub(crate) fn metadata(&self) -> &FrameMetadataV2Args {
        &self.metadata
    }
}

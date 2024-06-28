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
use supermusr_common::{
    spanned::{SpanWrapper, Spanned},
    Channel, DigitizerId, FrameNumber, Intensity, Time,
};
use supermusr_streaming_types::{
    aev2_frame_assembled_event_v2_generated::{
        finish_frame_assembled_event_list_message_buffer, FrameAssembledEventListMessage,
        FrameAssembledEventListMessageArgs,
    },
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
use tracing::info_span;

impl<'a> TraceMessage {
    fn get_random_pulse_attributes(&self, distr: &WeightedIndex<f64>) -> &PulseAttributes {
        &self.pulses[distr.sample(&mut rand::rngs::StdRng::seed_from_u64(
            Utc::now().timestamp_subsec_nanos() as u64,
        ))]
        .attributes
    }

    fn create_metadata(
        frame_number: FrameNumber,
        timestamp: Option<&GpsTime>,
    ) -> FrameMetadataV2Args {
        FrameMetadataV2Args {
            frame_number,
            period_number: 0,
            protons_per_pulse: 0,
            running: true,
            timestamp,
            veto_flags: 0,
        }
    }

    fn create_pulses(&self, frame_index: usize, distr: &WeightedIndex<f64>) -> Vec<MuonEvent> {
        // Creates a unique template for each channel
        (0..self.num_pulses.sample(frame_index) as usize)
            .map(|_| MuonEvent::sample(self.get_random_pulse_attributes(distr), frame_index))
            .collect::<Vec<_>>()
    }

    fn create_aggregated_template(
        &'a self,
        frame_index: usize,
        metadata: FrameMetadataV2Args<'a>,
        channels: Vec<(u32, Vec<MuonEvent>)>,
    ) -> TraceTemplate<'a> {
        TraceTemplate {
            frame_index,
            time_bins: self.time_bins,
            digitizer_id: None,
            sample_rate: self.sample_rate.unwrap_or(1_000_000_000),
            metadata,
            channels,
            noises: &self.noises,
        }
    }

    fn create_digitiser_template(
        &'a self,
        frame_index: usize,
        digitizer_id: DigitizerId,
        metadata: FrameMetadataV2Args<'a>,
        channels: Vec<(u32, Vec<MuonEvent>)>,
    ) -> TraceTemplate<'a> {
        TraceTemplate {
            frame_index,
            time_bins: self.time_bins,
            digitizer_id: Some(digitizer_id),
            sample_rate: self.sample_rate.unwrap_or(1_000_000_000),
            metadata,
            channels,
            noises: &self.noises,
        }
    }

    #[tracing::instrument(skip_all, fields(frame_number = frame_number))]
    pub(crate) fn create_frame_templates(
        &'a self,
        frame_index: usize,
        frame_number: FrameNumber,
        timestamp: &'a GpsTime,
    ) -> Result<Vec<TraceTemplate>> {
        let distr = WeightedIndex::new(self.pulses.iter().map(|p| p.weight))?;
        match &self.source_type {
            crate::simulation_config::SourceType::AggregatedFrame(aggregated_frame) => {
                //  Unfortunately we can't clone these
                let metadata = Self::create_metadata(frame_number, Some(timestamp));

                let channels = aggregated_frame
                    .get_channels()
                    .map(|channel| (channel, self.create_pulses(frame_index, &distr)))
                    .collect();

                Ok(vec![self.create_aggregated_template(
                    frame_index,
                    metadata,
                    channels,
                )])
            }
            crate::simulation_config::SourceType::ChannelsByDigitisers(channels_by_digitisers) => {
                Ok((0..channels_by_digitisers.num_digitisers)
                    .map(|digitizer_index| {
                        //  Unfortunately we can't clone these
                        let metadata = Self::create_metadata(frame_number, Some(timestamp));

                        let digitizer_id = digitizer_index as DigitizerId;
                        let channels = (0..channels_by_digitisers.channels_per_digitiser)
                            .map(|channel_index| {
                                (
                                    (channel_index
                                        + digitizer_index
                                            * channels_by_digitisers.channels_per_digitiser)
                                        as Channel,
                                    self.create_pulses(frame_index, &distr),
                                )
                            })
                            .collect::<Vec<_>>();

                        self.create_digitiser_template(
                            frame_index,
                            digitizer_id,
                            metadata,
                            channels,
                        )
                    })
                    .collect())
            }
            crate::simulation_config::SourceType::Digitisers(digitisers) => Ok(digitisers
                .iter()
                .map(|digitizer| {
                    //  Unfortunately we can't clone these
                    let metadata = Self::create_metadata(frame_number, Some(timestamp));

                    let channels = digitizer
                        .get_channels()
                        .map(|channel| (channel, self.create_pulses(frame_index, &distr)))
                        .collect();

                    self.create_digitiser_template(frame_index, digitizer.id, metadata, channels)
                })
                .collect()),
        }
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
    digitizer_id: Option<DigitizerId>,
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

    #[tracing::instrument(skip_all, target = "otel", fields(digitizer_id = digitizer_id))]
    pub(crate) async fn send_trace_messages(
        &self,
        fbb: &mut FlatBufferBuilder<'_>,
        digitizer_id: DigitizerId,
        voltage_transformation: &Transformation<f64>,
    ) -> Result<()> {
        let sample_time = 1_000_000_000.0 / self.sample_rate as f64;
        let channels = self
            .channels
            .iter()
            .map(SpanWrapper::<_>::new_with_current)
            .collect::<Vec<_>>()
            .par_iter()
            .map(|spanned_channel_pulses| {
                let channel = spanned_channel_pulses.0;
                let pulses : &[MuonEvent] = spanned_channel_pulses.1.as_ref();
                let channel_span = spanned_channel_pulses.span().get()
                    .expect("Channel has span");
                channel_span.in_scope(|| {
                    let _guard = info_span!(target: "otel", "Channel", channel = channel, num_pulses = pulses.len()).entered();
                    //  This line creates the actual trace for the channel
                    let trace =
                        self.generate_trace(pulses, self.noises, sample_time, voltage_transformation);
                    (channel, trace)
                })
            })
            .collect::<Vec<_>>()
            .into_iter()
            .map(|(channel, trace)| {
                let voltage = Some(fbb.create_vector::<Intensity>(&trace));
                ChannelTrace::create(fbb, &ChannelTraceArgs { channel, voltage })
            })
            .collect::<Vec<_>>();

        let message = DigitizerAnalogTraceMessageArgs {
            digitizer_id,
            metadata: Some(FrameMetadataV2::create(fbb, &self.metadata)),
            sample_rate: self.sample_rate,
            channels: Some(fbb.create_vector(&channels)),
        };
        let message = DigitizerAnalogTraceMessage::create(fbb, &message);
        finish_digitizer_analog_trace_message_buffer(fbb, message);

        Ok(())
    }

    pub(crate) async fn send_digitiser_event_messages(
        &self,
        fbb: &mut FlatBufferBuilder<'_>,
        digitizer_id: DigitizerId,
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
            digitizer_id,
            metadata: Some(FrameMetadataV2::create(fbb, &self.metadata)),
            time: Some(fbb.create_vector(&time)),
            voltage: Some(fbb.create_vector(&voltage)),
            channel: Some(fbb.create_vector(&channels)),
        };
        let message = DigitizerEventListMessage::create(fbb, &message);
        finish_digitizer_event_list_message_buffer(fbb, message);
        Ok(())
    }

    pub(crate) async fn send_frame_event_messages(
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

        let message = FrameAssembledEventListMessageArgs {
            metadata: Some(FrameMetadataV2::create(fbb, &self.metadata)),
            time: Some(fbb.create_vector(&time)),
            voltage: Some(fbb.create_vector(&voltage)),
            channel: Some(fbb.create_vector(&channels)),
        };
        let message = FrameAssembledEventListMessage::create(fbb, &message);
        finish_frame_assembled_event_list_message_buffer(fbb, message);
        Ok(())
    }

    pub(crate) fn digitizer_id(&self) -> Option<DigitizerId> {
        self.digitizer_id
    }

    pub(crate) fn metadata(&self) -> &FrameMetadataV2Args {
        &self.metadata
    }

    // This is currently only called by the test suite
    #[cfg(test)]
    pub(crate) fn channels(&self) -> &[(u32, Vec<MuonEvent>)] {
        &self.channels
    }
}

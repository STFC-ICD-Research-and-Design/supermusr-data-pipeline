use anyhow::Result;
use chrono::{DateTime, Utc};
use rand::{distributions::{Distribution, WeightedIndex}, SeedableRng};
use rdkafka::{
    producer::{FutureProducer, FutureRecord},
    util::Timeout
};
use std::time::Duration;
use supermusr_common::{Channel, DigitizerId, FrameNumber, Intensity, Time};
use supermusr_streaming_types::{
    dat1_digitizer_analog_trace_v1_generated::{
        finish_digitizer_analog_trace_message_buffer, ChannelTrace, ChannelTraceArgs,
        DigitizerAnalogTraceMessage, DigitizerAnalogTraceMessageArgs,
    },
    dev1_digitizer_event_v1_generated::{
        finish_digitizer_event_list_message_buffer, DigitizerEventListMessage,
        DigitizerEventListMessageArgs,
    },
    flatbuffers::FlatBufferBuilder,
    frame_metadata_v1_generated::{FrameMetadataV1, FrameMetadataV1Args, GpsTime},
};

use crate::{json::NoiseSource, muon::Muon, noise::Noise};
use crate::json::{Transformation, PulseAttributes, TraceMessage};

impl<'a> TraceMessage {
    fn get_random_pulse_attributes(&self, distr: &WeightedIndex<f64>) -> &PulseAttributes {
        &self.pulses[distr.sample(&mut rand::rngs::StdRng::seed_from_u64(Utc::now().timestamp_subsec_nanos() as u64))].attributes
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
                let metadata = FrameMetadataV1Args {
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
                        let pulses : Vec<_> = (0..self.num_pulses.sample(frame_index) as usize)
                            .map(|_| Muon::sample(self.get_random_pulse_attributes(&distr),frame_index))
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
                    noises: &self.noises
                }
            })
            .collect())
    }

    pub(crate) fn create_time_stamp(&self, now: &DateTime<Utc>, frame_index: usize) -> GpsTime {
        match self.timestamp {
            crate::json::Timestamp::Now => {
                *now + Duration::from_micros(frame_index as u64 * self.frame_delay_us)
            }
            crate::json::Timestamp::From(now) => 
                now + Duration::from_micros(frame_index as u64 * self.frame_delay_us)
        }
        .into()
    }
}



pub(crate) struct TraceTemplate<'a> {
    frame_index : usize,
    digitizer_id: DigitizerId,
    time_bins: Time,
    sample_rate: u64,
    metadata: FrameMetadataV1Args<'a>,
    channels: Vec<(Channel, Vec<Muon>)>,
    noises: &'a [NoiseSource]
}

impl TraceTemplate<'_> {
    fn generate_trace(&self, muons: &[Muon], noise: &mut [Noise], voltage_transformation : &Transformation<f64>) -> Vec<Intensity> {
        (0..self.time_bins)
            .map(|time| {
                let signal = muons.iter()
                    .map(|p| p.get_value_at(time))
                    .sum::<f64>();
                noise.iter_mut()
                    .fold(signal, |signal, n|
                        n.noisify(signal, time, self.frame_index)
                    )
            })
            .map(|x: f64| voltage_transformation.transform(x) as Intensity)
            .collect()
    }

    pub(crate) async fn send_trace_messages(
        &self,
        producer: &FutureProducer,
        fbb: &mut FlatBufferBuilder<'_>,
        topic: &str,
        voltage_transformation: &Transformation<f64>,
    ) -> Result<()> {
        let channels = self
            .channels
            .iter()
            .map(|(channel, pulses)| {
                //  This line creates the actual trace for the channel
                let mut noises = self.noises.iter().map(Noise::new).collect::<Vec<_>>();
                let trace = self.generate_trace(pulses, &mut noises, voltage_transformation);
                let channel = *channel;
                let voltage = Some(fbb.create_vector::<Intensity>(&trace));
                ChannelTrace::create(fbb, &ChannelTraceArgs { channel, voltage })
            })
            .collect::<Vec<_>>();

        let message = DigitizerAnalogTraceMessageArgs {
            digitizer_id: self.digitizer_id,
            metadata: Some(FrameMetadataV1::create(fbb, &self.metadata)),
            sample_rate: self.sample_rate,
            channels: Some(fbb.create_vector(&channels)),
        };
        let message = DigitizerAnalogTraceMessage::create(fbb, &message);
        finish_digitizer_analog_trace_message_buffer(fbb, message);

        match producer
            .send(
                FutureRecord::to(topic)
                    .payload(fbb.finished_data())
                    .key(&"todo".to_string()),
                Timeout::After(Duration::from_millis(100)),
            )
            .await
        {
            Ok(r) => log::debug!("Delivery: {:?}", r),
            Err(e) => log::error!("Delivery failed: {:?}", e),
        };

        /*log::info!(
            "Event send took: {:?}",
            SystemTime::now().duration_since(start_time).unwrap()
        );*/
        Ok(())
    }

    pub(crate) async fn send_event_messages(
        &self,
        producer: &FutureProducer,
        fbb: &mut FlatBufferBuilder<'_>,
        topic: &str,
        voltage_transformation: &Transformation<f64>,
    ) -> Result<()> {
        let sample_time_ns = 1_000_000_000.0 / self.sample_rate as f64;
        let mut channel = Vec::<Channel>::new();
        let mut time = Vec::<Time>::new();
        let mut voltage = Vec::<Intensity>::new();
        for (c, events) in &self.channels {
            for event in events {
                time.push((event.time() as f64 * sample_time_ns) as Time);
                voltage.push(voltage_transformation.transform(event.intensity() as f64) as Intensity);
                channel.push(*c)
            }
        }

        let message = DigitizerEventListMessageArgs {
            digitizer_id: self.digitizer_id,
            metadata: Some(FrameMetadataV1::create(fbb, &self.metadata)),
            time: Some(fbb.create_vector(&time)),
            voltage: Some(fbb.create_vector(&voltage)),
            channel: Some(fbb.create_vector(&channel)),
        };
        let message = DigitizerEventListMessage::create(fbb, &message);
        finish_digitizer_event_list_message_buffer(fbb, message);

        match producer
            .send(
                FutureRecord::to(topic)
                    .payload(fbb.finished_data())
                    .key(&"todo".to_string()),
                Timeout::After(Duration::from_millis(100)),
            )
            .await
        {
            Ok(r) => log::debug!("Delivery: {:?}", r),
            Err(e) => log::error!("Delivery failed: {:?}", e),
        };

        /*log::info!(
            "Event send took: {:?}",
            SystemTime::now().duration_since(start_time).unwrap()
        );*/
        Ok(())
    }
}

use std::time::Duration;
use std::time::SystemTime;

use anyhow::Result;
use rand::distributions::{Distribution,WeightedIndex};
use rdkafka::producer::{FutureProducer,FutureRecord};
use rdkafka::util::Timeout;
use supermusr_common::{Channel, DigitizerId, FrameNumber, Intensity, Time};
use supermusr_streaming_types::{
    dev1_digitizer_event_v1_generated::{
        finish_digitizer_event_list_message_buffer,
        DigitizerEventListMessage,
        DigitizerEventListMessageArgs
    },
    dat1_digitizer_analog_trace_v1_generated::{
        finish_digitizer_analog_trace_message_buffer,
        ChannelTrace,
        ChannelTraceArgs,
        DigitizerAnalogTraceMessage,
        DigitizerAnalogTraceMessageArgs
    },
    flatbuffers::FlatBufferBuilder,
    frame_metadata_v1_generated::{
        FrameMetadataV1,
        FrameMetadataV1Args,
        GpsTime
    }
};

use super::json;
use super::channel_trace;


impl<'a> json::TraceMessage {
    pub(crate) fn create_frame_templates(&'a self, frame_number: FrameNumber, timestamp: &'a GpsTime) -> Result<Vec<TraceTemplate>> {
        let distr = WeightedIndex::new(self.pulses.iter().map(|p|p.weight))?;
        let mut templates = Vec::<TraceTemplate>::new();
        for digitizer in &self.digitizers {
            let start_time = SystemTime::now();

            let metadata = FrameMetadataV1Args {
                frame_number,
                period_number: 0,
                protons_per_pulse: 0,
                running: true,
                timestamp: Some(timestamp),
                veto_flags: 0,
            };   
            let channels = (digitizer.channels.min..digitizer.channels.max).map(|channel| {
                let pulses = (0..self.num_pulses.sample())
                .map(|_| channel_trace::Pulse::sample(
                    &self.pulses[distr.sample(&mut rand::thread_rng())].attributes
                )).collect();
                (channel, pulses)
            })
            .collect();
            templates.push(TraceTemplate {
                time_bins: self.time_bins,
                digitizer_id: digitizer.id,
                metadata,
                channels
            });
        }
        Ok(templates)
    }
}

pub(crate) struct TraceTemplate<'a> {
    digitizer_id: DigitizerId,
    time_bins: Time,
    metadata: FrameMetadataV1Args<'a>,
    channels: Vec<(Channel, Vec<channel_trace::Pulse>)>
}

impl TraceTemplate<'_> {
    pub(crate) async fn send_trace_messages(&self, producer: &FutureProducer, fbb: &mut FlatBufferBuilder<'_>, topic: &str) -> Result<()> {
        let channels = self.channels.iter().map(|(channel,v)| {
            let voltage = Some(fbb.create_vector::<Intensity>(&channel_trace::generate_trace(self.time_bins, v, &[].to_vec())));
            ChannelTrace::create(fbb, &ChannelTraceArgs { channel: *channel, voltage})
        })
        .collect::<Vec<_>>();

        let message = DigitizerAnalogTraceMessageArgs {
            digitizer_id: self.digitizer_id,
            metadata: Some(FrameMetadataV1::create(fbb, &self.metadata)),
            sample_rate: 1_000_000_000,
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
        .await {
            Ok(r) => log::debug!("Delivery: {:?}", r),
            Err(e) => log::error!("Delivery failed: {:?}", e),
        };

        /*log::info!(
            "Event send took: {:?}",
            SystemTime::now().duration_since(start_time).unwrap()
        );*/
        Ok(())
    }
    
    pub(crate) async fn send_event_messages(&self, producer: &FutureProducer, fbb: &mut FlatBufferBuilder<'_>, topic: &str) -> Result<()> {
        let mut channel = Vec::<Channel>::new();
        let mut time = Vec::<Time>::new();
        let mut voltage = Vec::<Intensity>::new();
        for (c,events) in &self.channels {
            for event in events {
                time.push(event.time());
                voltage.push(event.intensity());
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
        .await {
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

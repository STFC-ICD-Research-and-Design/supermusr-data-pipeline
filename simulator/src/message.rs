use std::time::SystemTime;

use anyhow::Result;
use chrono::Utc;
use rand::distributions::Distribution;
use rand::distributions::WeightedIndex;
use supermusr_streaming_types::dat1_digitizer_analog_trace_v1_generated::finish_digitizer_analog_trace_message_buffer;
use supermusr_streaming_types::dat1_digitizer_analog_trace_v1_generated::ChannelTrace;
use supermusr_streaming_types::dat1_digitizer_analog_trace_v1_generated::ChannelTraceArgs;
use supermusr_streaming_types::dat1_digitizer_analog_trace_v1_generated::DigitizerAnalogTraceMessage;
use supermusr_streaming_types::dat1_digitizer_analog_trace_v1_generated::DigitizerAnalogTraceMessageArgs;
use supermusr_streaming_types::flatbuffers::FlatBufferBuilder;
use supermusr_streaming_types::frame_metadata_v1_generated::FrameMetadataV1;
use supermusr_streaming_types::frame_metadata_v1_generated::GpsTime;

use super::json;
use super::channel_trace;

impl json::Simulation {
    pub(crate) fn generate_trace_messages(&self) -> Result<Vec<DigitizerAnalogTraceMessage>> {
        let vec = self.traces
            .iter()
            .map(|trace|trace.generate_trace_messages())
            .collect::<Result<Vec<_>>>()?
            .into_iter()
            .flatten()
            .collect();
        Ok(vec)
    }
}
impl json::TraceMessage {
    pub(crate) fn generate_trace_messages(&self, fbb: &mut FlatBufferBuilder<'_>) -> Result<Vec<DigitizerAnalogTraceMessage>> {
        let mut list = Vec::<DigitizerAnalogTraceMessage>::with_capacity(self.frames.len()*self.digitizer_ids.len());
        let distr = WeightedIndex::new(self.pulses.iter().map(|p|p.weight))?;
        for frame in self.frames {
            for digitzer in self.digitizer {
                let start_time = SystemTime::now();
                fbb.reset();
        
                let time: GpsTime = Utc::now().into();

                let metadata = FrameMetadataV1Args {
                    frame_number: frame,
                    period_number: 0,
                    protons_per_pulse: 0,
                    running: true,
                    timestamp: Some(&time),
                    veto_flags: 0,
                };
                let metadata = FrameMetadataV1::create(fbb, &metadata);
                
                let voltages = (digitzer.channels.min..digitzer.channels.max).map(|channel| {
                    let pulses = (0..self.num_pulses.sample())
                    .map(|_|
                        channel_trace::Pulse::sample(&self.pulses[distr.sample(&mut rand::thread_rng())].attributes)
                    )
                    .collect();
                    let trace = channel_trace::generate_trace(self.time_bins, pulses, [].to_vec());
                    (channel, trace)
                });
                let channels = voltages.map(|c,v| ChannelTrace::create(
                    fbb,
                    &ChannelTraceArgs {
                        channel: c,
                        voltage: v,
                    },
                ))
                .collect();
        
                let message = DigitizerAnalogTraceMessageArgs {
                    digitizer_id: digitzer.id,
                    metadata: Some(metadata),
                    sample_rate: 1_000_000_000,
                    channels: Some(fbb.create_vector(&channels)),
                };
                let message = DigitizerAnalogTraceMessage::create(fbb, &message);
                finish_digitizer_analog_trace_message_buffer(fbb, message);
        
                
                list.push();    
            }
        }
        list
    }
}

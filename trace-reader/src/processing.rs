//! This module allows one to simulate instances of DigitizerAnalogTraceMessage
//! using the FlatBufferBuilder.

use super::loader::{TraceFile, TraceFileEvent};
use anyhow::{Error, Result};
use chrono::Utc;
use log::{debug, error};
use rand::Rng;
use rdkafka::{
    producer::{FutureProducer, FutureRecord},
    util::Timeout,
};
use std::{ops::RangeInclusive, time::Duration};

use common::{Channel, DigitizerId, FrameNumber, Intensity};
use streaming_types::{
    dat1_digitizer_analog_trace_v1_generated::{
        finish_digitizer_analog_trace_message_buffer, ChannelTrace, ChannelTraceArgs,
        DigitizerAnalogTraceMessage, DigitizerAnalogTraceMessageArgs,
    },
    flatbuffers::{FlatBufferBuilder, WIPOffset},
    frame_metadata_v1_generated::{FrameMetadataV1, FrameMetadataV1Args, GpsTime},
};

/// Reads the contents of trace_file and dispatches messages to the given Kafka topic.
pub(crate) async fn dispatch_trace_file(
    mut trace_file: TraceFile,
    trace_event_indices: Vec<usize>,
    producer: &FutureProducer,
    topic: &str,
    timeout_ms: u64,
) -> Result<()> {
    let mut fbb = FlatBufferBuilder::new();
    for index in trace_event_indices {
        let event = trace_file.get_trace_event(index)?;
        create_partly_random_message(
            &mut fbb,
            Utc::now().into(),
            1..=10,
            1..=10,
            trace_file.get_num_channels(),
            trace_file.get_num_samples(),
            &event,
        )?;

        let future_record = FutureRecord::to(topic).payload(fbb.finished_data()).key("");
        let timeout = Timeout::After(Duration::from_millis(timeout_ms));
        match producer.send(future_record, timeout).await {
            Ok(r) => debug!("Delivery: {:?}", r),
            Err(e) => error!("Delivery failed: {:?}", e.0),
        };
    }
    Ok(())
}

pub fn create_channel<'a>(
    fbb: &mut FlatBufferBuilder<'a>,
    channel: Channel,
    trace: &[Intensity],
) -> WIPOffset<ChannelTrace<'a>> {
    let voltage = Some(fbb.create_vector::<Intensity>(trace));
    ChannelTrace::create(fbb, &ChannelTraceArgs { channel, voltage })
}

/// Loads a FlatBufferBuilder with a new DigitizerAnalogTraceMessage instance with a custom timestamp.
/// #Arguments
/// * `fbb` - A mutable reference to the FlatBufferBuilder to use.
/// * `time` - A `frame_metadata_v1_generated::GpsTime` instance containing the timestamp.
/// * `frame_number` - The frame number to use.
/// * `digitizer_id` - The id of the digitizer to use.
/// * `measurements_per_frame` - The number of measurements to simulate in each channel.
/// * `num_channels` - The number of channels to simulate.
/// #Returns
/// A string result, or an error.
pub fn create_message(
    fbb: &mut FlatBufferBuilder<'_>,
    time: GpsTime,
    frame_number: u32,
    digitizer_id: u8,
    number_of_channels: usize,
    number_of_samples: usize,
    event: &TraceFileEvent,
) -> Result<String, Error> {
    fbb.reset();

    let metadata: FrameMetadataV1Args = FrameMetadataV1Args {
        frame_number,
        period_number: 0,
        protons_per_pulse: 0,
        running: true,
        timestamp: Some(&time),
        veto_flags: 0,
    };
    let metadata: WIPOffset<FrameMetadataV1> = FrameMetadataV1::create(fbb, &metadata);

    let channels: Vec<_> = (0..number_of_channels)
        .map(|c| create_channel(fbb, c as u32, event.raw_trace[c].as_slice()))
        .collect();

    let message = DigitizerAnalogTraceMessageArgs {
        digitizer_id,
        metadata: Some(metadata),
        sample_rate: 1_000_000_000,
        channels: Some(fbb.create_vector_from_iter(channels.iter())),
    };
    let message = DigitizerAnalogTraceMessage::create(fbb, &message);
    finish_digitizer_analog_trace_message_buffer(fbb, message);

    Ok(format!("New message created for digitizer {digitizer_id}, frame number {frame_number}, and has {number_of_channels} channels, and {number_of_samples} measurements."))
}

/// Loads a FlatBufferBuilder with a new DigitizerAnalogTraceMessage instance with a custom timestamp,
/// and a random frame number and digitizer id.
/// #Arguments
/// * `fbb` - A mutable reference to the FlatBufferBuilder to use.
/// * `time` - A `frame_metadata_v1_generated::GpsTime` instance containing the timestamp.
/// * `frame_number` - The upper and lower bounds from which to sample the frame number from.
/// * `digitizer_id` - The upper and lower bounds from which to sample the digitizer id from.
/// * `measurements_per_frame` - The number of measurements to simulate in each channel.
/// * `num_channels` - The number of channels to simulate.
/// #Returns
/// A string result, or an error.
pub fn create_partly_random_message(
    fbb: &mut FlatBufferBuilder<'_>,
    time: GpsTime,
    frame_number: RangeInclusive<FrameNumber>,
    digitizer_id: RangeInclusive<DigitizerId>,
    number_of_channels: usize,
    number_of_samples: usize,
    event: &TraceFileEvent,
) -> Result<String, Error> {
    let mut rng = rand::thread_rng();
    let frame_number = rng.gen_range(frame_number);
    let digitizer_id = rng.gen_range(digitizer_id);
    create_message(
        fbb,
        time,
        frame_number,
        digitizer_id,
        number_of_channels,
        number_of_samples,
        event,
    )
}

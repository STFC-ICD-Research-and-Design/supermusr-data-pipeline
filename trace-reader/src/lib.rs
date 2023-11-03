//! This module allows one to simulate instances of DigitizerAnalogTraceMessage
//! using the FlatBufferBuilder.
//!

use anyhow::{Error, Result};
//use std::ops::Range;
use chrono::Utc;
use loader::TraceFileEvent;

use flatbuffers::{FlatBufferBuilder, WIPOffset};
use rand::Rng;
use rdkafka::{
    producer::{FutureProducer, FutureRecord},
    util::Timeout,
};
use std::{ops::RangeInclusive, time::Duration};

pub mod loader;
pub use loader::{load_trace_file, TraceFile};

use common::{Channel, DigitizerId, FrameNumber, Intensity};
use streaming_types::{
    dat1_digitizer_analog_trace_v1_generated::{
        finish_digitizer_analog_trace_message_buffer, ChannelTrace, ChannelTraceArgs,
        DigitizerAnalogTraceMessage, DigitizerAnalogTraceMessageArgs,
    },
    frame_metadata_v1_generated::{FrameMetadataV1, FrameMetadataV1Args, GpsTime},
};

pub async fn dispatch_trace_file(
    mut trace_file: TraceFile,
    events: Vec<usize>,
    producer: &FutureProducer,
    topic: &str,
    timeout_ms: u64,
) -> Result<()> {
    let mut fbb = FlatBufferBuilder::new();
    for event_idx in events {
        let event = trace_file.get_event(event_idx)?;
        create_partly_random_message_with_now(
            &mut fbb,
            1..=10,
            1..=10,
            trace_file.get_num_channels(),
            trace_file.get_num_samples(),
            &event,
        )?;

        let future_record = FutureRecord::to(topic).payload(fbb.finished_data()).key("");
        let timeout = Timeout::After(Duration::from_millis(timeout_ms));
        match producer.send(future_record, timeout).await {
            Ok(r) => log::debug!("Delivery: {:?}", r),
            Err(e) => log::error!("Delivery failed: {:?}", e.0),
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

/// Loads a FlatBufferBuilder with a new DigitizerAnalogTraceMessage instance with the present timestamp.
/// #Arguments
/// * `fbb` - A mutable reference to the FlatBufferBuilder to use.
/// * `frame_number` - The frame number to use.
/// * `digitizer_id` - The id of the digitizer to use.
/// * `measurements_per_frame` - The number of measurements to simulate in each channel.
/// * `num_channels` - The number of channels to simulate.
/// #Returns
/// A string result, or an error.
pub fn create_message_with_now(
    fbb: &mut FlatBufferBuilder<'_>,
    frame_number: u32,
    digitizer_id: u8,
    number_of_channels: usize,
    number_of_samples: usize,
    event: &TraceFileEvent,
) -> Result<String, Error> {
    let time: GpsTime = Utc::now().into();
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

/// Loads a FlatBufferBuilder with a new DigitizerAnalogTraceMessage instance with the present timestamp,
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
pub fn create_partly_random_message_with_now(
    fbb: &mut FlatBufferBuilder<'_>,
    frame_number: RangeInclusive<FrameNumber>,
    digitizer_id: RangeInclusive<DigitizerId>,
    number_of_channels: usize,
    number_of_samples: usize,
    event: &TraceFileEvent,
) -> Result<String, Error> {
    let time: GpsTime = Utc::now().into();
    create_partly_random_message(
        fbb,
        time,
        frame_number,
        digitizer_id,
        number_of_channels,
        number_of_samples,
        event,
    )
}

#[cfg(test)]
mod test {
    use super::*;
    use flatbuffers::FlatBufferBuilder;
    use std::ops::RangeInclusive;
    use streaming_types::{
        dat1_digitizer_analog_trace_v1_generated::root_as_digitizer_analog_trace_message,
        frame_metadata_v1_generated::GpsTime,
    };

    #[test]
    fn test_basic() {
        let timestamp = GpsTime::new(22, 205, 10, 55, 30, 0, 1, 5);
        let digitizer_id = 2;
        let frame_number = 495;
        let num_time_points = 20;
        let num_channels = 4;

        //et mut fbb = FlatBufferBuilder::new();
        /*let string = create_message(
            &mut fbb,
            timestamp,
            frame_number,
            digitizer_id,
            num_time_points,
            num_channels,
        )
        .unwrap();
        assert_eq!(string, format!("New message created for digitizer {digitizer_id}, frame number {frame_number}, and has {num_channels} channels, and {num_time_points} measurements."));
        let msg = root_as_digitizer_analog_trace_message(fbb.finished_data()).unwrap();

        assert_eq!(msg.digitizer_id(), digitizer_id);

        assert!(msg.metadata().timestamp().is_some());
        /*assert_eq!(msg.metadata().timestamp().timestamp_seconds(), 0);
        assert_eq!(msg.metadata().timestamp().timestamp_millis(), 0);
        assert_eq!(msg.metadata().timestamp().timestamp_micros(), 0);
        assert_eq!(msg.metadata().timestamp().timestamp_nanos(), 0);*/

        assert_eq!(msg.metadata().frame_number(), frame_number);

        assert!(msg.channels().is_some());
        assert_eq!(msg.channels().unwrap().len(), num_channels);
        let channels = msg.channels().unwrap();
        for (i, c) in channels.iter().enumerate() {
            assert_eq!(c.channel(), i as Channel);
            assert!(c.voltage().is_some());
            assert_eq!(c.voltage().unwrap().len(), num_time_points);
        }*/
    }
}

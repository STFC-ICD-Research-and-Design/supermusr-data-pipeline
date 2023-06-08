//! This module allows one to simulate instances of DigitizerAnalogTraceMessage
//! using the FlatBufferBuilder.
//! 

use anyhow::Error;
//use std::ops::Range;
use rand::{random, Rng};
use core::ops::Range;
use std::ops::RangeInclusive;
use chrono::Utc;
use flatbuffers::{FlatBufferBuilder, WIPOffset};

use common::{Channel, Intensity, DigitizerId, FrameNumber};
use streaming_types::{
    dat1_digitizer_analog_trace_v1_generated::{
        finish_digitizer_analog_trace_message_buffer, ChannelTrace, ChannelTraceArgs,
        DigitizerAnalogTraceMessage, DigitizerAnalogTraceMessageArgs,
    },
    frame_metadata_v1_generated::{FrameMetadataV1, FrameMetadataV1Args, GpsTime},
};

fn create_channel<'a>(fbb : &mut FlatBufferBuilder<'a>, channel : Channel, measurements_per_frame : usize) -> WIPOffset<ChannelTrace<'a>> {
    let items : Vec<Intensity> = (0..measurements_per_frame).into_iter()
        .map(|_|random::<Intensity>())
        .collect();
    let voltage = Some(fbb.create_vector::<Intensity>(&items));
    ChannelTrace::create(fbb,&ChannelTraceArgs {channel,voltage,},)
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
pub fn create_message_with_now(fbb : &mut FlatBufferBuilder<'_>,
        frame_number: u32,
        digitizer_id : u8,
        measurements_per_frame : usize,
        num_channels : usize) -> Result<String,Error> {
    let time : GpsTime = Utc::now().into();
    create_message(fbb, time, frame_number, digitizer_id, measurements_per_frame, num_channels)
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
pub fn create_message(fbb : &mut FlatBufferBuilder<'_>,
        time : GpsTime,
        frame_number: u32,
        digitizer_id : u8,
        measurements_per_frame : usize,
        num_channels : usize) -> Result<String,Error> {
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

    let channels : Vec<_> = (0..num_channels).into_iter().map(|c|create_channel(fbb,c as u32, measurements_per_frame)).collect();

    let message = DigitizerAnalogTraceMessageArgs {
        digitizer_id: digitizer_id,
        metadata: Some(metadata),
        sample_rate: 1_000_000_000,
        channels: Some(fbb.create_vector_from_iter(channels.iter())),
    };
    let message = DigitizerAnalogTraceMessage::create(fbb, &message);
    finish_digitizer_analog_trace_message_buffer(fbb, message);

    Ok(format!("New message created for digitizer {0}, frame number {1}, and has {2} channels, and {3} measurements.",digitizer_id, frame_number,num_channels,measurements_per_frame))
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
pub fn create_partly_random_message(fbb : &mut FlatBufferBuilder<'_>,
        time : GpsTime,
        frame_number: RangeInclusive<FrameNumber>,
        digitizer_id : RangeInclusive<DigitizerId>,
        measurements_per_frame : usize,
        num_channels : usize) -> Result<String,Error> {
    let mut rng = rand::thread_rng();
    let frame_number = rng.gen_range(frame_number);
    let digitizer_id = rng.gen_range(digitizer_id);
    create_message(fbb, time, frame_number, digitizer_id, measurements_per_frame, num_channels)
}

/// Loads a FlatBufferBuilder with a new DigitizerAnalogTraceMessage instance with a custom timestamp,
/// and all random parameters.
/// #Arguments
/// * `fbb` - A mutable reference to the FlatBufferBuilder to use.
/// * `time` - A `frame_metadata_v1_generated::GpsTime` instance containing the timestamp.
/// * `frame_number` - The upper and lower bounds from which to sample the frame number from.
/// * `digitizer_id` - The upper and lower bounds from which to sample the digitizer id from.
/// * `measurements_per_frame` - The upper and lower bounds from which to sample the number of measurements from.
/// * `num_channels` - The upper and lower bounds from which to sample the number of channels from.
/// #Returns
/// A string result, or an error.
pub fn create_random_message(fbb : &mut FlatBufferBuilder<'_>,
        time : GpsTime,
        frame_number: RangeInclusive<FrameNumber>,
        digitizer_id : RangeInclusive<DigitizerId>,
        measurements_per_frame : RangeInclusive<usize>,
        num_channels : RangeInclusive<usize>) -> Result<String,Error> {
    let mut rng = rand::thread_rng();
    let measurements_per_frame = rng.gen_range(measurements_per_frame);
    let num_channels = rng.gen_range(num_channels);
    create_partly_random_message(fbb, time, frame_number, digitizer_id, measurements_per_frame, num_channels)
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
pub fn create_partly_random_message_with_now(fbb : &mut FlatBufferBuilder<'_>,
    frame_number: RangeInclusive<FrameNumber>,
    digitizer_id : RangeInclusive<DigitizerId>,
    measurements_per_frame : usize,
    num_channels : usize) -> Result<String,Error> {
    let time: GpsTime = Utc::now().into();
    create_partly_random_message(fbb, time, frame_number, digitizer_id, measurements_per_frame, num_channels)
}

/// Loads a FlatBufferBuilder with a new DigitizerAnalogTraceMessage instance with the present timestamp.
/// and all random parameters.
/// #Arguments
/// * `fbb` - A mutable reference to the FlatBufferBuilder to use.
/// * `time` - A `frame_metadata_v1_generated::GpsTime` instance containing the timestamp.
/// * `frame_number` - The upper and lower bounds from which to sample the frame number from.
/// * `digitizer_id` - The upper and lower bounds from which to sample the digitizer id from.
/// * `measurements_per_frame` - The upper and lower bounds from which to sample the number of measurements from.
/// * `num_channels` - The upper and lower bounds from which to sample the number of channels from.
/// #Returns
/// A string result, or an error.
pub fn create_random_message_with_now(fbb : &mut FlatBufferBuilder<'_>,
        frame_number: RangeInclusive<FrameNumber>,
        digitizer_id : RangeInclusive<DigitizerId>,
        measurements_per_frame : RangeInclusive<usize>,
        num_channels : RangeInclusive<usize>) -> Result<String,Error> {
    let time: GpsTime = Utc::now().into();
    create_random_message(fbb, time, frame_number, digitizer_id, measurements_per_frame, num_channels)
}



#[cfg(test)]
mod test {
    use super::*;
    use std::ops::RangeInclusive;
    use flatbuffers::FlatBufferBuilder;
    use streaming_types::{
        dat1_digitizer_analog_trace_v1_generated::root_as_digitizer_analog_trace_message,
        frame_metadata_v1_generated::GpsTime
    };

    #[test]
    fn test_basic() {
        let timestamp = GpsTime::new(22, 205, 10, 55, 30, 0, 1, 5);
        let digitizer_id = 2;
        let frame_number = 495;
        let num_time_points = 20;
        let num_channels = 4;

        let mut fbb = FlatBufferBuilder::new();
        let string = create_message(&mut fbb, timestamp, frame_number, digitizer_id, num_time_points,num_channels).unwrap();
        assert_eq!(string, format!("New message created for digitizer {digitizer_id}, frame number {frame_number}, and has {num_channels} channels, and {num_time_points} measurements."));
        let msg = root_as_digitizer_analog_trace_message(fbb.finished_data()).unwrap();
        
        assert_eq!(msg.digitizer_id(),digitizer_id);

        assert!(msg.metadata().timestamp().is_some());
        /*assert_eq!(msg.metadata().timestamp().timestamp_seconds(), 0);
        assert_eq!(msg.metadata().timestamp().timestamp_millis(), 0);
        assert_eq!(msg.metadata().timestamp().timestamp_micros(), 0);
        assert_eq!(msg.metadata().timestamp().timestamp_nanos(), 0);*/

        assert_eq!(msg.metadata().frame_number(),frame_number);
        
        assert!(msg.channels().is_some());
        assert_eq!(msg.channels().unwrap().len(),num_channels);
        let channels = msg.channels().unwrap();
        for (i,c) in channels.iter().enumerate() {
            assert_eq!(c.channel(),i as Channel);
            assert!(c.voltage().is_some());
            assert_eq!(c.voltage().unwrap().len(),num_time_points);
        }
    }

    #[test]
    fn test_basic_with_now() {
        let digitizer_id = 2;
        let frame_number = 495;
        let num_time_points = 20;
        let num_channels = 4;

        let mut fbb = FlatBufferBuilder::new();
        let string = create_message_with_now(&mut fbb, frame_number, digitizer_id, num_time_points,num_channels).unwrap();
        assert_eq!(string, format!("New message created for digitizer {digitizer_id}, frame number {frame_number}, and has {num_channels} channels, and {num_time_points} measurements."));
        let msg = root_as_digitizer_analog_trace_message(fbb.finished_data()).unwrap();
        
        assert_eq!(msg.digitizer_id(),digitizer_id);

        assert!(msg.metadata().timestamp().is_some());

        assert_eq!(msg.metadata().frame_number(),frame_number);
        
        assert!(msg.channels().is_some());
        assert_eq!(msg.channels().unwrap().len(),num_channels);
        let channels = msg.channels().unwrap();
        for (i,c) in channels.iter().enumerate() {
            assert_eq!(c.channel(),i as Channel);
            assert!(c.voltage().is_some());
            assert_eq!(c.voltage().unwrap().len(),num_time_points);
        }
    }
    
    #[test]
    fn test_random() {
        let timestamp = GpsTime::new(22, 205, 10, 55, 30, 0, 1, 5);
        let digitizer_id : RangeInclusive<DigitizerId> = 0..=24;
        let frame_number : RangeInclusive<FrameNumber> = 0..=495;
        let num_time_points : RangeInclusive<usize> = 10..=30;
        let num_channels : RangeInclusive<usize> = 4..=8;

        let mut fbb = FlatBufferBuilder::new();
        let _string = create_random_message(&mut fbb, timestamp, frame_number.clone(), digitizer_id.clone(), num_time_points.clone(), num_channels.clone()).unwrap();
        //assert_eq!(string, format!("New message created for digitizer {digitizer_id}, frame number {frame_number}, and has {num_channels} channels, and {num_time_points} measurements."));
        let msg = root_as_digitizer_analog_trace_message(fbb.finished_data()).unwrap();
        
        assert!(digitizer_id.contains(&msg.digitizer_id()));

        assert!(msg.metadata().timestamp().is_some());
        /*assert_eq!(msg.metadata().timestamp().timestamp_seconds(), 0);
        assert_eq!(msg.metadata().timestamp().timestamp_millis(), 0);
        assert_eq!(msg.metadata().timestamp().timestamp_micros(), 0);
        assert_eq!(msg.metadata().timestamp().timestamp_nanos(), 0);*/

        assert!(frame_number.contains(&msg.metadata().frame_number()));
        
        assert!(msg.channels().is_some());
        let channels = msg.channels().unwrap();
        assert!(num_channels.contains(&channels.len()));
        
        for (i,c) in channels.iter().enumerate() {
            assert_eq!(c.channel(),i as Channel);
            assert!(c.voltage().is_some());
            assert!(num_time_points.contains(&c.voltage().unwrap().len()));
        }
    }
}
 
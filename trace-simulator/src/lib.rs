//! This module allows one to simulate instances of DigitizerAnalogTraceMessage
//! using the FlatBufferBuilder.
//!

use anyhow::Error;
use itertools::Itertools;
//use std::ops::Range;
use chrono::Utc;
use core::ops::Range;
use flatbuffers::{FlatBufferBuilder, WIPOffset};
use rand::{random, rngs::ThreadRng, thread_rng, Rng};
use std::ops::RangeInclusive;

pub mod generator;
pub use generator::{create_pulses, create_trace, Pulse, PulseDistribution, RandomInterval};

use common::{Channel, DigitizerId, FrameNumber, Intensity};
use streaming_types::{
    dat1_digitizer_analog_trace_v1_generated::{
        finish_digitizer_analog_trace_message_buffer, ChannelTrace, ChannelTraceArgs,
        DigitizerAnalogTraceMessage, DigitizerAnalogTraceMessageArgs,
    },
    frame_metadata_v1_generated::{FrameMetadataV1, FrameMetadataV1Args, GpsTime},
};

pub type Malform = Vec<MalformType>;

#[derive(PartialEq)]
pub enum MalformType {
    DeleteTimestamp,
    DeleteMetadata,
    DeleteChannels,
    DeleteVoltagesOfChannel(Channel),
    TruncateVoltagesOfChannelByHalf(Channel),
    SetChannelId(Channel, Channel),
}

fn none_if_malform_contains_or<T>(malform: &Malform, mt: MalformType, output: T) -> Option<T> {
    match malform.contains(&mt) {
        true => None,
        false => Some(output),
    }
}

pub fn create_channel<'a>(
    fbb: &mut FlatBufferBuilder<'a>,
    channel: Channel,
    measurements_per_frame: usize,
    malform: &Malform,
) -> WIPOffset<ChannelTrace<'a>> {
    let measurements_per_frame =
        match malform.contains(&MalformType::TruncateVoltagesOfChannelByHalf(channel)) {
            true => measurements_per_frame / 2,
            false => measurements_per_frame,
        };
    let distrbution = PulseDistribution {
        std_dev: RandomInterval(
            0.01 * measurements_per_frame as f64,
            0.1 * measurements_per_frame as f64,
        ),
        decay_factor: RandomInterval(0., 1.),
        time_wobble: RandomInterval(0., 2.),
        value_wobble: RandomInterval(0., 0.01),
        peak: RandomInterval(220., 880.),
    };
    let pulses: Vec<Pulse> = create_pulses(measurements_per_frame, 0, 25, &distrbution);
    let items: Vec<Intensity> = create_trace(measurements_per_frame, pulses, 0, 50, 30000, 10);
    let voltage = none_if_malform_contains_or(
        malform,
        MalformType::DeleteVoltagesOfChannel(channel),
        fbb.create_vector::<Intensity>(&items),
    );
    let channel = malform.iter().fold(
        channel,
        |current_channel_id, malform_type| match malform_type {
            MalformType::SetChannelId(channel_index, new_channel_id) => {
                if *channel_index == channel {
                    *new_channel_id
                } else {
                    current_channel_id
                }
            }
            _ => current_channel_id,
        },
    );
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
    measurements_per_frame: usize,
    num_channels: usize,
    malform: &Malform,
) -> Result<String, Error> {
    let time: GpsTime = Utc::now().into();
    create_message(
        fbb,
        time,
        frame_number,
        digitizer_id,
        measurements_per_frame,
        num_channels,
        malform,
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
    measurements_per_frame: usize,
    num_channels: usize,
    malform: &Malform,
) -> Result<String, Error> {
    fbb.reset();

    let metadata: FrameMetadataV1Args = FrameMetadataV1Args {
        frame_number,
        period_number: 0,
        protons_per_pulse: 0,
        running: true,
        timestamp: none_if_malform_contains_or(malform, MalformType::DeleteTimestamp, &time),
        veto_flags: 0,
    };
    let metadata: WIPOffset<FrameMetadataV1> = FrameMetadataV1::create(fbb, &metadata);

    let channels: Vec<_> = (0..num_channels)
        .into_iter()
        .map(|c| create_channel(fbb, c as u32, measurements_per_frame, malform))
        .collect();

    let message = DigitizerAnalogTraceMessageArgs {
        digitizer_id: digitizer_id,
        metadata: none_if_malform_contains_or(malform, MalformType::DeleteMetadata, metadata),
        sample_rate: 1_000_000_000,
        channels: none_if_malform_contains_or(
            malform,
            MalformType::DeleteChannels,
            fbb.create_vector_from_iter(channels.iter()),
        ),
    };
    let message = DigitizerAnalogTraceMessage::create(fbb, &message);
    finish_digitizer_analog_trace_message_buffer(fbb, message);

    Ok(format!("New message created for digitizer {digitizer_id}, frame number {frame_number}, and has {num_channels} channels, and {measurements_per_frame} measurements."))
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
    measurements_per_frame: usize,
    num_channels: usize,
    malform: &Malform,
) -> Result<String, Error> {
    let mut rng = rand::thread_rng();
    let frame_number = rng.gen_range(frame_number);
    let digitizer_id = rng.gen_range(digitizer_id);
    create_message(
        fbb,
        time,
        frame_number,
        digitizer_id,
        measurements_per_frame,
        num_channels,
        malform,
    )
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
pub fn create_random_message(
    fbb: &mut FlatBufferBuilder<'_>,
    time: GpsTime,
    frame_number: RangeInclusive<FrameNumber>,
    digitizer_id: RangeInclusive<DigitizerId>,
    measurements_per_frame: RangeInclusive<usize>,
    num_channels: RangeInclusive<usize>,
    malform: &Malform,
) -> Result<String, Error> {
    let mut rng = rand::thread_rng();
    let measurements_per_frame = rng.gen_range(measurements_per_frame);
    let num_channels = rng.gen_range(num_channels);
    create_partly_random_message(
        fbb,
        time,
        frame_number,
        digitizer_id,
        measurements_per_frame,
        num_channels,
        malform,
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
    measurements_per_frame: usize,
    num_channels: usize,
    malform: &Malform,
) -> Result<String, Error> {
    let time: GpsTime = Utc::now().into();
    create_partly_random_message(
        fbb,
        time,
        frame_number,
        digitizer_id,
        measurements_per_frame,
        num_channels,
        malform,
    )
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
pub fn create_random_message_with_now(
    fbb: &mut FlatBufferBuilder<'_>,
    frame_number: RangeInclusive<FrameNumber>,
    digitizer_id: RangeInclusive<DigitizerId>,
    measurements_per_frame: RangeInclusive<usize>,
    num_channels: RangeInclusive<usize>,
    malform: &Malform,
) -> Result<String, Error> {
    let time: GpsTime = Utc::now().into();
    create_random_message(
        fbb,
        time,
        frame_number,
        digitizer_id,
        measurements_per_frame,
        num_channels,
        malform,
    )
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
pub fn remove_message_timestamp(
    fbb: &mut FlatBufferBuilder<'_>,
    time: GpsTime,
    frame_number: RangeInclusive<FrameNumber>,
    digitizer_id: RangeInclusive<DigitizerId>,
    measurements_per_frame: RangeInclusive<usize>,
    num_channels: RangeInclusive<usize>,
    malform: &Malform,
) -> Result<String, Error> {
    let mut rng = rand::thread_rng();
    let measurements_per_frame = rng.gen_range(measurements_per_frame);
    let num_channels = rng.gen_range(num_channels);
    create_partly_random_message(
        fbb,
        time,
        frame_number,
        digitizer_id,
        measurements_per_frame,
        num_channels,
        malform,
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

        let mut fbb = FlatBufferBuilder::new();
        let string = create_message(
            &mut fbb,
            timestamp,
            frame_number,
            digitizer_id,
            num_time_points,
            num_channels,
            &Malform::default(),
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
        }
    }

    #[test]
    fn test_basic_with_now() {
        let digitizer_id = 2;
        let frame_number = 495;
        let num_time_points = 20;
        let num_channels = 4;

        let mut fbb = FlatBufferBuilder::new();
        let string = create_message_with_now(
            &mut fbb,
            frame_number,
            digitizer_id,
            num_time_points,
            num_channels,
            &Malform::default(),
        )
        .unwrap();
        assert_eq!(string, format!("New message created for digitizer {digitizer_id}, frame number {frame_number}, and has {num_channels} channels, and {num_time_points} measurements."));
        let msg = root_as_digitizer_analog_trace_message(fbb.finished_data()).unwrap();

        assert_eq!(msg.digitizer_id(), digitizer_id);

        assert!(msg.metadata().timestamp().is_some());

        assert_eq!(msg.metadata().frame_number(), frame_number);

        assert!(msg.channels().is_some());
        assert_eq!(msg.channels().unwrap().len(), num_channels);
        let channels = msg.channels().unwrap();
        for (i, c) in channels.iter().enumerate() {
            assert_eq!(c.channel(), i as Channel);
            assert!(c.voltage().is_some());
            assert_eq!(c.voltage().unwrap().len(), num_time_points);
        }
    }

    #[test]
    fn test_random() {
        let timestamp = GpsTime::new(22, 205, 10, 55, 30, 0, 1, 5);
        let digitizer_id: RangeInclusive<DigitizerId> = 0..=24;
        let frame_number: RangeInclusive<FrameNumber> = 0..=495;
        let num_time_points: RangeInclusive<usize> = 10..=30;
        let num_channels: RangeInclusive<usize> = 4..=8;

        let mut fbb = FlatBufferBuilder::new();
        let _string = create_random_message(
            &mut fbb,
            timestamp,
            frame_number.clone(),
            digitizer_id.clone(),
            num_time_points.clone(),
            num_channels.clone(),
            &Malform::default(),
        )
        .unwrap();
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

        for (i, c) in channels.iter().enumerate() {
            assert_eq!(c.channel(), i as Channel);
            assert!(c.voltage().is_some());
            assert!(num_time_points.contains(&c.voltage().unwrap().len()));
        }
    }
    fn test_malformed_generate_message<'a>(
        fbb: &'a mut FlatBufferBuilder,
        malform: Malform,
    ) -> DigitizerAnalogTraceMessage<'a> {
        let timestamp = GpsTime::new(22, 205, 10, 55, 30, 0, 1, 5);
        let digitizer_id: RangeInclusive<DigitizerId> = 0..=24;
        let frame_number: RangeInclusive<FrameNumber> = 0..=495;
        let num_time_points: RangeInclusive<usize> = 10..=30;
        let num_channels: RangeInclusive<usize> = 4..=8;

        let _string = create_random_message(
            fbb,
            timestamp,
            frame_number.clone(),
            digitizer_id.clone(),
            num_time_points.clone(),
            num_channels.clone(),
            &malform,
        )
        .unwrap();
        //assert_eq!(string, format!("New message created for digitizer {digitizer_id}, frame number {frame_number}, and has {num_channels} channels, and {num_time_points} measurements."));
        root_as_digitizer_analog_trace_message(fbb.finished_data()).unwrap()
    }

    #[test]
    fn test_malformed() {
        let mut fbb = FlatBufferBuilder::new();
        let message = test_malformed_generate_message(&mut fbb, vec![MalformType::DeleteTimestamp]);
        assert!(message.metadata().timestamp().is_none());
        assert!(message.channels().is_some());
        assert!(message.channels().unwrap().get(0).voltage().is_some());
        assert!(message.channels().unwrap().get(1).voltage().is_some());

        fbb.reset();
        let message = test_malformed_generate_message(&mut fbb, vec![MalformType::DeleteChannels]);
        assert!(message.metadata().timestamp().is_some());
        assert!(message.channels().is_none());

        fbb.reset();
        let message = test_malformed_generate_message(
            &mut fbb,
            vec![MalformType::DeleteTimestamp, MalformType::DeleteChannels],
        );
        assert!(message.metadata().timestamp().is_none());
        assert!(message.channels().is_none());

        fbb.reset();
        let message = test_malformed_generate_message(
            &mut fbb,
            vec![MalformType::DeleteVoltagesOfChannel(0)],
        );
        assert!(message.metadata().timestamp().is_some());
        assert!(message.channels().is_some());
        assert!(message.channels().unwrap().get(0).voltage().is_none());
        assert!(message.channels().unwrap().get(1).voltage().is_some());

        fbb.reset();
        let message = test_malformed_generate_message(
            &mut fbb,
            vec![MalformType::TruncateVoltagesOfChannelByHalf(0)],
        );
        assert!(message.metadata().timestamp().is_some());
        assert!(message.channels().is_some());
        let channels = message.channels().unwrap();
        assert!(channels.get(0).voltage().is_some());
        assert!(channels.get(1).voltage().is_some());
        assert!(channels.get(2).voltage().is_some());
        assert_eq!(
            channels.get(0).voltage().unwrap().len(),
            channels.get(1).voltage().unwrap().len() / 2
        );
        assert_eq!(
            channels.get(1).voltage().unwrap().len(),
            channels.get(2).voltage().unwrap().len()
        );
    }

    #[test]
    fn test_malformed_duplicate_channels() {
        let mut fbb = FlatBufferBuilder::new();
        let message = test_malformed_generate_message(
            &mut fbb,
            vec![
                MalformType::SetChannelId(1, 23),
                MalformType::SetChannelId(2, 23),
            ],
        );
        assert!(message.metadata().timestamp().is_some());
        assert!(message.channels().is_some());
        let channels = message.channels().unwrap();
        assert_eq!(channels.get(0).channel(), 0);
        assert_eq!(channels.get(1).channel(), 23);
        assert_eq!(channels.get(2).channel(), 23);
        assert_eq!(channels.get(3).channel(), 3);
    }
}

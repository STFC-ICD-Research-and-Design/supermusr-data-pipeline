use super::*;
use std::{env, path::PathBuf};
use supermusr_streaming_types::{
    dat2_digitizer_analog_trace_v2_generated::{
        finish_digitizer_analog_trace_message_buffer, root_as_digitizer_analog_trace_message,
        ChannelTrace, ChannelTraceArgs, DigitizerAnalogTraceMessage,
        DigitizerAnalogTraceMessageArgs,
    },
    flatbuffers::FlatBufferBuilder,
    frame_metadata_v2_generated::{FrameMetadataV2, FrameMetadataV2Args, GpsTime},
};

mod basic;
mod multiple_digitizers;
mod multiple_digitizers_missing_data;

fn create_test_filename(name: &str) -> PathBuf {
    let mut path = env::temp_dir();
    path.push(format!("{name}.h5"));
    path
}

fn push_frame(
    file: &mut TraceFile,
    num_time_points: usize,
    frame_number: u32,
    time: GpsTime,
    channel_offset: u32,
    digitizer_id: u8,
) {
    let mut fbb = FlatBufferBuilder::new();

    let metadata = FrameMetadataV2Args {
        frame_number,
        period_number: 0,
        protons_per_pulse: 0,
        running: true,
        timestamp: Some(&time),
        veto_flags: 0,
    };
    let metadata = FrameMetadataV2::create(&mut fbb, &metadata);

    let mut voltage: Vec<Intensity> = vec![10; num_time_points];
    voltage[0] = digitizer_id as Intensity;
    voltage[1] = frame_number as Intensity;
    let voltage = Some(fbb.create_vector::<Intensity>(&voltage));
    let channel0 = ChannelTrace::create(
        &mut fbb,
        &ChannelTraceArgs {
            channel: channel_offset,
            voltage,
        },
    );

    let mut voltage: Vec<Intensity> = vec![11; num_time_points];
    voltage[0] = digitizer_id as Intensity;
    voltage[1] = frame_number as Intensity;
    let voltage = Some(fbb.create_vector::<Intensity>(&voltage));
    let channel1 = ChannelTrace::create(
        &mut fbb,
        &ChannelTraceArgs {
            channel: channel_offset + 1,
            voltage,
        },
    );

    let message = DigitizerAnalogTraceMessageArgs {
        digitizer_id,
        metadata: Some(metadata),
        sample_rate: 1_000_000_000,
        channels: Some(fbb.create_vector(&[channel0, channel1])),
    };
    let message = DigitizerAnalogTraceMessage::create(&mut fbb, &message);
    finish_digitizer_analog_trace_message_buffer(&mut fbb, message);

    let message = root_as_digitizer_analog_trace_message(fbb.finished_data()).unwrap();
    assert!(file.push(&message).is_ok());
}

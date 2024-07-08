use super::{
    engine::SimulationEngineCache,
    schedule::{SelectionModeOptions, SourceOptions},
};
use anyhow::Result;
use supermusr_common::{Channel, DigitizerId, Intensity, Time};
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
    FrameMetadata,
};

fn create_v2_metadata_args<'a>(timestamp : &'a GpsTime, metadata : &FrameMetadata) -> FrameMetadataV2Args<'a> {
    FrameMetadataV2Args {
        frame_number: metadata.frame_number,
        period_number: metadata.period_number,
        protons_per_pulse: metadata.protons_per_pulse,
        running: metadata.running,
        timestamp: Some(timestamp),
        veto_flags: metadata.veto_flags,
    }
}

#[tracing::instrument(skip_all, target = "otel", fields(digitizer_id = digitizer_id))]
pub(crate) fn build_trace_message(
    fbb: &mut FlatBufferBuilder<'_>,
    sample_rate: u64,
    cache: &mut SimulationEngineCache,
    metadata: &FrameMetadata,
    digitizer_id: DigitizerId,
    channels: &[Channel],
    selection_mode: SelectionModeOptions,
) -> Option<()> {
    let channels = channels
        .iter()
        .map(|&channel| {
            let trace = cache.get_trace(selection_mode);
            let voltage = Some(fbb.create_vector::<Intensity>(trace));
            cache.finish_trace(selection_mode);
            ChannelTrace::create(fbb, &ChannelTraceArgs { channel, voltage })
        })
        .collect::<Vec<_>>();
    
    let timestamp = metadata.timestamp.try_into().unwrap();
    let metadata_args = create_v2_metadata_args(&timestamp, metadata);
    
    let message = DigitizerAnalogTraceMessageArgs {
        digitizer_id,
        metadata: Some(FrameMetadataV2::create(fbb, &metadata_args)),
        sample_rate,
        channels: Some(fbb.create_vector(&channels)),
    };
    let message = DigitizerAnalogTraceMessage::create(fbb, &message);
    finish_digitizer_analog_trace_message_buffer(fbb, message);

    Some(())
}

#[tracing::instrument(skip_all, target = "otel", fields(digitizer_id = digitizer_id))]
pub(crate) fn build_digitiser_event_list_message(
    fbb: &mut FlatBufferBuilder<'_>,
    cache: &mut SimulationEngineCache,
    metadata: &FrameMetadata,
    digitizer_id: DigitizerId,
    channels: &[Channel],
    source_options: &SourceOptions,
) -> Result<()> {
    let mut time = Vec::<Time>::new();
    let mut voltage = Vec::<Intensity>::new();
    let mut channel = Vec::<Channel>::new();

    if let SourceOptions::SelectFromCache(selection_mode) = source_options {
        let event_lists = cache.get_event_lists(*selection_mode, channels.len());
        channels
            .iter()
            .zip(event_lists)
            .for_each(|(c, event_list)| {
                event_list.pulses.iter().for_each(|p| {
                    time.push(p.time());
                    voltage.push(p.intensity());
                    channel.push(*c);
                });
            });
        cache.finish_event_lists(*selection_mode, channels.len());
    }

    let timestamp = metadata.timestamp.try_into().unwrap();
    let metadata_args = create_v2_metadata_args(&timestamp, metadata);
    
    let message = DigitizerEventListMessageArgs {
        digitizer_id,
        metadata: Some(FrameMetadataV2::create(fbb, &metadata_args)),
        time: Some(fbb.create_vector(&time)),
        voltage: Some(fbb.create_vector(&voltage)),
        channel: Some(fbb.create_vector(&channels)),
    };
    let message = DigitizerEventListMessage::create(fbb, &message);
    finish_digitizer_event_list_message_buffer(fbb, message);

    Ok(())
}

#[tracing::instrument(skip_all, target = "otel")]
pub(crate) fn build_aggregated_event_list_message(
    fbb: &mut FlatBufferBuilder<'_>,
    cache: &mut SimulationEngineCache,
    metadata: &FrameMetadata,
    channels: &[Channel],
    source_options: &SourceOptions,
) -> Result<()> {
    let mut time = Vec::<Time>::new();
    let mut voltage = Vec::<Intensity>::new();
    let mut channel = Vec::<Channel>::new();

    if let SourceOptions::SelectFromCache(selection_mode) = source_options {
        let event_lists = cache.get_event_lists(*selection_mode, channels.len());
        channels
            .iter()
            .zip(event_lists)
            .for_each(|(c, event_list)| {
                event_list.pulses.iter().for_each(|p| {
                    time.push(p.time());
                    voltage.push(p.intensity());
                    channel.push(*c);
                });
            });
        cache.finish_event_lists(*selection_mode, channels.len());
    }
    
    let timestamp = metadata.timestamp.try_into().unwrap();
    let metadata_args = create_v2_metadata_args(&timestamp, metadata);

    let message = FrameAssembledEventListMessageArgs {
        metadata: Some(FrameMetadataV2::create(fbb, &metadata_args)),
        time: Some(fbb.create_vector(&time)),
        voltage: Some(fbb.create_vector(&voltage)),
        channel: Some(fbb.create_vector(&channels)),
    };
    let message = FrameAssembledEventListMessage::create(fbb, &message);
    finish_frame_assembled_event_list_message_buffer(fbb, message);

    Ok(())
}

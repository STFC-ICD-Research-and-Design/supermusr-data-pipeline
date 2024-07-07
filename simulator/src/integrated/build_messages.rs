use super::{engine::Cache, schedule::Source, simulation::Simulation};
use anyhow::Result;
use supermusr_common::{Channel, DigitizerId, Intensity};
use supermusr_streaming_types::{
    dat2_digitizer_analog_trace_v2_generated::{
        finish_digitizer_analog_trace_message_buffer, ChannelTrace, ChannelTraceArgs,
        DigitizerAnalogTraceMessage, DigitizerAnalogTraceMessageArgs,
    },
    flatbuffers::FlatBufferBuilder,
    frame_metadata_v2_generated::{FrameMetadataV2, FrameMetadataV2Args, GpsTime},
    FrameMetadata,
};

#[tracing::instrument(skip_all, target = "otel", fields(digitizer_id = digitizer_id))]
pub(crate) fn build_trace_message(
    fbb: &mut FlatBufferBuilder<'_>,
    simulation: &Simulation,
    cache: &mut Cache,
    timestamp: GpsTime,
    metadata: &FrameMetadata,
    digitizer_id: DigitizerId,
    channels: &[Channel],
    source: &Source,
) -> Result<()> {
    let channels = channels
        .iter()
        .map(|&channel| {
            let trace = cache.get_trace(source);
            let voltage = Some(fbb.create_vector::<Intensity>(trace));
            cache.finish_trace(source);
            ChannelTrace::create(fbb, &ChannelTraceArgs { channel, voltage })
        })
        .collect::<Vec<_>>();

    let metadata = FrameMetadataV2Args {
        frame_number: metadata.frame_number,
        period_number: metadata.period_number,
        protons_per_pulse: metadata.protons_per_pulse,
        running: metadata.running,
        timestamp: Some(&timestamp),
        veto_flags: metadata.veto_flags,
    };
    let message = DigitizerAnalogTraceMessageArgs {
        digitizer_id,
        metadata: Some(FrameMetadataV2::create(fbb, &metadata)),
        sample_rate: simulation.sample_rate,
        channels: Some(fbb.create_vector(&channels)),
    };
    let message = DigitizerAnalogTraceMessage::create(fbb, &message);
    finish_digitizer_analog_trace_message_buffer(fbb, message);

    Ok(())
}

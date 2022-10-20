use common::{Channel, EventData, Intensity, Time};
use flatbuffers::FlatBufferBuilder;
use itertools::Itertools;
use rayon::prelude::*;
use streaming_types::{
    dat1_digitizer_analog_trace_v1_generated::{ChannelTrace, DigitizerAnalogTraceMessage},
    dev1_digitizer_event_v1_generated::{
        finish_digitizer_event_list_message_buffer, DigitizerEventListMessage,
        DigitizerEventListMessageArgs,
    },
    frame_metadata_v1_generated::{FrameMetadataV1, FrameMetadataV1Args},
};

struct ChannnelEvents {
    channel_number: Channel,

    time: Vec<Time>,
    voltage: Vec<Intensity>,
}

fn find_channel_events(
    trace: &ChannelTrace,
    threshold: Intensity,
    sample_time: Time,
) -> ChannnelEvents {
    let events: Vec<(usize, Intensity)> = trace
        .voltage()
        .unwrap()
        .into_iter()
        .enumerate()
        .tuple_windows()
        .flat_map(|p: ((usize, Intensity), (usize, Intensity))| {
            if p.0 .1 < threshold && p.1 .1 >= threshold {
                Some(p.1)
            } else {
                None
            }
        })
        .collect();

    let mut time = Vec::default();
    let mut voltage = Vec::default();

    for event in events {
        time.push((event.0 as Time) * sample_time);
        voltage.push(event.1);
    }

    ChannnelEvents {
        channel_number: trace.channel(),
        time,
        voltage,
    }
}

pub(crate) fn process(trace: &DigitizerAnalogTraceMessage, threshold: Intensity) -> Vec<u8> {
    log::info!(
        "Dig ID: {}, Metadata: {:?}",
        trace.digitizer_id(),
        trace.metadata()
    );

    let mut fbb = FlatBufferBuilder::new();

    let mut events = EventData::default();

    let sample_time_in_us: Time = (1_000_000 / trace.sample_rate())
        .try_into()
        .expect("Sample time range");

    let channel_events = trace
        .channels()
        .unwrap()
        .iter()
        .collect::<Vec<ChannelTrace>>()
        .par_iter()
        .map(|i| find_channel_events(i, threshold, sample_time_in_us))
        .collect::<Vec<ChannnelEvents>>();

    for mut channel in channel_events {
        events
            .channel
            .append(&mut vec![channel.channel_number; channel.time.len()]);
        events.time.append(&mut channel.time);
        events.voltage.append(&mut channel.voltage);
    }

    let metadata = FrameMetadataV1Args {
        frame_number: trace.metadata().frame_number(),
        period_number: trace.metadata().period_number(),
        running: trace.metadata().running(),
        protons_per_pulse: trace.metadata().protons_per_pulse(),
        timestamp: trace.metadata().timestamp(),
        veto_flags: trace.metadata().veto_flags(),
    };
    let metadata = FrameMetadataV1::create(&mut fbb, &metadata);

    let time = Some(fbb.create_vector(&events.time));
    let voltage = Some(fbb.create_vector(&events.voltage));
    let channel = Some(fbb.create_vector(&events.channel));

    let message = DigitizerEventListMessageArgs {
        digitizer_id: trace.digitizer_id(),
        metadata: Some(metadata),
        time,
        voltage,
        channel,
    };
    let message = DigitizerEventListMessage::create(&mut fbb, &message);
    finish_digitizer_event_list_message_buffer(&mut fbb, message);

    fbb.finished_data().to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use streaming_types::{
        dat1_digitizer_analog_trace_v1_generated::{
            finish_digitizer_analog_trace_message_buffer, root_as_digitizer_analog_trace_message,
            ChannelTraceArgs, DigitizerAnalogTraceMessage, DigitizerAnalogTraceMessageArgs,
        },
        dev1_digitizer_event_v1_generated::{
            digitizer_event_list_message_buffer_has_identifier,
            root_as_digitizer_event_list_message,
        },
        frame_metadata_v1_generated::{FrameMetadataV1, FrameMetadataV1Args, GpsTime},
    };

    #[test]
    fn test_full_message() {
        let mut fbb = FlatBufferBuilder::new();

        let time: GpsTime = Utc::now().into();

        let metadata = FrameMetadataV1Args {
            frame_number: 0,
            period_number: 0,
            protons_per_pulse: 0,
            running: true,
            timestamp: Some(&time),
            veto_flags: 0,
        };
        let metadata = FrameMetadataV1::create(&mut fbb, &metadata);

        let channel0_voltage: Vec<u16> = vec![0, 1, 2, 1, 0, 1, 2, 1, 8, 0, 2, 8, 3, 1, 2];
        let channel0_voltage = Some(fbb.create_vector::<u16>(&channel0_voltage));
        let channel0 = ChannelTrace::create(
            &mut fbb,
            &ChannelTraceArgs {
                channel: 0,
                voltage: channel0_voltage,
            },
        );

        let message = DigitizerAnalogTraceMessageArgs {
            digitizer_id: 0,
            metadata: Some(metadata),
            sample_rate: 1_000_000, // 1 GS/s
            channels: Some(fbb.create_vector(&[channel0])),
        };
        let message = DigitizerAnalogTraceMessage::create(&mut fbb, &message);
        finish_digitizer_analog_trace_message_buffer(&mut fbb, message);

        let message = fbb.finished_data().to_vec();
        let message = root_as_digitizer_analog_trace_message(&message).unwrap();

        let result = process(&message, 2);

        assert!(digitizer_event_list_message_buffer_has_identifier(&result));
        let message = root_as_digitizer_event_list_message(&result).unwrap();

        assert_eq!(
            vec![0, 0, 0, 0, 0],
            message.channel().unwrap().iter().collect::<Vec<_>>()
        );

        assert_eq!(
            vec![2, 6, 8, 10, 14],
            message.time().unwrap().iter().collect::<Vec<_>>()
        );

        assert_eq!(
            vec![2, 2, 8, 2, 2],
            message.voltage().unwrap().iter().collect::<Vec<_>>()
        );
    }
}

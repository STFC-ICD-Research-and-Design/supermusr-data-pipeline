use std::str::FromStr;

use common::{Channel, EventData, Intensity, Time};
use rayon::prelude::*;
use streaming_types::{
    dat1_digitizer_analog_trace_v1_generated::{ChannelTrace, DigitizerAnalogTraceMessage},
    dev1_digitizer_event_v1_generated::{
        finish_digitizer_event_list_message_buffer, DigitizerEventListMessage,
        DigitizerEventListMessageArgs,
    },
    flatbuffers::FlatBufferBuilder,
    frame_metadata_v1_generated::{FrameMetadataV1, FrameMetadataV1Args},
};
use trace_to_pulses::{
    basic_muon_detector::{BasicMuonAssembler, BasicMuonDetector},
    detectors::threshold_detector::{ThresholdAssembler, ThresholdDetector},
    events::{iter::AssembleFilter, SavePulsesToFile},
    trace_iterators::save_to_file::SaveToFile,
    tracedata,
    window::{finite_differences::FiniteDifferences, WindowFilter},
    EventFilter, Real, SmoothingWindow,
};

use crate::parameters::{
    BasicParameters, Mode, SaveOptions, SimpleParameters, ThresholdDurationWrapper,
};

struct ChannnelEvents {
    channel_number: Channel,

    time: Vec<Time>,
    voltage: Vec<Intensity>,
}

fn find_channel_events(
    trace: &ChannelTrace,
    sample_time: Real,
    mode: Option<&Mode>,
    save_options: Option<&SaveOptions>,
) -> ChannnelEvents {
    let events = match &mode {
        Some(Mode::Simple(simple_parameters)) => {
            find_simple_events(trace, simple_parameters, save_options)
        }
        Some(Mode::Basic(basic_parameters)) => {
            find_basic_events(trace, basic_parameters, save_options)
        }
        None => find_simple_events(
            trace,
            &SimpleParameters {
                threshold_trigger: ThresholdDurationWrapper::from_str("-80.0,4").unwrap(),
            },
            save_options,
        ),
    };

    let mut time = Vec::default();
    let mut voltage = Vec::default();

    for event in events {
        time.push(((event.0 as Real) * sample_time) as Time);
        voltage.push(event.1);
    }

    ChannnelEvents {
        channel_number: trace.channel(),
        time,
        voltage,
    }
}

fn find_simple_events(
    trace: &ChannelTrace,
    simple_parameters: &SimpleParameters,
    save_options: Option<&SaveOptions>,
) -> Vec<(usize, Intensity)> {
    let raw = trace
        .voltage()
        .unwrap()
        .into_iter()
        .enumerate()
        .map(|(i, v)| (i as Real, -(v as Real)));

    let pulses = raw
        .clone()
        .events(ThresholdDetector::new(
            &simple_parameters.threshold_trigger.0,
        ))
        .assemble(ThresholdAssembler::default());

    if let Some(save_options) = save_options {
        raw.clone()
            .save_to_file(
                save_options.save_path,
                &(save_options.file_name.to_owned()
                    + &trace.channel().to_string()
                    + "_raw"
                    + ".csv"),
            )
            .unwrap();
        pulses
            .clone()
            .save_to_file(
                save_options.save_path,
                &(save_options.file_name.to_owned()
                    + &trace.channel().to_string()
                    + "_pulses"
                    + ".csv"),
            )
            .unwrap();
    }
    pulses
        .map(|pulse| {
            (
                pulse.start.time.unwrap() as usize,
                pulse.start.value.unwrap_or_default() as Intensity,
            )
        })
        .collect()
}

fn find_basic_events(
    trace: &ChannelTrace,
    basic_parameters: &BasicParameters,
    save_options: Option<&SaveOptions>,
) -> Vec<(usize, Intensity)> {
    let raw = trace
        .voltage()
        .unwrap()
        .into_iter()
        .enumerate()
        .map(|(i, v)| (i as Real, -(v as Real)));

    let smoothed = raw
        .clone()
        .window(SmoothingWindow::new(basic_parameters.smoothing_window_size))
        .map(tracedata::extract::enumerated_mean);

    let pulses = smoothed
        .clone()
        .window(FiniteDifferences::<2>::new())
        .events(BasicMuonDetector::new(
            &basic_parameters.muon_onset.0,
            &basic_parameters.muon_fall.0,
            &basic_parameters.muon_termination.0,
        ))
        .assemble(BasicMuonAssembler::default())
        .filter(|pulse| {
            basic_parameters
                .min_amplitude
                .map(|min_amplitude| {
                    pulse
                        .peak
                        .value
                        .map(|peak_value| peak_value >= min_amplitude)
                        .unwrap_or(true)
                })
                .unwrap_or(true)
        })
        .filter(|pulse| {
            basic_parameters
                .max_amplitude
                .map(|max_amplitude| {
                    pulse
                        .peak
                        .value
                        .map(|peak_value| peak_value <= max_amplitude)
                        .unwrap_or(true)
                })
                .unwrap_or(true)
        });
    if let Some(save_options) = save_options {
        raw.clone()
            .save_to_file(
                save_options.save_path,
                &(save_options.file_name.to_owned()
                    + &trace.channel().to_string()
                    + "_raw"
                    + ".csv"),
            )
            .unwrap();
        smoothed
            .clone()
            .save_to_file(
                save_options.save_path,
                &(save_options.file_name.to_owned()
                    + &trace.channel().to_string()
                    + "_smoothed"
                    + ".csv"),
            )
            .unwrap();
        pulses
            .clone()
            .save_to_file(
                save_options.save_path,
                &(save_options.file_name.to_owned()
                    + &trace.channel().to_string()
                    + "_pulses"
                    + ".csv"),
            )
            .unwrap();
    }
    pulses
        .map(|pulse| {
            (
                pulse.steepest_rise.time.unwrap_or_default() as usize,
                pulse.peak.value.unwrap_or_default() as Intensity,
            )
        })
        .collect()
}

pub(crate) fn process(
    trace: &DigitizerAnalogTraceMessage,
    mode: Option<&Mode>,
    save_options: Option<&SaveOptions>,
) -> Vec<u8> {
    log::info!(
        "Dig ID: {}, Metadata: {:?}",
        trace.digitizer_id(),
        trace.metadata()
    );

    let mut fbb = FlatBufferBuilder::new();

    let mut events = EventData::default();

    let sample_time_in_ns: Real = 1_000_000_000.0 / trace.sample_rate() as Real;

    let channel_events = trace
        .channels()
        .unwrap()
        .iter()
        .collect::<Vec<ChannelTrace>>()
        .par_iter()
        .map(move |channel_trace| {
            find_channel_events(channel_trace, sample_time_in_ns, mode, save_options)
        })
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

        let result = process(&message, None, None);

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

use crate::{
    parameters::{
        AdvancedMuonDetectorParameters, ConstantPhaseDiscriminatorParameters, DetectorSettings,
        Mode, Polarity,
    },
    pulse_detection::{
        advanced_muon_detector::{AdvancedMuonDetector, BasicMuonAssembler},
        threshold_detector::{ThresholdDetector, ThresholdDuration},
        window::{Baseline, FiniteDifferences, SmoothingWindow, WindowFilter},
        AssembleFilter, EventFilter, Real, SaveToFileFilter,
    },
};
use rayon::prelude::*;
use std::path::{Path, PathBuf};
use supermusr_common::{tracer::Spanned, Channel, EventData, FrameNumber, Intensity, Time};
use supermusr_streaming_types::{
    dat1_digitizer_analog_trace_v1_generated::{ChannelTrace, DigitizerAnalogTraceMessage},
    dev1_digitizer_event_v1_generated::{
        finish_digitizer_event_list_message_buffer, DigitizerEventListMessage,
        DigitizerEventListMessageArgs,
    },
    flatbuffers::FlatBufferBuilder,
    frame_metadata_v1_generated::{FrameMetadataV1, FrameMetadataV1Args},
};
use tracing::info;

#[tracing::instrument(skip(trace))]
fn find_channel_events(
    metadata: &FrameMetadataV1,
    trace: &ChannelTrace,
    sample_time: Real,
    detector_settings: &DetectorSettings,
    save_options: Option<&Path>,
) -> (Vec<Time>, Vec<Intensity>) {
    match &detector_settings.mode {
        Mode::ConstantPhaseDiscriminator(parameters) => find_constant_events(
            metadata,
            trace,
            sample_time,
            detector_settings.polarity,
            detector_settings.baseline as Real,
            parameters,
            save_options,
        ),
        Mode::AdvancedMuonDetector(parameters) => find_advanced_events(
            metadata,
            trace,
            sample_time,
            detector_settings.polarity,
            detector_settings.baseline as Real,
            parameters,
            save_options,
        ),
    }
}

#[tracing::instrument(skip(trace))]
fn find_constant_events(
    metadata: &FrameMetadataV1,
    trace: &ChannelTrace,
    sample_time: Real,
    polarity: &Polarity,
    baseline: Real,
    parameters: &ConstantPhaseDiscriminatorParameters,
    save_path: Option<&Path>,
) -> (Vec<Time>, Vec<Intensity>) {
    let sign = match polarity {
        Polarity::Positive => 1.0,
        Polarity::Negative => -1.0,
    };
    let raw = trace
        .voltage()
        .unwrap()
        .into_iter()
        .enumerate()
        .map(|(i, v)| (i as Real * sample_time, sign * (v as Real - baseline)));

    let pulses = raw
        .clone()
        .events(ThresholdDetector::new(&ThresholdDuration {
            threshold: parameters.threshold,
            duration: parameters.duration,
            cool_off: parameters.cool_off,
        }));

    if let Some(save_path) = save_path {
        raw.clone()
            .save_to_file(&get_save_file_name(
                save_path,
                metadata.frame_number(),
                trace.channel(),
                "raw",
            ))
            .unwrap();

        pulses
            .clone()
            .save_to_file(&get_save_file_name(
                save_path,
                metadata.frame_number(),
                trace.channel(),
                "pulses",
            ))
            .unwrap();
    }

    let mut time = Vec::<Time>::new();
    let mut voltage = Vec::<Intensity>::new();
    for pulse in pulses {
        time.push(pulse.0 as Time);
        voltage.push(parameters.threshold as Intensity);
    }
    (time, voltage)
}

#[tracing::instrument(skip(trace), fields(num_pulses))]
fn find_advanced_events(
    metadata: &FrameMetadataV1,
    trace: &ChannelTrace,
    sample_time: Real,
    polarity: &Polarity,
    baseline: Real,
    parameters: &AdvancedMuonDetectorParameters,
    save_path: Option<&Path>,
) -> (Vec<Time>, Vec<Intensity>) {
    let sign = match polarity {
        Polarity::Positive => 1.0,
        Polarity::Negative => -1.0,
    };
    let raw = trace
        .voltage()
        .unwrap()
        .into_iter()
        .enumerate()
        .map(|(i, v)| (i as Real * sample_time, sign * (v as Real - baseline)));

    let smoothed = raw
        .clone()
        .window(Baseline::new(parameters.baseline_length.unwrap_or(0), 0.1))
        .window(SmoothingWindow::new(
            parameters.smoothing_window_size.unwrap_or(1),
        ))
        .map(|(i, stats)| (i, stats.mean));

    let events = smoothed
        .clone()
        .window(FiniteDifferences::<2>::new())
        .events(AdvancedMuonDetector::new(
            parameters.muon_onset,
            parameters.muon_fall,
            parameters.muon_termination,
            parameters.duration,
        ));

    let pulses = events
        .clone()
        .assemble(BasicMuonAssembler::default())
        .filter(|pulse| {
            Option::zip(parameters.min_amplitude, pulse.peak.value)
                .map(|(min, val)| min <= val)
                .unwrap_or(true)
        })
        .filter(|pulse| {
            Option::zip(parameters.max_amplitude, pulse.peak.value)
                .map(|(max, val)| max >= val)
                .unwrap_or(true)
        });

    if let Some(save_path) = save_path {
        raw.clone()
            .save_to_file(&get_save_file_name(
                save_path,
                metadata.frame_number(),
                trace.channel(),
                "raw",
            ))
            .unwrap();

        smoothed
            .clone()
            .save_to_file(&get_save_file_name(
                save_path,
                metadata.frame_number(),
                trace.channel(),
                "smoothed",
            ))
            .unwrap();

        pulses
            .clone()
            .save_to_file(&get_save_file_name(
                save_path,
                metadata.frame_number(),
                trace.channel(),
                "pulses",
            ))
            .unwrap();
    }

    let mut time = Vec::<Time>::new();
    let mut voltage = Vec::<Intensity>::new();
    for pulse in pulses {
        time.push(pulse.steepest_rise.time.unwrap_or_default() as Time);
        voltage.push(pulse.peak.value.unwrap_or_default() as Intensity);
    }
    tracing::Span::current().record("num_pulses", time.len());
    (time, voltage)
}

fn get_save_file_name(
    path: &Path,
    frame_number: FrameNumber,
    channel: Channel,
    subscript: &str,
) -> PathBuf {
    let file_name = format!(
        "{0}f{frame_number}c{channel}_{subscript}",
        path.file_stem()
            .and_then(|os_str| os_str.to_str())
            .expect("file-name should be a valid file name")
    );
    match path.parent() {
        Some(parent) => parent.to_owned().join(file_name).with_extension("csv"),
        None => PathBuf::from(file_name).with_extension("csv"),
    }
}

#[tracing::instrument(skip(trace))]
pub(crate) fn process<'a>(
    fbb: &mut FlatBufferBuilder<'a>,
    trace: &'a DigitizerAnalogTraceMessage,
    detector_settings: &DetectorSettings,
    save_options: Option<&Path>,
) {
    info!(
        "Dig ID: {}, Metadata: {:?}",
        trace.digitizer_id(),
        trace.metadata()
    );

    let sample_time_in_ns: Real = 1_000_000_000.0 / trace.sample_rate() as Real;

    let vec: Vec<(Channel, _)> = trace
        .channels()
        .unwrap()
        .iter()
        .map(Spanned::<ChannelTrace>::new_with_current)
        .collect::<Vec<Spanned<ChannelTrace>>>()
        .par_iter()
        .map(|spanned_channel_trace| {
            spanned_channel_trace.span.in_scope(|| {
                (
                    spanned_channel_trace.value.channel(),
                    find_channel_events(
                        &trace.metadata(),
                        &spanned_channel_trace.value,
                        sample_time_in_ns,
                        detector_settings,
                        save_options,
                    )
                )
            })
        })
        .collect();

    let mut events = EventData::default();
    for (channel, (time, voltage)) in vec {
        events.channel.extend_from_slice(&vec![channel; time.len()]);
        events.time.extend_from_slice(&time);
        events.voltage.extend_from_slice(&voltage);
    }

    let metadata = FrameMetadataV1Args {
        frame_number: trace.metadata().frame_number(),
        period_number: trace.metadata().period_number(),
        running: trace.metadata().running(),
        protons_per_pulse: trace.metadata().protons_per_pulse(),
        timestamp: trace.metadata().timestamp(),
        veto_flags: trace.metadata().veto_flags(),
    };
    let metadata = FrameMetadataV1::create(fbb, &metadata);

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
    let message = DigitizerEventListMessage::create(fbb, &message);
    finish_digitizer_event_list_message_buffer(fbb, message);
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use supermusr_streaming_types::{
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

    fn create_message(
        fbb: &mut FlatBufferBuilder<'_>,
        channel_intensities: &[&[Intensity]],
        time: &GpsTime,
    ) {
        let metadata = FrameMetadataV1Args {
            frame_number: 0,
            period_number: 0,
            protons_per_pulse: 0,
            running: true,
            timestamp: Some(time),
            veto_flags: 0,
        };
        let metadata = FrameMetadataV1::create(fbb, &metadata);

        let channel_vectors: Vec<_> = channel_intensities
            .iter()
            .map(|intensities| Some(fbb.create_vector::<u16>(intensities)))
            .collect();
        let channel_traces: Vec<_> = channel_vectors
            .iter()
            .enumerate()
            .map(|(i, intensities)| {
                ChannelTrace::create(
                    fbb,
                    &ChannelTraceArgs {
                        channel: i as Channel,
                        voltage: *intensities,
                    },
                )
            })
            .collect();

        let message = DigitizerAnalogTraceMessageArgs {
            digitizer_id: 0,
            metadata: Some(metadata),
            sample_rate: 1_000_000_000,
            channels: Some(fbb.create_vector(&channel_traces)),
        };
        let message = DigitizerAnalogTraceMessage::create(fbb, &message);
        finish_digitizer_analog_trace_message_buffer(fbb, message);
    }

    #[test]
    fn const_phase_descr_positive_zero_baseline() {
        let mut fbb = FlatBufferBuilder::new();

        let time: GpsTime = Utc::now().into();
        let channels: Vec<&[Intensity]> =
            vec![[0, 1, 2, 1, 0, 1, 2, 1, 8, 0, 2, 8, 3, 1, 2].as_slice()];
        create_message(&mut fbb, &channels, &time);
        let message = fbb.finished_data().to_vec();
        let message = root_as_digitizer_analog_trace_message(&message).unwrap();

        let test_parameters = ConstantPhaseDiscriminatorParameters {
            threshold: 5.0,
            duration: 1,
            cool_off: 0,
        };
        let mut fbb = FlatBufferBuilder::new();
        process(
            &mut fbb,
            &message,
            &DetectorSettings {
                mode: &Mode::ConstantPhaseDiscriminator(test_parameters),
                polarity: &Polarity::Positive,
                baseline: Intensity::default(),
            },
            None,
        );

        assert!(digitizer_event_list_message_buffer_has_identifier(
            fbb.finished_data()
        ));
        let event_message = root_as_digitizer_event_list_message(fbb.finished_data()).unwrap();

        assert_eq!(
            vec![0, 0],
            event_message.channel().unwrap().iter().collect::<Vec<_>>()
        );

        assert_eq!(
            vec![8, 11],
            event_message.time().unwrap().iter().collect::<Vec<_>>()
        );

        assert_eq!(
            vec![5, 5],
            event_message.voltage().unwrap().iter().collect::<Vec<_>>()
        );
    }

    #[test]
    fn const_phase_descr_positive_zero_baseline_two_channel() {
        let mut fbb = FlatBufferBuilder::new();

        let time: GpsTime = Utc::now().into();
        let channels: Vec<&[Intensity]> = vec![
            [0, 1, 2, 1, 0, 1, 2, 1, 8, 0, 2, 8, 3, 1, 2].as_slice(),
            [0, 1, 2, 1, 0, 1, 2, 1, 8, 0, 2, 8, 3, 1, 2].as_slice(),
        ];
        create_message(&mut fbb, &channels, &time);
        let message = fbb.finished_data().to_vec();
        let message = root_as_digitizer_analog_trace_message(&message).unwrap();

        let test_parameters = ConstantPhaseDiscriminatorParameters {
            threshold: 5.0,
            duration: 1,
            cool_off: 0,
        };
        let mut fbb = FlatBufferBuilder::new();
        process(
            &mut fbb,
            &message,
            &DetectorSettings {
                mode: &Mode::ConstantPhaseDiscriminator(test_parameters),
                polarity: &Polarity::Positive,
                baseline: Intensity::default(),
            },
            None,
        );

        assert!(digitizer_event_list_message_buffer_has_identifier(
            fbb.finished_data()
        ));
        let event_message = root_as_digitizer_event_list_message(fbb.finished_data()).unwrap();

        assert_eq!(
            vec![0, 0, 1, 1],
            event_message.channel().unwrap().iter().collect::<Vec<_>>()
        );

        assert_eq!(
            vec![8, 11, 8, 11],
            event_message.time().unwrap().iter().collect::<Vec<_>>()
        );

        assert_eq!(
            vec![5, 5, 5, 5],
            event_message.voltage().unwrap().iter().collect::<Vec<_>>()
        );
    }

    #[test]
    fn advanced_positive_zero_baseline() {
        let mut fbb = FlatBufferBuilder::new();

        let time: GpsTime = Utc::now().into();
        let channel0: Vec<u16> = vec![0, 1, 2, 1, 0, 1, 2, 1, 8, 0, 2, 8, 3, 1, 2];
        create_message(&mut fbb, &[channel0.as_slice()], &time);
        let message = fbb.finished_data().to_vec();
        let message = root_as_digitizer_analog_trace_message(&message).unwrap();

        let mut fbb = FlatBufferBuilder::new();

        let test_parameters = AdvancedMuonDetectorParameters {
            muon_onset: 0.5,
            muon_fall: -0.01,
            muon_termination: 0.001,
            duration: 0.0,
            smoothing_window_size: Some(2),
            ..Default::default()
        };
        process(
            &mut fbb,
            &message,
            &DetectorSettings {
                mode: &Mode::AdvancedMuonDetector(test_parameters),
                polarity: &Polarity::Positive,
                baseline: Intensity::default(),
            },
            None,
        );

        assert!(digitizer_event_list_message_buffer_has_identifier(
            fbb.finished_data()
        ));
        let event_message = root_as_digitizer_event_list_message(fbb.finished_data()).unwrap();

        assert_eq!(
            vec![0, 0],
            event_message.channel().unwrap().iter().collect::<Vec<_>>()
        );

        assert_eq!(
            vec![1, 7],
            event_message.time().unwrap().iter().collect::<Vec<_>>()
        );

        assert_eq!(
            vec![1, 4],
            event_message.voltage().unwrap().iter().collect::<Vec<_>>()
        );
    }

    #[test]
    fn const_phase_descr_positive_nonzero_baseline() {
        let mut fbb = FlatBufferBuilder::new();

        let time: GpsTime = Utc::now().into();
        let channel0: Vec<u16> = vec![3, 4, 5, 4, 3, 4, 5, 4, 11, 3, 5, 11, 6, 4, 5];
        create_message(&mut fbb, &[channel0.as_slice()], &time);
        let message = fbb.finished_data().to_vec();
        let message = root_as_digitizer_analog_trace_message(&message).unwrap();

        let test_parameters = ConstantPhaseDiscriminatorParameters {
            threshold: 5.0,
            duration: 1,
            cool_off: 0,
        };
        let mut fbb = FlatBufferBuilder::new();
        process(
            &mut fbb,
            &message,
            &DetectorSettings {
                mode: &Mode::ConstantPhaseDiscriminator(test_parameters),
                polarity: &Polarity::Positive,
                baseline: 3,
            },
            None,
        );

        assert!(digitizer_event_list_message_buffer_has_identifier(
            fbb.finished_data()
        ));
        let event_message = root_as_digitizer_event_list_message(fbb.finished_data()).unwrap();

        assert_eq!(
            vec![0, 0],
            event_message.channel().unwrap().iter().collect::<Vec<_>>()
        );

        assert_eq!(
            vec![8, 11],
            event_message.time().unwrap().iter().collect::<Vec<_>>()
        );

        assert_eq!(
            vec![5, 5],
            event_message.voltage().unwrap().iter().collect::<Vec<_>>()
        );
    }

    #[test]
    fn advanced_positive_nonzero_baseline() {
        let mut fbb = FlatBufferBuilder::new();

        let time: GpsTime = Utc::now().into();
        let channel0: Vec<u16> = vec![3, 4, 5, 4, 3, 4, 5, 4, 11, 3, 5, 11, 6, 4, 5];
        create_message(&mut fbb, &[channel0.as_slice()], &time);
        let message = fbb.finished_data().to_vec();
        let message = root_as_digitizer_analog_trace_message(&message).unwrap();

        let mut fbb = FlatBufferBuilder::new();

        let test_parameters = AdvancedMuonDetectorParameters {
            muon_onset: 0.5,
            muon_fall: -0.01,
            muon_termination: 0.001,
            duration: 0.0,
            smoothing_window_size: Some(2),
            ..Default::default()
        };
        process(
            &mut fbb,
            &message,
            &DetectorSettings {
                mode: &Mode::AdvancedMuonDetector(test_parameters),
                polarity: &Polarity::Positive,
                baseline: 3,
            },
            None,
        );

        assert!(digitizer_event_list_message_buffer_has_identifier(
            fbb.finished_data()
        ));
        let event_message = root_as_digitizer_event_list_message(fbb.finished_data()).unwrap();

        assert_eq!(
            vec![0, 0],
            event_message.channel().unwrap().iter().collect::<Vec<_>>()
        );

        assert_eq!(
            vec![1, 7],
            event_message.time().unwrap().iter().collect::<Vec<_>>()
        );

        assert_eq!(
            vec![1, 4],
            event_message.voltage().unwrap().iter().collect::<Vec<_>>()
        );
    }

    #[test]
    fn const_phase_descr_negative_nonzero_baseline() {
        let mut fbb = FlatBufferBuilder::new();

        let time: GpsTime = Utc::now().into();
        let channel0: Vec<u16> = vec![10, 9, 8, 9, 10, 9, 8, 9, 2, 10, 8, 2, 7, 9, 8];
        create_message(&mut fbb, &[channel0.as_slice()], &time);
        let message = fbb.finished_data().to_vec();
        let message = root_as_digitizer_analog_trace_message(&message).unwrap();

        let test_parameters = ConstantPhaseDiscriminatorParameters {
            threshold: 5.0,
            duration: 1,
            cool_off: 0,
        };
        let mut fbb = FlatBufferBuilder::new();
        process(
            &mut fbb,
            &message,
            &DetectorSettings {
                mode: &Mode::ConstantPhaseDiscriminator(test_parameters),
                polarity: &Polarity::Negative,
                baseline: 10,
            },
            None,
        );

        assert!(digitizer_event_list_message_buffer_has_identifier(
            fbb.finished_data()
        ));
        let event_message = root_as_digitizer_event_list_message(fbb.finished_data()).unwrap();

        assert_eq!(
            vec![0, 0],
            event_message.channel().unwrap().iter().collect::<Vec<_>>()
        );

        assert_eq!(
            vec![8, 11],
            event_message.time().unwrap().iter().collect::<Vec<_>>()
        );

        assert_eq!(
            vec![5, 5],
            event_message.voltage().unwrap().iter().collect::<Vec<_>>()
        );
    }

    #[test]
    fn advanced_negative_nonzero_baseline() {
        let mut fbb = FlatBufferBuilder::new();

        let time: GpsTime = Utc::now().into();
        let channel0: Vec<u16> = vec![10, 9, 8, 9, 10, 9, 8, 9, 2, 10, 8, 2, 7, 9, 8];
        create_message(&mut fbb, &[channel0.as_slice()], &time);
        let message = fbb.finished_data().to_vec();
        let message = root_as_digitizer_analog_trace_message(&message).unwrap();

        let mut fbb = FlatBufferBuilder::new();

        let test_parameters = AdvancedMuonDetectorParameters {
            muon_onset: 0.5,
            muon_fall: -0.01,
            muon_termination: 0.001,
            duration: 0.0,
            smoothing_window_size: Some(2),
            ..Default::default()
        };
        process(
            &mut fbb,
            &message,
            &DetectorSettings {
                mode: &Mode::AdvancedMuonDetector(test_parameters),
                polarity: &Polarity::Negative,
                baseline: 10,
            },
            None,
        );

        assert!(digitizer_event_list_message_buffer_has_identifier(
            fbb.finished_data()
        ));
        let event_message = root_as_digitizer_event_list_message(fbb.finished_data()).unwrap();

        assert_eq!(
            vec![0, 0],
            event_message.channel().unwrap().iter().collect::<Vec<_>>()
        );

        assert_eq!(
            vec![1, 7],
            event_message.time().unwrap().iter().collect::<Vec<_>>()
        );

        assert_eq!(
            vec![1, 4],
            event_message.voltage().unwrap().iter().collect::<Vec<_>>()
        );
    }
}

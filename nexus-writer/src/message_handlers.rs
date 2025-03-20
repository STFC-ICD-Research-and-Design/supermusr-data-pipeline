use crate::{nexus::{NexusEngine, Run, SampleEnvironmentLog, SampleEnvironmentLogType}, NexusFile};
use metrics::counter;
use supermusr_common::{
    metrics::{
        failures::{self, FailureKind},
        messages_received::{self, MessageKind},
        metric_names::{FAILURES, MESSAGES_RECEIVED},
    },
    record_metadata_fields_to_span,
    spanned::SpannedAggregator,
};
use supermusr_streaming_types::{
    aev2_frame_assembled_event_v2_generated::{
        frame_assembled_event_list_message_buffer_has_identifier,
        root_as_frame_assembled_event_list_message,
    },
    ecs_6s4t_run_stop_generated::{root_as_run_stop, run_stop_buffer_has_identifier},
    ecs_al00_alarm_generated::{alarm_buffer_has_identifier, root_as_alarm},
    ecs_f144_logdata_generated::{
        f144_LogData, f_144_log_data_buffer_has_identifier, root_as_f_144_log_data,
    },
    ecs_pl72_run_start_generated::{root_as_run_start, run_start_buffer_has_identifier},
    ecs_se00_data_generated::{
        root_as_se_00_sample_environment_data, se00_SampleEnvironmentData,
        se_00_sample_environment_data_buffer_has_identifier,
    },
    flatbuffers::InvalidFlatbuffer,
    FrameMetadata,
};
use tracing::{info_span, instrument, warn, warn_span, Span};

/// Processes the message payload for a message on the frame_event_list topic
pub(crate) fn process_payload_on_frame_event_list_topic(
    nexus_engine: &mut NexusEngine<NexusFile>,
    payload: &[u8],
) {
    if frame_assembled_event_list_message_buffer_has_identifier(payload) {
        process_frame_assembled_event_list_message(nexus_engine, payload);
    } else {
        warn!("Incorrect message identifier on frame event list topic");
    }
}

/// Processes the message payload for a message on the sample_environment topic
pub(crate) fn process_payload_on_sample_env_topic(nexus_engine: &mut NexusEngine<NexusFile>, payload: &[u8]) {
    if f_144_log_data_buffer_has_identifier(payload) {
        process_sample_environment_message(
            nexus_engine,
            SampleEnvironmentLogType::LogData,
            payload,
        );
    } else if se_00_sample_environment_data_buffer_has_identifier(payload) {
        process_sample_environment_message(
            nexus_engine,
            SampleEnvironmentLogType::SampleEnvironmentData,
            payload,
        );
    } else {
        warn!("Incorrect message identifier on sample environment topic");
    }
}

/// Processes the message payload for a message on the run_log topic
pub(crate) fn process_payload_on_runlog_topic(nexus_engine: &mut NexusEngine<NexusFile>, payload: &[u8]) {
    if f_144_log_data_buffer_has_identifier(payload) {
        process_logdata_message(nexus_engine, payload);
    } else {
        warn!("Incorrect message identifier on runlog topic");
    }
}

/// Processes the message payload for a message on the alarm topic
pub(crate) fn process_payload_on_alarm_topic(nexus_engine: &mut NexusEngine<NexusFile>, payload: &[u8]) {
    if alarm_buffer_has_identifier(payload) {
        process_alarm_message(nexus_engine, payload);
    } else {
        warn!("Incorrect message identifier on alarm topic");
    }
}

/// Processes the message payload for a message on the control topic
pub(crate) fn process_payload_on_control_topic(nexus_engine: &mut NexusEngine<NexusFile>, payload: &[u8]) {
    if run_start_buffer_has_identifier(payload) {
        process_run_start_message(nexus_engine, payload);
    } else if run_stop_buffer_has_identifier(payload) {
        process_run_stop_message(nexus_engine, payload);
    } else {
        warn!("Incorrect message identifier on control topic");
    }
}

/// This wrapper function is used to allow tracing of the flatbuffer decoding function.
/// This operation is generally so fast, that I'm not sure this is needed.
/// Alternatively, it may be possible to trace this using functionality in the flatbuffer crate.
#[instrument(skip_all, level = "trace", err(level = "WARN"))]
fn spanned_root_as<'a, R, F>(f: F, payload: &'a [u8]) -> Result<R, InvalidFlatbuffer>
where
    F: Fn(&'a [u8]) -> Result<R, InvalidFlatbuffer>,
{
    f(payload)
}

/// A wrapper function that handles repetative error handing.
fn link_current_span_to_run<F>(run: &Run<NexusFile>, f: F)
where
    F: Fn() -> Span,
{
    if let Err(e) = run.link_current_span(f) {
        warn!("Run span linking failed {e}")
    }
}

/// Emit the warning on an invalid flatbuffer error and increase metric
fn report_parse_message_failure(e: InvalidFlatbuffer) {
    warn!("Failed to parse message: {}", e);
    counter!(
        FAILURES,
        &[failures::get_label(FailureKind::UnableToDecodeMessage)]
    )
    .increment(1);
}

/// Decode, validate and process a flatbuffer RunStart message
#[tracing::instrument(skip_all)]
fn process_run_start_message(nexus_engine: &mut NexusEngine<NexusFile>, payload: &[u8]) {
    counter!(
        MESSAGES_RECEIVED,
        &[messages_received::get_label(MessageKind::RunStart)]
    )
    .increment(1);
    match spanned_root_as(root_as_run_start, payload) {
        Ok(data) => match nexus_engine.start_command(data) {
            Ok(run) => link_current_span_to_run(run, || {
                info_span!(
                    "Run Start Command",
                    "Start" = run.parameters().collect_from.to_string()
                )
            }),
            Err(e) => warn!("Start command ({data:?}) failed {e}"),
        },
        Err(e) => report_parse_message_failure(e),
    }
}

/// Decode, validate and process a flatbuffer RunStop message
#[tracing::instrument(skip_all)]
fn process_run_stop_message(nexus_engine: &mut NexusEngine<NexusFile>, payload: &[u8]) {
    counter!(
        MESSAGES_RECEIVED,
        &[messages_received::get_label(MessageKind::RunStop)]
    )
    .increment(1);
    match spanned_root_as(root_as_run_stop, payload) {
        Ok(data) => match nexus_engine.stop_command(data) {
            Ok(run) => link_current_span_to_run(run, || {
                info_span!(
                    "Run Stop Command",
                    "Stop" = run
                        .parameters()
                        .run_stop_parameters
                        .as_ref()
                        .map(|s| s.collect_until.to_rfc3339())
                        .unwrap_or_default()
                )
            }),
            Err(e) => {
                let _guard = warn_span!(
                    "RunStop Error",
                    run_name = data.run_name(),
                    stop_time = data.stop_time(),
                )
                .entered();
                warn!("{e}");
            }
        },
        Err(e) => report_parse_message_failure(e),
    }
}

/// Decode, validate and process a flatbuffer FrameEventList message
#[tracing::instrument(skip_all,
    fields(
        metadata_timestamp = tracing::field::Empty,
        metadata_frame_number = tracing::field::Empty,
        metadata_period_number = tracing::field::Empty,
        metadata_veto_flags = tracing::field::Empty,
        metadata_protons_per_pulse = tracing::field::Empty,
        metadata_running = tracing::field::Empty,
        frame_is_complete = tracing::field::Empty,
    )
)]
fn process_frame_assembled_event_list_message(nexus_engine: &mut NexusEngine<NexusFile>, payload: &[u8]) {
    counter!(
        MESSAGES_RECEIVED,
        &[messages_received::get_label(MessageKind::Event)]
    )
    .increment(1);
    match spanned_root_as(root_as_frame_assembled_event_list_message, payload) {
        Ok(data) => {
            data.metadata()
                .try_into()
                .map(|metadata: FrameMetadata| {
                    record_metadata_fields_to_span!(metadata, tracing::Span::current());
                    tracing::Span::current().record("frame_is_complete", data.complete());
                })
                .ok();
            match nexus_engine.process_event_list(&data) {
                Ok(Some(run)) => link_current_span_to_run(run, || {
                    let span = info_span!(
                        "Frame Event List",
                        "metadata_timestamp" = tracing::field::Empty,
                        "metadata_frame_number" = tracing::field::Empty,
                        "metadata_period_number" = tracing::field::Empty,
                        "metadata_veto_flags" = tracing::field::Empty,
                        "metadata_protons_per_pulse" = tracing::field::Empty,
                        "metadata_running" = tracing::field::Empty,
                        "frame_is_complete" = data.complete(),
                    );
                    data.metadata()
                        .try_into()
                        .map(|metadata: FrameMetadata| {
                            record_metadata_fields_to_span!(metadata, span);
                        })
                        .ok();
                    span
                }),
                Ok(_) => (),
                Err(e) => warn!("Failed to save frame assembled event list to file: {}", e),
            }
        }
        Err(e) => report_parse_message_failure(e),
    }
}

/// Decode, validate and process flatbuffer SampleEnvironmentLog messages
#[tracing::instrument(skip_all)]
fn process_sample_environment_message(
    nexus_engine: &mut NexusEngine<NexusFile>,
    se_type: SampleEnvironmentLogType,
    payload: &[u8],
) {
    counter!(
        MESSAGES_RECEIVED,
        &[messages_received::get_label(
            MessageKind::SampleEnvironmentData
        )]
    )
    .increment(1);
    let wrapped_result = match se_type {
        SampleEnvironmentLogType::LogData => {
            spanned_root_as(root_as_f_144_log_data, payload).map(SampleEnvironmentLog::LogData)
        }
        SampleEnvironmentLogType::SampleEnvironmentData => {
            spanned_root_as(root_as_se_00_sample_environment_data, payload)
                .map(SampleEnvironmentLog::SampleEnvironmentData)
        }
    };
    match wrapped_result {
        Ok(wrapped_se) => {
            let result = nexus_engine
                .sample_envionment(wrapped_se)
                .inspect_err(|e| warn!("Sample environment error: {e}."));
            match result {
                Ok(Some(run)) => {
                    link_current_span_to_run(run, || info_span!("Sample Environment Log"))
                }
                Ok(_) => (),
                Err(e) => warn!("Sample environment error: {e}"),
            }
        }
        Err(e) => report_parse_message_failure(e),
    }
}

/// Decode, validate and process a flatbuffer Alarm message
#[tracing::instrument(skip_all)]
fn process_alarm_message(nexus_engine: &mut NexusEngine<NexusFile>, payload: &[u8]) {
    counter!(
        MESSAGES_RECEIVED,
        &[messages_received::get_label(MessageKind::Alarm)]
    )
    .increment(1);
    match spanned_root_as(root_as_alarm, payload) {
        Ok(data) => match nexus_engine.alarm(data) {
            Ok(Some(run)) => link_current_span_to_run(run, || info_span!("Alarm")),
            Ok(_) => (),
            Err(e) => warn!("Alarm ({data:?}) failed {e}"),
        },
        Err(e) => report_parse_message_failure(e),
    }
}

/// Decode, validate and process a flatbuffer RunLog message
#[tracing::instrument(skip_all)]
pub(crate) fn process_logdata_message(nexus_engine: &mut NexusEngine<NexusFile>, payload: &[u8]) {
    counter!(
        MESSAGES_RECEIVED,
        &[messages_received::get_label(MessageKind::LogData)]
    )
    .increment(1);
    match spanned_root_as(root_as_f_144_log_data, payload) {
        Ok(data) => match nexus_engine.logdata(&data) {
            Ok(Some(run)) => link_current_span_to_run(run, || info_span!("Run Log Data")),
            Ok(_) => (),
            Err(e) => warn!("Run Log Data ({data:?}) failed. Error: {e}"),
        },
        Err(e) => report_parse_message_failure(e),
    }
}

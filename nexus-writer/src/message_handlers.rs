use metrics::counter;
use crate::nexus::NexusEngine;
use supermusr_common::{
    metrics::{
        failures::{self, FailureKind},
        messages_received::{self, MessageKind},
        metric_names::{FAILURES, MESSAGES_RECEIVED},
    },
    record_metadata_fields_to_span,
    spanned::SpannedAggregator
};
use supermusr_streaming_types::{
    aev2_frame_assembled_event_v2_generated::{frame_assembled_event_list_message_buffer_has_identifier, root_as_frame_assembled_event_list_message},
    ecs_6s4t_run_stop_generated::{root_as_run_stop, run_stop_buffer_has_identifier},
    ecs_al00_alarm_generated::{alarm_buffer_has_identifier, root_as_alarm},
    ecs_f144_logdata_generated::{
        f144_LogData, f_144_log_data_buffer_has_identifier, root_as_f_144_log_data
    },
    ecs_pl72_run_start_generated::{root_as_run_start, run_start_buffer_has_identifier},
    ecs_se00_data_generated::{
        root_as_se_00_sample_environment_data, se00_SampleEnvironmentData, se_00_sample_environment_data_buffer_has_identifier
    },
    flatbuffers::InvalidFlatbuffer,
    FrameMetadata,
};
use tracing::{info_span, instrument, warn, warn_span};

/// Processes the message payload for a message on the frame_event_list topic
pub(crate) fn process_payload_on_frame_event_list_topic(nexus_engine: &mut NexusEngine, payload: &[u8]) {
    if frame_assembled_event_list_message_buffer_has_identifier(payload) {
        process_frame_assembled_event_list_message(nexus_engine, payload);
    } else {
        warn!("Incorrect message identifier on frame event list topic");
    }
}

/// Processes the message payload for a message on the sample_environment topic
pub(crate) fn process_payload_on_sample_env_topic(nexus_engine: &mut NexusEngine, payload: &[u8]) {
    if f_144_log_data_buffer_has_identifier(payload) {
        process_sample_environment_message(nexus_engine, SampleEnvironmentLogType::LogData, payload);
    } else if se_00_sample_environment_data_buffer_has_identifier(payload) {
        process_sample_environment_message(nexus_engine, SampleEnvironmentLogType::SampleEnvironmentData, payload);
    } else {
        warn!("Incorrect message identifier on sample environment topic");
    }
}

pub(crate) fn process_payload_on_runlog_topic(nexus_engine: &mut NexusEngine, payload: &[u8]) {
    if f_144_log_data_buffer_has_identifier(payload) {
        process_logdata_message(nexus_engine, payload);
    } else {
        warn!("Incorrect message identifier on runlog topic");
    }
}

pub(crate) fn process_payload_on_alarm_topic(nexus_engine: &mut NexusEngine, payload: &[u8]) {
    if alarm_buffer_has_identifier(payload) {
        process_alarm_message(nexus_engine, payload);
    } else {
        warn!("Incorrect message identifier on alarm topic");
    }
}

pub(crate) fn process_payload_on_control_topic(nexus_engine: &mut NexusEngine, payload: &[u8]) {
    if run_start_buffer_has_identifier(payload) {
        process_run_start_message(nexus_engine, payload);
    } else if run_stop_buffer_has_identifier(payload) {
        process_run_stop_message(nexus_engine, payload);
    } else {
        warn!("Incorrect message identifier on control topic");
    }
}

#[instrument(skip_all, level = "trace", err(level = "WARN"))]
fn spanned_root_as<'a, R, F>(f: F, payload: &'a [u8]) -> Result<R, InvalidFlatbuffer>
where
    F: Fn(&'a [u8]) -> Result<R, InvalidFlatbuffer>,
{
    f(payload)
}

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
fn process_frame_assembled_event_list_message(nexus_engine: &mut NexusEngine, payload: &[u8]) {
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
                Ok(run) => {
                    if let Some(run) = run {
                        if let Err(e) = run.link_current_span(|| {
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
                        }) {
                            warn!("Run span linking failed {e}")
                        }
                    }
                }
                Err(e) => warn!("Failed to save frame assembled event list to file: {}", e),
            }
        }
        Err(e) => {
            warn!("Failed to parse message: {}", e);
            counter!(
                FAILURES,
                &[failures::get_label(FailureKind::UnableToDecodeMessage)]
            )
            .increment(1);
        }
    }
}

#[tracing::instrument(skip_all)]
fn process_run_start_message(nexus_engine: &mut NexusEngine, payload: &[u8]) {
    counter!(
        MESSAGES_RECEIVED,
        &[messages_received::get_label(MessageKind::RunStart)]
    )
    .increment(1);
    match spanned_root_as(root_as_run_start, payload) {
        Ok(data) => match nexus_engine.start_command(data) {
            Ok(run) => {
                if let Err(e) = run.link_current_span(|| {
                    info_span!(
                        "Run Start Command",
                        "Start" = run.parameters().collect_from.to_string()
                    )
                }) {
                    warn!("Run span linking failed {e}")
                }
            }
            Err(e) => warn!("Start command ({data:?}) failed {e}"),
        },
        Err(e) => {
            warn!("Failed to parse message: {}", e);
            counter!(
                FAILURES,
                &[failures::get_label(FailureKind::UnableToDecodeMessage)]
            )
            .increment(1);
        }
    }
}

#[tracing::instrument(skip_all)]
fn process_run_stop_message(nexus_engine: &mut NexusEngine, payload: &[u8]) {
    counter!(
        MESSAGES_RECEIVED,
        &[messages_received::get_label(MessageKind::RunStop)]
    )
    .increment(1);
    match spanned_root_as(root_as_run_stop, payload) {
        Ok(data) => match nexus_engine.stop_command(data) {
            Ok(run) => {
                if let Err(e) = run.link_current_span(|| {
                    info_span!(
                        "Run Stop Command",
                        "Stop" = run
                            .parameters()
                            .run_stop_parameters
                            .as_ref()
                            .map(|s| s.collect_until.to_rfc3339())
                            .unwrap_or_default()
                    )
                }) {
                    warn!("Run span linking failed {e}")
                }
            }
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
        Err(e) => {
            warn!("Failed to parse message: {}", e);
            counter!(
                FAILURES,
                &[failures::get_label(FailureKind::UnableToDecodeMessage)]
            )
            .increment(1);
        }
    }
}

enum SampleEnvironmentLogType {
    LogData,
    SampleEnvironmentData,
}

pub(crate) enum SampleEnvironmentLog<'a> {
    LogData(f144_LogData<'a>),
    SampleEnvironmentData(se00_SampleEnvironmentData<'a>),
}

#[tracing::instrument(skip_all)]
fn process_sample_environment_message(
    nexus_engine: &mut NexusEngine,
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
    let result = match se_type {
        SampleEnvironmentLogType::LogData => {
            spanned_root_as(root_as_f_144_log_data, payload)
                .map(|data|
                    nexus_engine.sample_envionment(SampleEnvironmentLog::LogData(data))
                        .inspect_err(|_|{
                            warn!("Sample environment ({data:?}) failed.")
                        })
            )
        }
        SampleEnvironmentLogType::SampleEnvironmentData => {
            spanned_root_as(root_as_se_00_sample_environment_data, payload)
                .map(|data|
                    nexus_engine.sample_envionment(SampleEnvironmentLog::SampleEnvironmentData(data))
                        .map_err(|e|{
                            warn!("Sample environment ({data:?}) failed."); e
                        })
                )
        }
    };
    match result {
        Ok(result) => match result {
            Ok(run) => {
                if let Some(run) = run {
                    if let Err(e) = run.link_current_span(|| info_span!("Sample Environment Log")) {
                        warn!("Run span linking failed {e}")
                    }
                }
            }
            Err(e) => warn!("Sample environment error: {e}"),
        },
        Err(e) => {
            warn!("Failed to parse message: {}", e);
            counter!(
                FAILURES,
                &[failures::get_label(FailureKind::UnableToDecodeMessage)]
            )
            .increment(1);
        }
    }
}

#[tracing::instrument(skip_all)]
fn process_alarm_message(nexus_engine: &mut NexusEngine, payload: &[u8]) {
    counter!(
        MESSAGES_RECEIVED,
        &[messages_received::get_label(MessageKind::Alarm)]
    )
    .increment(1);
    match spanned_root_as(root_as_alarm, payload) {
        Ok(data) => match nexus_engine.alarm(data) {
            Ok(run) => {
                if let Some(run) = run {
                    if let Err(e) = run.link_current_span(|| info_span!("Alarm")) {
                        warn!("Run span linking failed {e}")
                    }
                }
            }
            Err(e) => warn!("Alarm ({data:?}) failed {e}"),
        },
        Err(e) => {
            warn!("Failed to parse message: {}", e);
            counter!(
                FAILURES,
                &[failures::get_label(FailureKind::UnableToDecodeMessage)]
            )
            .increment(1);
        }
    }
}

#[tracing::instrument(skip_all)]
pub(crate) fn process_logdata_message(nexus_engine: &mut NexusEngine, payload: &[u8]) {
    counter!(
        MESSAGES_RECEIVED,
        &[messages_received::get_label(MessageKind::LogData)]
    )
    .increment(1);
    match spanned_root_as(root_as_f_144_log_data, payload) {
        Ok(data) => match nexus_engine.logdata(&data) {
            Ok(run) => {
                if let Some(run) = run {
                    if let Err(e) = run.link_current_span(|| info_span!("Run Log Data")) {
                        warn!("Run span linking failed {e}")
                    }
                }
            }
            Err(e) => warn!("Run Log Data ({data:?}) failed. Error: {e}"),
        },
        Err(e) => {
            warn!("Failed to parse message: {}", e);
            counter!(
                FAILURES,
                &[failures::get_label(FailureKind::UnableToDecodeMessage)]
            )
            .increment(1);
        }
    }
}

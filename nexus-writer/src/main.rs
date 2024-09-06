mod nexus;

use chrono::Duration;
use clap::Parser;
use metrics::counter;
use metrics_exporter_prometheus::PrometheusBuilder;
use nexus::{NexusEngine, NexusSettings};
use rdkafka::{
    consumer::{CommitMode, Consumer},
    message::{BorrowedMessage, Message},
};
use std::{net::SocketAddr, path::PathBuf};
use supermusr_common::{
    init_tracer,
    metrics::{
        failures::{self, FailureKind},
        messages_received::{self, MessageKind},
        metric_names::{FAILURES, MESSAGES_PROCESSED, MESSAGES_RECEIVED},
    },
    record_metadata_fields_to_span,
    spanned::SpannedAggregator,
    tracer::{OptionalHeaderTracerExt, TracerEngine, TracerOptions},
    CommonKafkaOpts,
};
use supermusr_streaming_types::{
    aev2_frame_assembled_event_v2_generated::{
        frame_assembled_event_list_message_buffer_has_identifier,
        root_as_frame_assembled_event_list_message,
    },
    ecs_6s4t_run_stop_generated::{root_as_run_stop, run_stop_buffer_has_identifier},
    ecs_al00_alarm_generated::{alarm_buffer_has_identifier, root_as_alarm},
    ecs_f144_logdata_generated::{f_144_log_data_buffer_has_identifier, root_as_f_144_log_data},
    ecs_pl72_run_start_generated::{root_as_run_start, run_start_buffer_has_identifier},
    ecs_se00_data_generated::{
        root_as_se_00_sample_environment_data, se_00_sample_environment_data_buffer_has_identifier,
    },
    flatbuffers::InvalidFlatbuffer,
    FrameMetadata,
};
use tokio::time;
use tracing::{debug, error, info_span, instrument, level_filters::LevelFilter, warn};

#[derive(Debug, Parser)]
#[clap(author, version, about)]
struct Cli {
    #[clap(flatten)]
    common_kafka_options: CommonKafkaOpts,

    /// Kafka consumer group
    #[clap(long)]
    consumer_group: String,

    /// Kafka control topic
    #[clap(long)]
    control_topic: String,

    /// Kafka topic for sample environment messages
    #[clap(long)]
    sample_env_topic: String,

    /// Kafka topic for log environment messages
    #[clap(long)]
    log_topic: String,

    /// Kafka topic for alarm messages
    #[clap(long)]
    alarm_topic: String,

    /// Topic to publish frame assembled event messages to
    #[clap(long)]
    frame_event_topic: String,

    /// Path to the NeXus file to be read
    #[clap(long)]
    file_name: PathBuf,

    /// How often in milliseconds expired runs are checked for and removed
    #[clap(long, default_value = "200")]
    cache_poll_interval_ms: u64,

    /// The amount of time in milliseconds to wait before clearing the run cache
    #[clap(long, default_value = "2000")]
    cache_run_ttl_ms: i64,

    /// If set, then OpenTelemetry data is sent to the URL specified, otherwise the standard tracing subscriber is used
    #[clap(long)]
    otel_endpoint: Option<String>,

    /// The reporting level to use for OpenTelemetry
    #[clap(long, default_value = "info")]
    otel_level: LevelFilter,

    /// Endpoint on which OpenMetrics flavour metrics are available
    #[clap(long, default_value = "127.0.0.1:9090")]
    observability_address: SocketAddr,

    /// The HDF5 chunk size in bytes used when writing the event list
    #[clap(long, default_value = "1048576")]
    event_list_chunk_size: usize,

    /// The HDF5 chunk size in bytes used when writing the frame list
    #[clap(long, default_value = "1024")]
    frame_list_chunk_size: usize,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Cli::parse();

    let tracer = init_tracer!(TracerOptions::new(
        args.otel_endpoint.as_deref(),
        args.otel_level
    ));

    debug!("{args:?}");

    // Get topics to subscribe to from command line arguments.
    let topics_to_subscribe = {
        let mut topics_to_subscribe = [
            args.control_topic.as_str(),
            args.log_topic.as_str(),
            args.frame_event_topic.as_str(),
            args.sample_env_topic.as_str(),
            args.alarm_topic.as_str(),
        ]
        .into_iter()
        .collect::<Vec<&str>>();
        debug!("{topics_to_subscribe:?}");
        topics_to_subscribe.sort();
        topics_to_subscribe.dedup();
        topics_to_subscribe
    };

    let kafka_opts = args.common_kafka_options;

    let consumer = supermusr_common::create_default_consumer(
        &kafka_opts.broker,
        &kafka_opts.username,
        &kafka_opts.password,
        &args.consumer_group,
        &topics_to_subscribe,
    );

    let nexus_settings = NexusSettings::new(args.frame_list_chunk_size, args.event_list_chunk_size);
    let mut nexus_engine = NexusEngine::new(Some(&args.file_name), nexus_settings);

    let mut nexus_write_interval =
        tokio::time::interval(time::Duration::from_millis(args.cache_poll_interval_ms));

    // Install exporter and register metrics
    let builder = PrometheusBuilder::new();
    builder
        .with_http_listener(args.observability_address)
        .install()
        .expect("Prometheus metrics exporter should be setup");

    metrics::describe_counter!(
        MESSAGES_RECEIVED,
        metrics::Unit::Count,
        "Number of messages received"
    );
    metrics::describe_counter!(
        MESSAGES_PROCESSED,
        metrics::Unit::Count,
        "Number of messages processed"
    );
    metrics::describe_counter!(
        FAILURES,
        metrics::Unit::Count,
        "Number of failures encountered"
    );

    loop {
        tokio::select! {
            _ = nexus_write_interval.tick() => {
                nexus_engine.flush(&Duration::try_milliseconds(args.cache_run_ttl_ms).expect("Conversion is possible"));
            }
            event = consumer.recv() => {
                match event {
                    Err(e) => {
                        warn!("{e}")
                    },
                    Ok(msg) => {
                        process_kafka_message(&mut nexus_engine, tracer.use_otel(), &msg);
                        if let Err(e) = consumer.commit_message(&msg, CommitMode::Async){
                            error!("Failed to commit Kafka message consumption: {e}");
                        }
                    }
                }
            }
        }
    }
}

#[tracing::instrument(skip_all, fields(
    num_cached_runs = nexus_engine.get_num_cached_runs(),
    kafka_message_timestamp_ms = msg.timestamp().to_millis()
))]
fn process_kafka_message(nexus_engine: &mut NexusEngine, use_otel: bool, msg: &BorrowedMessage) {
    msg.headers().conditional_extract_to_current_span(use_otel);

    debug!(
        "key: '{:?}', topic: {}, partition: {}, offset: {}, timestamp: {:?}",
        msg.key(),
        msg.topic(),
        msg.partition(),
        msg.offset(),
        msg.timestamp()
    );

    if let Some(payload) = msg.payload() {
        process_payload(nexus_engine, msg.topic(), payload);
    }
}

fn process_payload(nexus_engine: &mut NexusEngine, message_topic: &str, payload: &[u8]) {
    if frame_assembled_event_list_message_buffer_has_identifier(payload) {
        process_frame_assembled_event_list_message(nexus_engine, payload);
    } else if f_144_log_data_buffer_has_identifier(payload) {
        process_logdata_message(nexus_engine, payload);
    } else if se_00_sample_environment_data_buffer_has_identifier(payload) {
        process_sample_environment_message(nexus_engine, payload);
    } else if alarm_buffer_has_identifier(payload) {
        process_alarm_message(nexus_engine, payload);
    } else if run_start_buffer_has_identifier(payload) {
        process_run_start_message(nexus_engine, payload);
    } else if run_stop_buffer_has_identifier(payload) {
        process_run_stop_message(nexus_engine, payload);
    } else {
        warn!("Incorrect message identifier on topic \"{message_topic}\"");
        debug!("Payload size: {}", payload.len());
        counter!(
            MESSAGES_RECEIVED,
            &[messages_received::get_label(MessageKind::Unexpected)]
        )
        .increment(1);
    }
}

#[instrument(skip_all, target = "otel")]
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
                })
                .ok();
            match nexus_engine.process_event_list(&data) {
                Ok(run) => {
                    if let Some(run) = run {
                        if let Err(e) = run.link_current_span(|| {
                            let span = info_span!(target: "otel",
                                "Frame Event List",
                                "metadata_timestamp" = tracing::field::Empty,
                                "metadata_frame_number" = tracing::field::Empty,
                                "metadata_period_number" = tracing::field::Empty,
                                "metadata_veto_flags" = tracing::field::Empty,
                                "metadata_protons_per_pulse" = tracing::field::Empty,
                                "metadata_running" = tracing::field::Empty,
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
                    info_span!(target: "otel",
                    "Run Start Command",
                    "Start" = run.parameters().collect_from.to_string())
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
                    info_span!(target: "otel",
                        "Run Stop Command",
                        "Stop" = run.parameters()
                            .run_stop_parameters
                            .as_ref()
                            .map(|s|s.collect_until.to_rfc3339())
                            .unwrap_or_default()
                    )
                }) {
                    warn!("Run span linking failed {e}")
                }
            }
            Err(e) => warn!("Stop command ({data:?}) failed {e}"),
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
fn process_sample_environment_message(nexus_engine: &mut NexusEngine, payload: &[u8]) {
    counter!(
        MESSAGES_RECEIVED,
        &[messages_received::get_label(
            MessageKind::SampleEnvironmentData
        )]
    )
    .increment(1);
    match spanned_root_as(root_as_se_00_sample_environment_data, payload) {
        Ok(data) => match nexus_engine.sample_envionment(data) {
            Ok(run) => {
                if let Some(run) = run {
                    if let Err(e) = run
                        .link_current_span(|| info_span!(target: "otel", "Sample Environment Log"))
                    {
                        warn!("Run span linking failed {e}")
                    }
                }
            }
            Err(e) => warn!("Sample environment ({data:?}) failed {e}"),
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
                    if let Err(e) = run.link_current_span(|| info_span!(target: "otel", "Alarm")) {
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
fn process_logdata_message(nexus_engine: &mut NexusEngine, payload: &[u8]) {
    counter!(
        MESSAGES_RECEIVED,
        &[messages_received::get_label(MessageKind::LogData)]
    )
    .increment(1);
    match spanned_root_as(root_as_f_144_log_data, payload) {
        Ok(data) => match nexus_engine.logdata(&data) {
            Ok(run) => {
                if let Some(run) = run {
                    if let Err(e) =
                        run.link_current_span(|| info_span!(target: "otel", "Run Log Data"))
                    {
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

mod error;
mod flush_to_archive;
mod hdf5_handlers;
mod message_handlers;
mod run_engine;
mod nexus_structure;

use chrono::Duration;
use clap::Parser;
use flush_to_archive::create_archive_flush_task;
use message_handlers::{
    process_payload_on_alarm_topic, process_payload_on_control_topic,
    process_payload_on_frame_event_list_topic, process_payload_on_runlog_topic,
    process_payload_on_sample_env_topic,
};
use metrics::counter;
use metrics_exporter_prometheus::PrometheusBuilder;
use run_engine::{NexusConfiguration, NexusEngine, NexusSettings};
use rdkafka::{
    consumer::{CommitMode, Consumer},
    message::{BorrowedMessage, Message},
};
use nexus_structure::NexusFile;
use std::{fs::create_dir_all, net::SocketAddr, path::PathBuf};
use supermusr_common::{
    init_tracer,
    metrics::{
        messages_received::{self, MessageKind},
        metric_names::{FAILURES, MESSAGES_PROCESSED, MESSAGES_RECEIVED},
    },
    tracer::{OptionalHeaderTracerExt, TracerEngine, TracerOptions},
    CommonKafkaOpts,
};
use tokio::{
    signal::unix::{signal, SignalKind},
    time,
};
use tracing::{debug, error, warn};

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

    /// Optional data pipeline configuration options to include in the nexus file. If present written to attribute `/raw_data_1/program_name/configuration`.
    #[clap(long)]
    configuration_options: Option<String>,

    /// Whilst the nexus file is being written, it is stored in "local-path/", and moved to "local-path/completed/" once it is finished. The "local-path/" and "local-path/completed/" folders are created automatically.
    #[clap(long)]
    local_path: PathBuf,

    /// Remote path the NeXus file will eventually be moved to after it is finished. If not set, no move takes place.
    #[clap(long)]
    archive_path: Option<PathBuf>,

    /// How often in seconds completed run files are flushed to the remote archive (this does nothing if "archive_path" is not set)
    #[clap(long, default_value = "60")]
    archive_flush_interval_sec: u64,

    /// How often in milliseconds expired runs are checked for and removed
    #[clap(long, default_value = "200")]
    cache_poll_interval_ms: u64,

    /// The amount of time in milliseconds to wait before clearing the run cache
    #[clap(long, default_value = "2000")]
    cache_run_ttl_ms: i64,

    /// If set, then OpenTelemetry data is sent to the URL specified, otherwise the standard tracing subscriber is used
    #[clap(long)]
    otel_endpoint: Option<String>,

    /// All OpenTelemetry spans are emitted with this as the "service.namespace" property. Can be used to track different instances of the pipeline running in parallel.
    #[clap(long, default_value = "")]
    otel_namespace: String,

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

struct Topics<'a> {
    control: &'a str,
    log: &'a str,
    frame_event: &'a str,
    sample_env: &'a str,
    alarm: &'a str,
}

impl Topics<'_> {
    fn get_nonrepeating_list(&self) -> Vec<&str> {
        let mut topics_to_subscribe = [
            self.control,
            self.log,
            self.frame_event,
            self.sample_env,
            self.alarm,
        ]
        .into_iter()
        .collect::<Vec<&str>>();
        debug!("{topics_to_subscribe:?}");
        topics_to_subscribe.sort();
        topics_to_subscribe.dedup();
        topics_to_subscribe
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Cli::parse();

    debug!("{args:?}");

    let tracer = init_tracer!(TracerOptions::new(
        args.otel_endpoint.as_deref(),
        args.otel_namespace
    ));

    // Get topics to subscribe to from command line arguments.
    let topics = Topics {
        control: args.control_topic.as_str(),
        log: args.log_topic.as_str(),
        frame_event: args.frame_event_topic.as_str(),
        sample_env: args.sample_env_topic.as_str(),
        alarm: args.alarm_topic.as_str(),
    };

    let kafka_opts = args.common_kafka_options;

    let consumer = supermusr_common::create_default_consumer(
        &kafka_opts.broker,
        &kafka_opts.username,
        &kafka_opts.password,
        &args.consumer_group,
        &topics.get_nonrepeating_list(),
    );

    let nexus_settings = NexusSettings::new(
        args.local_path.as_path(),
        args.frame_list_chunk_size,
        args.event_list_chunk_size,
        args.archive_path.as_deref(),
        args.archive_flush_interval_sec,
    );

    let mut cache_poll_interval =
        tokio::time::interval(time::Duration::from_millis(args.cache_poll_interval_ms));

    let archive_flush_task = create_archive_flush_task(&nexus_settings)?;

    //  Setup the directory structure, if it doesn't already exist.
    create_dir_all(nexus_settings.get_local_path())?;
    create_dir_all(nexus_settings.get_local_completed_path())?;
    if let Some(archive_path) = nexus_settings.get_archive_path() {
        create_dir_all(archive_path)?;
    }

    let nexus_configuration = NexusConfiguration::new(args.configuration_options);

    let mut nexus_engine = NexusEngine::<NexusFile>::new(nexus_settings, nexus_configuration);
    nexus_engine.resume_partial_runs()?;

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

    let run_ttl =
        Duration::try_milliseconds(args.cache_run_ttl_ms).expect("Conversion is possible");

    // Is used to await any sigint signals
    let mut sigint = signal(SignalKind::interrupt())?;

    loop {
        tokio::select! {
            _ = cache_poll_interval.tick() => {
                nexus_engine.flush(&run_ttl)?;
            }
            event = consumer.recv() => {
                match event {
                    Err(e) => {
                        warn!("{e}")
                    },
                    Ok(msg) => {
                        process_kafka_message(&topics, &mut nexus_engine, tracer.use_otel(), &msg);
                        if let Err(e) = consumer.commit_message(&msg, CommitMode::Async){
                            error!("Failed to commit Kafka message consumption: {e}");
                        }
                    }
                }
            }
            _ = sigint.recv() => {
                // Await completion of the archive_flush_task (which also receives sigint)
                if let Some(archive_flush_task) = archive_flush_task {
                    let _ = archive_flush_task.await?;
                }
                return Ok(());
            }
        }
    }
}

#[tracing::instrument(skip_all, fields(
    num_cached_runs = nexus_engine.get_num_cached_runs(),
    kafka_message_timestamp_ms = msg.timestamp().to_millis()
))]
fn process_kafka_message(
    topics: &Topics,
    nexus_engine: &mut NexusEngine<NexusFile>,
    use_otel: bool,
    msg: &BorrowedMessage,
) {
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
        process_payload(topics, nexus_engine, msg.topic(), payload);
    }
}

fn process_payload(
    topics: &Topics,
    nexus_engine: &mut NexusEngine<NexusFile>,
    message_topic: &str,
    payload: &[u8],
) {
    if message_topic == topics.frame_event {
        process_payload_on_frame_event_list_topic(nexus_engine, payload);
    } else if message_topic == topics.control {
        process_payload_on_control_topic(nexus_engine, payload);
    } else if message_topic == topics.log {
        process_payload_on_runlog_topic(nexus_engine, payload);
    } else if message_topic == topics.sample_env {
        process_payload_on_sample_env_topic(nexus_engine, payload);
    } else if message_topic == topics.alarm {
        process_payload_on_alarm_topic(nexus_engine, payload);
    } else {
        warn!("Unknown topic: \"{message_topic}\"");
        debug!("Payload size: {}", payload.len());
        counter!(
            MESSAGES_RECEIVED,
            &[messages_received::get_label(MessageKind::Unexpected)]
        )
        .increment(1);
    }
}

mod event_message;
mod nexus;

use anyhow::Result;
use chrono::Duration;
use clap::Parser;
use event_message::GenericEventMessage;
use nexus::NexusEngine;
use rdkafka::{
    consumer::{stream_consumer::StreamConsumer, CommitMode, Consumer},
    message::{BorrowedMessage, Message},
};
use std::{net::SocketAddr, path::PathBuf};
use supermusr_common::{
    init_tracer,
    spanned::{Spanned, SpannedMut},
    tracer::{OptionalHeaderTracerExt, TracerEngine, TracerOptions},
};
use supermusr_streaming_types::{
    aev2_frame_assembled_event_v2_generated::{
        frame_assembled_event_list_message_buffer_has_identifier,
        root_as_frame_assembled_event_list_message,
    },
    dev2_digitizer_event_v2_generated::{
        digitizer_event_list_message_buffer_has_identifier, root_as_digitizer_event_list_message,
    },
    ecs_6s4t_run_stop_generated::{root_as_run_stop, run_stop_buffer_has_identifier},
    ecs_al00_alarm_generated::{alarm_buffer_has_identifier, root_as_alarm},
    ecs_f144_logdata_generated::{f_144_log_data_buffer_has_identifier, root_as_f_144_log_data},
    ecs_pl72_run_start_generated::{root_as_run_start, run_start_buffer_has_identifier},
    ecs_se00_data_generated::{
        root_as_se_00_sample_environment_data, se_00_sample_environment_data_buffer_has_identifier,
    },
};
use tokio::time;
use tracing::{debug, error, info_span, level_filters::LevelFilter, trace_span, warn};

#[derive(Debug, Parser)]
#[clap(author, version, about)]
struct Cli {
    #[clap(long)]
    broker: String,

    #[clap(long)]
    username: Option<String>,

    #[clap(long)]
    password: Option<String>,

    #[clap(long)]
    consumer_group: String,

    #[clap(long)]
    control_topic: String,

    /// Kafka topic for sample environment messages
    #[clap(long)]
    sample_env_topic: Option<String>,

    /// Kafka topic for log environment messages
    #[clap(long)]
    log_topic: String,

    /// Kafka topic for alarm messages
    #[clap(long)]
    alarm_topic: Option<String>,

    #[clap(long)]
    digitiser_event_topic: Option<String>,

    #[clap(long)]
    frame_event_topic: Option<String>,

    #[clap(long)]
    file_name: PathBuf,

    #[clap(long, default_value = "200")]
    cache_poll_interval_ms: u64,

    #[clap(long, default_value = "2000")]
    cache_run_ttl_ms: i64,

    /// If set, then open-telemetry data is sent to the URL specified, otherwise the standard tracing subscriber is used
    #[clap(long)]
    otel_endpoint: Option<String>,

    /// If open-telemetry is used then the following log level is used
    #[clap(long, default_value = "info")]
    otel_level: LevelFilter,

    #[clap(long, default_value = "127.0.0.1:9090")]
    observability_address: SocketAddr,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Cli::parse();

    let tracer = init_tracer!(TracerOptions::new(
        args.otel_endpoint.as_deref(),
        args.otel_level
    ));

    trace_span!("Args:").in_scope(|| debug!("{args:?}"));

    let consumer: StreamConsumer = supermusr_common::generate_kafka_client_config(
        &args.broker,
        &args.username,
        &args.password,
    )
    .set("group.id", &args.consumer_group)
    .set("enable.partition.eof", "false")
    .set("session.timeout.ms", "6000")
    .set("enable.auto.commit", "false")
    .create()?;

    //  This line can be simplified when is it clear which topics we need
    let topics_to_subscribe = {
        let mut topics_to_subscribe = [
            Some(args.control_topic.as_str()),
            Some(args.log_topic.as_str()),
            args.digitiser_event_topic.as_deref(),
            args.frame_event_topic.as_deref(),
            args.sample_env_topic.as_deref(),
            args.alarm_topic.as_deref(),
        ]
        .into_iter()
        .flatten()
        .collect::<Vec<&str>>();
        trace_span!("Topics in: ").in_scope(|| debug!("{topics_to_subscribe:?}"));
        topics_to_subscribe.sort();
        topics_to_subscribe.dedup();
        topics_to_subscribe
    };

    consumer
        .subscribe(&topics_to_subscribe)
        .expect("Should subscribe to Kafka topics.");

    let mut nexus_engine = NexusEngine::new(Some(&args.file_name));

    let mut nexus_write_interval =
        tokio::time::interval(time::Duration::from_millis(args.cache_poll_interval_ms));

    loop {
        tokio::select! {
            _ = nexus_write_interval.tick() => {
                nexus_engine.flush(&Duration::try_milliseconds(args.cache_run_ttl_ms).unwrap())?
            }
            event = consumer.recv() => {
                match event {
                    Err(e) => {
                        trace_span!("Kafka Error").in_scope(||warn!("{e}"))
                    },
                    Ok(msg) => {
                        process_kafka_message(&mut nexus_engine, tracer.is_some(), &msg);
                        consumer.commit_message(&msg, CommitMode::Async).unwrap();
                    }
                }
            }
        }
    }
}

// Handles Run Span for
macro_rules! link_current_span_to_run_span {
    ($run:ident, $span_name:literal) => {
        let cur_span = tracing::Span::current();
        match $run.span().get() {
            Ok(run_span) => run_span.in_scope(|| {
                info_span!($span_name).follows_from(cur_span);
            }),
            Err(e) => debug!("No run found. Error: {e}"),
        }
    };
}

#[tracing::instrument(skip_all)]
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
    if digitizer_event_list_message_buffer_has_identifier(payload) {
        process_digitizer_event_list_message(nexus_engine, payload);
    } else if frame_assembled_event_list_message_buffer_has_identifier(payload) {
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
    }
}

fn process_digitizer_event_list_message(nexus_engine: &mut NexusEngine, payload: &[u8]) {
    match root_as_digitizer_event_list_message(payload) {
        Ok(data) => match GenericEventMessage::from_digitizer_event_list_message(data) {
            Ok(event_data) => match nexus_engine.process_event_list(&event_data) {
                Ok(run) => {
                    if let Some(run) = run {
                        link_current_span_to_run_span!(run, "Digitiser Event List");
                    }
                }
                Err(e) => warn!("Failed to save digitiser event list to file: {}", e),
            },
            Err(e) => error!("Digitiser event list message error: {}", e),
        },
        Err(e) => {
            warn!("Failed to parse message: {}", e);
        }
    }
}

fn process_frame_assembled_event_list_message(nexus_engine: &mut NexusEngine, payload: &[u8]) {
    match root_as_frame_assembled_event_list_message(payload) {
        Ok(data) => match GenericEventMessage::from_frame_assembled_event_list_message(data) {
            Ok(event_data) => match nexus_engine.process_event_list(&event_data) {
                Ok(run) => {
                    if let Some(run) = run {
                        link_current_span_to_run_span!(run, "Frame Event List");
                    }
                }
                Err(e) => warn!("Failed to save frame assembled event list to file: {}", e),
            },
            Err(e) => error!("Frame assembled event list message error: {}", e),
        },
        Err(e) => {
            warn!("Failed to parse message: {}", e);
        }
    }
}

fn process_run_start_message(nexus_engine: &mut NexusEngine, payload: &[u8]) {
    match root_as_run_start(payload) {
        Ok(data) => match nexus_engine.start_command(data) {
            Ok(run) => {
                run.span_mut().init(info_span!(target: "otel", parent: None, "Run")).unwrap();
                link_current_span_to_run_span!(run, "Run Start Command");
            }
            Err(e) => warn!("Start command ({data:?}) failed {e}"),
        },
        Err(e) => {
            warn!("Failed to parse message: {}", e);
        }
    }
}

fn process_run_stop_message(nexus_engine: &mut NexusEngine, payload: &[u8]) {
    match root_as_run_stop(payload) {
        Ok(data) => match nexus_engine.stop_command(data) {
            Ok(run) => {
                link_current_span_to_run_span!(run, "Run Stop Command");
            }
            Err(e) => warn!("Stop command ({data:?}) failed {e}"),
        },
        Err(e) => {
            warn!("Failed to parse message: {}", e);
        }
    }
}

fn process_sample_environment_message(nexus_engine: &mut NexusEngine, payload: &[u8]) {
    match root_as_se_00_sample_environment_data(payload) {
        Ok(data) => match nexus_engine.sample_envionment(data) {
            Ok(run) => {
                if let Some(run) = run {
                    link_current_span_to_run_span!(run, "Sample Environment Log");
                }
            }
            Err(e) => warn!("Sample environment ({data:?}) failed {e}"),
        },
        Err(e) => {
            warn!("Failed to parse message: {}", e);
        }
    }
}

fn process_alarm_message(nexus_engine: &mut NexusEngine, payload: &[u8]) {
    match root_as_alarm(payload) {
        Ok(data) => match nexus_engine.alarm(data) {
            Ok(run) => {
                if let Some(run) = run {
                    link_current_span_to_run_span!(run, "Alarm");
                }
            }
            Err(e) => warn!("Alarm ({data:?}) failed {e}"),
        },
        Err(e) => {
            warn!("Failed to parse message: {}", e);
        }
    }
}

fn process_logdata_message(nexus_engine: &mut NexusEngine, payload: &[u8]) {
    match root_as_f_144_log_data(payload) {
        Ok(data) => match nexus_engine.logdata(&data) {
            Ok(run) => {
                if let Some(run) = run {
                    link_current_span_to_run_span!(run, "Run Log Data");
                }
            }
            Err(e) => warn!("Run Log Data ({data:?}) failed. Error: {e}"),
        },
        Err(e) => {
            warn!("Failed to parse message: {}", e);
        }
    }
}

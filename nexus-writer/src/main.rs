mod event_message;
mod nexus;
mod spanned_run;

use anyhow::Result;
use chrono::Duration;
use clap::Parser;
use event_message::GenericEventMessage;
use nexus::Nexus;
use rdkafka::{
    consumer::{stream_consumer::StreamConsumer, CommitMode, Consumer},
    message::Message,
};
use spanned_run::SpannedRun;
use std::{net::SocketAddr, path::PathBuf};
use supermusr_common::tracer::OtelTracer;
use supermusr_streaming_types::{
    aev1_frame_assembled_event_v1_generated::{
        frame_assembled_event_list_message_buffer_has_identifier,
        root_as_frame_assembled_event_list_message,
    },
    dev1_digitizer_event_v1_generated::{
        digitizer_event_list_message_buffer_has_identifier, root_as_digitizer_event_list_message,
    },
    ecs_6s4t_run_stop_generated::{root_as_run_stop, run_stop_buffer_has_identifier},
    ecs_pl72_run_start_generated::{root_as_run_start, run_start_buffer_has_identifier},
};
use tokio::time;
use tracing::{debug, error, level_filters::LevelFilter, trace_span, warn, Span};

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

    #[clap(long, default_value = "127.0.0.1:9090")]
    observability_address: SocketAddr,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Cli::parse();

    let _tracer = init_tracer(args.otel_endpoint.as_deref());
    let root_span = trace_span!("Root");

    debug!("Args: {:?}", args);

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
    let topics_to_subscribe = [
        Some(args.control_topic.as_str()),
        args.digitiser_event_topic.as_deref(),
        args.frame_event_topic.as_deref(),
    ]
    .into_iter()
    .flatten()
    .collect::<Vec<&str>>();

    consumer.subscribe(&topics_to_subscribe)?;

    let mut nexus = Nexus::new(Some(args.file_name));

    let mut nexus_write_interval =
        tokio::time::interval(time::Duration::from_millis(args.cache_poll_interval_ms));

    loop {
        tokio::select! {
            _ = nexus_write_interval.tick() => {
                nexus.flush(&Duration::try_milliseconds(args.cache_run_ttl_ms).unwrap())?
            }
            event = consumer.recv() => {
                match event {
                    Err(e) => warn!("Kafka error: {}", e),
                    Ok(msg) => {
                        let span = trace_span!("Incoming Message");
                        let _guard = span.enter();
                        if args.otel_endpoint.is_some() {
                            if let Some(headers) = msg.headers() {
                                debug!("Kafka Header Found");
                                OtelTracer::extract_context_from_kafka_to_span(headers, &tracing::Span::current());
                            }
                        }
                        debug!(
                            "key: '{:?}', topic: {}, partition: {}, offset: {}, timestamp: {:?}",
                            msg.key(),
                            msg.topic(),
                            msg.partition(),
                            msg.offset(),
                            msg.timestamp()
                        );

                        if let Some(payload) = msg.payload() {
                            if args.digitiser_event_topic
                                .as_deref()
                                .map(|topic| msg.topic() == topic)
                                .unwrap_or(false)
                            {
                                if  digitizer_event_list_message_buffer_has_identifier(payload) {
                                    debug!("New digitizer event list.");
                                    process_digitizer_event_list_message(&mut nexus, payload);
                                } else {
                                    warn!("Incorrect message identifier on topic \"{}\"", msg.topic());
                                }
                            } else if args.frame_event_topic
                                .as_deref()
                                .map(|topic| msg.topic() == topic)
                                .unwrap_or(false)
                            {
                                if frame_assembled_event_list_message_buffer_has_identifier(payload) {
                                    debug!("New frame assembled event list.");
                                    process_frame_assembled_event_list_message(&mut nexus, payload);
                                } else {
                                    warn!("Incorrect message identifier on topic \"{}\"", msg.topic());
                                }
                            }
                            else if args.control_topic == msg.topic() {
                                if run_start_buffer_has_identifier(payload) {
                                    debug!("New run start.");
                                    process_run_start_message(&mut nexus, payload, &root_span);
                                } else if run_stop_buffer_has_identifier(payload) {
                                    debug!("New run stop.");
                                    process_run_stop_message(&mut nexus, payload);
                                } else {
                                    warn!("Incorrect message identifier on topic \"{}\"", msg.topic());
                                }
                            } else {
                                warn!("Unexpected message type on topic \"{}\"", msg.topic());
                            }
                        }
                        consumer.commit_message(&msg, CommitMode::Async).unwrap();
                    }
                }
            }
        }
    }
}

fn process_digitizer_event_list_message(nexus: &mut Nexus<SpannedRun>, payload: &[u8]) {
    match root_as_digitizer_event_list_message(payload) {
        Ok(data) => match GenericEventMessage::from_digitizer_event_list_message(data) {
            Ok(event_data) => match nexus.process_message(&event_data) {
                Ok(run) => {
                    if let Some(run) = run {
                        let cur_span = tracing::Span::current();
                        run.span.in_scope(|| {
                            let span = trace_span!("Digitiser Events List");
                            span.follows_from(cur_span);
                        });
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

fn process_frame_assembled_event_list_message(nexus: &mut Nexus<SpannedRun>, payload: &[u8]) {
    match root_as_frame_assembled_event_list_message(payload) {
        Ok(data) => match GenericEventMessage::from_frame_assembled_event_list_message(data) {
            Ok(event_data) => match nexus.process_message(&event_data) {
                Ok(run) => {
                    if let Some(run) = run {
                        let cur_span = tracing::Span::current();
                        run.span.in_scope(|| {
                            let span = trace_span!("Frame Events List");
                            span.follows_from(cur_span);
                        });
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

fn process_run_start_message(nexus: &mut Nexus<SpannedRun>, payload: &[u8], root_span: &Span) {
    match root_as_run_start(payload) {
        Ok(data) => match nexus.start_command(data) {
            Ok(run) => {
                let cur_span = tracing::Span::current();
                OtelTracer::set_span_parent_to(&run.span, root_span);
                run.span.in_scope(|| {
                    trace_span!("Run Start Command").follows_from(cur_span);
                });
            }
            Err(e) => warn!("Start command ({data:?}) failed {e}"),
        },
        Err(e) => {
            warn!("Failed to parse message: {}", e);
        }
    }
}

fn process_run_stop_message(nexus: &mut Nexus<SpannedRun>, payload: &[u8]) {
    match root_as_run_stop(payload) {
        Ok(data) => match nexus.stop_command(data) {
            Ok(run) => {
                let cur_span = tracing::Span::current();
                run.span.in_scope(|| {
                    let span = trace_span!("Run Stop Command");
                    span.follows_from(cur_span);
                });
            }
            Err(e) => warn!("Stop command ({data:?}) failed {e}"),
        },
        Err(e) => {
            warn!("Failed to parse message: {}", e);
        }
    }
}

fn init_tracer(otel_endpoint: Option<&str>) -> Option<OtelTracer> {
    otel_endpoint
        .map(|otel_endpoint| {
            OtelTracer::new(
                otel_endpoint,
                "Nexus Writer",
                Some(("nexus_writer", LevelFilter::TRACE)),
            )
            .expect("Open Telemetry Tracer is created")
        })
        .or_else(|| {
            tracing_subscriber::fmt::init();
            None
        })
}

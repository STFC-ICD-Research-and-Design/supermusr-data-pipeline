mod channels;
mod parameters;
mod processing;
mod pulse_detection;

use clap::Parser;
use metrics::counter;
use metrics_exporter_prometheus::PrometheusBuilder;
use parameters::{DetectorSettings, Mode, Polarity};
use rdkafka::{
    Message,
    consumer::{CommitMode, Consumer},
    message::{BorrowedHeaders, BorrowedMessage},
    producer::{DeliveryFuture, FutureProducer, FutureRecord},
};
use std::{net::SocketAddr, path::PathBuf};
use supermusr_common::{
    CommonKafkaOpts, Intensity, init_tracer,
    metrics::{
        failures::{self, FailureKind},
        messages_received::{self, MessageKind},
        metric_names::{FAILURES, MESSAGES_PROCESSED, MESSAGES_RECEIVED},
    },
    record_metadata_fields_to_span,
    tracer::{FutureRecordTracerExt, OptionalHeaderTracerExt, TracerEngine, TracerOptions},
};
use supermusr_streaming_types::{
    FrameMetadata,
    dat2_digitizer_analog_trace_v2_generated::{
        DigitizerAnalogTraceMessage, digitizer_analog_trace_message_buffer_has_identifier,
        root_as_digitizer_analog_trace_message,
    },
    flatbuffers::{FlatBufferBuilder, InvalidFlatbuffer},
};
use tokio::{
    select,
    signal::unix::{Signal, SignalKind, signal},
    sync::mpsc::{Receiver, Sender, error::TrySendError},
    task::JoinHandle,
};
use tracing::{debug, error, info, instrument, trace, warn};

type DigitiserEventListToBufferSender = Sender<DeliveryFuture>;
type TrySendDigitiserEventListError = TrySendError<DeliveryFuture>;

const EVENTS_FOUND_METRIC: &str = "events_found";

#[derive(Debug, Parser)]
#[clap(author, version = supermusr_common::version!(), about)]
struct Cli {
    #[clap(flatten)]
    common_kafka_options: CommonKafkaOpts,

    /// Kafka consumer group
    #[clap(long)]
    consumer_group: String,

    /// The Kafka topic that trace messages are consumed from
    #[clap(long)]
    trace_topic: String,

    /// Topic to publish digitiser event messages to
    #[clap(long)]
    event_topic: String,

    /// Determines whether events should register as positive or negative intensity
    #[clap(long)]
    polarity: Polarity,

    /// Value of the intensity baseline
    #[clap(long, default_value = "0")]
    baseline: Intensity,

    /// Size of the send eventlist buffer.
    /// If this limit is exceeded, the component will exit.
    #[clap(long, default_value = "1024")]
    send_eventlist_buffer_size: usize,

    /// Endpoint on which OpenMetrics flavour metrics are available
    #[clap(long, env, default_value = "127.0.0.1:9090")]
    observability_address: SocketAddr,

    /// If set, the trace and event lists are saved here
    #[clap(long)]
    save_file: Option<PathBuf>,

    /// If set, then OpenTelemetry data is sent to the URL specified, otherwise the standard tracing subscriber is used
    #[clap(long)]
    otel_endpoint: Option<String>,

    /// All OpenTelemetry spans are emitted with this as the "service.namespace" property. Can be used to track different instances of the pipeline running in parallel.
    #[clap(long, default_value = "")]
    otel_namespace: String,

    #[command(subcommand)]
    pub(crate) mode: Mode,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Cli::parse();

    let tracer = init_tracer!(TracerOptions::new(
        args.otel_endpoint.as_deref(),
        args.otel_namespace.clone()
    ));

    let kafka_opts = &args.common_kafka_options;

    let client_config = supermusr_common::generate_kafka_client_config(
        &kafka_opts.broker,
        &kafka_opts.username,
        &kafka_opts.password,
    );

    let producer: FutureProducer = client_config.create()?;

    let consumer = supermusr_common::create_default_consumer(
        &kafka_opts.broker,
        &kafka_opts.username,
        &kafka_opts.password,
        &args.consumer_group,
        Some(&[args.trace_topic.as_str()]),
    )?;

    // Install exporter and register metrics
    let builder = PrometheusBuilder::new();
    builder
        .with_http_listener(args.observability_address)
        .install()?;

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
    metrics::describe_counter!(
        EVENTS_FOUND_METRIC,
        metrics::Unit::Count,
        "Number of events found per channel"
    );

    let (sender, producer_task_handle) = create_producer_task(args.send_eventlist_buffer_size)?;

    // Is used to await any sigint signals
    let mut sigint = signal(SignalKind::interrupt())?;

    loop {
        tokio::select! {
            msg = consumer.recv() => match msg {
                Ok(m) => {
                    process_kafka_message(
                        &tracer,
                        &args,
                        &sender,
                        &producer,
                        &m,
                    )?;
                    consumer.commit_message(&m, CommitMode::Async).unwrap();
                }
                Err(e) => warn!("Kafka error: {}", e)
            },
            _ = sigint.recv() => {
                //  Wait for the channel to close and
                //  all pending production tasks to finish
                producer_task_handle.await?;
                return Ok(());
            }
        }
    }
}

#[instrument(skip_all, level = "trace", err(level = "warn"))]
fn spanned_root_as_digitizer_analog_trace_message(
    payload: &[u8],
) -> Result<DigitizerAnalogTraceMessage<'_>, InvalidFlatbuffer> {
    root_as_digitizer_analog_trace_message(payload)
}

#[instrument(skip_all, level = "debug", err(level = "warn"))]
fn process_kafka_message(
    tracer: &TracerEngine,
    args: &Cli,
    sender: &DigitiserEventListToBufferSender,
    producer: &FutureProducer,
    m: &BorrowedMessage,
) -> Result<(), TrySendDigitiserEventListError> {
    debug!(
        "key: '{:?}', topic: {}, partition: {}, offset: {}, timestamp: {:?}",
        m.key(),
        m.topic(),
        m.partition(),
        m.offset(),
        m.timestamp()
    );

    if let Some(payload) = m.payload() {
        if digitizer_analog_trace_message_buffer_has_identifier(payload) {
            counter!(
                MESSAGES_RECEIVED,
                &[messages_received::get_label(MessageKind::Trace)]
            )
            .increment(1);
            match spanned_root_as_digitizer_analog_trace_message(payload) {
                Ok(data) => {
                    let kafka_timestamp_ms = m.timestamp().to_millis().unwrap_or(-1);
                    process_digitiser_trace_message(
                        tracer,
                        m.headers(),
                        args,
                        sender,
                        producer,
                        kafka_timestamp_ms,
                        data,
                    )?
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
        } else {
            warn!("Unexpected message type on topic \"{}\"", m.topic());
            counter!(
                MESSAGES_RECEIVED,
                &[messages_received::get_label(MessageKind::Unexpected)]
            )
            .increment(1);
        }
    }
    Ok(())
}

#[instrument(
    skip_all,
    fields(
        digitiser_id = message.digitizer_id(),
        kafka_message_timestamp_ms = kafka_timestamp_ms,
        metadata_timestamp,
        metadata_frame_number,
        metadata_period_number,
        metadata_veto_flags,
        metadata_protons_per_pulse,
        metadata_running,
    )
)]
fn process_digitiser_trace_message(
    tracer: &TracerEngine,
    headers: Option<&BorrowedHeaders>,
    args: &Cli,
    sender: &DigitiserEventListToBufferSender,
    producer: &FutureProducer,
    kafka_timestamp_ms: i64,
    message: DigitizerAnalogTraceMessage,
) -> Result<(), TrySendDigitiserEventListError> {
    message
        .metadata()
        .try_into()
        .inspect(|metadata: &FrameMetadata| {
            record_metadata_fields_to_span!(metadata, tracing::Span::current());
        })
        .ok();

    headers.conditional_extract_to_current_span(tracer.use_otel());
    let mut fbb = FlatBufferBuilder::new();
    processing::process(
        &mut fbb,
        &message,
        &DetectorSettings {
            polarity: &args.polarity,
            baseline: args.baseline,
            mode: &args.mode,
        },
        args.save_file.as_deref(),
    );

    let future_record = FutureRecord::to(&args.event_topic)
        .payload(fbb.finished_data())
        .conditional_inject_current_span_into_headers(tracer.use_otel())
        .key("Digitiser Events List");

    let future = producer.send_result(future_record).expect("Producer sends");

    if let Err(e) = sender.try_send(future) {
        match &e {
            TrySendError::Closed(_) => {
                error!("Send-Frame Channel Closed");
            }
            TrySendError::Full(_) => {
                error!("Send-Frame Buffer Full");
            }
        }
        Err(e)
    } else {
        Ok(())
    }
}

// The following functions control the kafka producer thread
fn create_producer_task(
    send_digitiser_eventlist_buffer_size: usize,
) -> std::io::Result<(DigitiserEventListToBufferSender, JoinHandle<()>)> {
    let (channel_send, channel_recv) =
        tokio::sync::mpsc::channel::<DeliveryFuture>(send_digitiser_eventlist_buffer_size);

    let sigint = signal(SignalKind::interrupt())?;
    let handle = tokio::spawn(produce_to_kafka(channel_recv, sigint));
    Ok((channel_send, handle))
}

async fn produce_to_kafka(mut channel_recv: Receiver<DeliveryFuture>, mut sigint: Signal) {
    loop {
        // Blocks until a frame is received
        select! {
            message = channel_recv.recv() => {
                match message {
                    Some(future) => {
                        produce_eventlist_to_kafka(future).await
                    },
                    None => {
                        info!("Send-Eventlist channel closed");
                        return;
                    }
                }
            },
            _ = sigint.recv() => {
                close_and_flush_producer_channel(&mut channel_recv).await;
            }
        }
    }
}

async fn produce_eventlist_to_kafka(future: DeliveryFuture) {
    match future.await {
        Ok(_) => {
            trace!("Published event message");
            counter!(MESSAGES_PROCESSED).increment(1);
        }
        Err(e) => {
            error!("{:?}", e);
            counter!(
                FAILURES,
                &[failures::get_label(FailureKind::KafkaPublishFailed)]
            )
            .increment(1);
        }
    }
}

#[tracing::instrument(skip_all, name = "Closing", level = "info", fields(capactity = channel_recv.capacity(), max_capactity = channel_recv.max_capacity()))]
async fn close_and_flush_producer_channel(
    channel_recv: &mut Receiver<DeliveryFuture>,
) -> Option<()> {
    channel_recv.close();

    loop {
        let future = channel_recv.recv().await?;
        flush_eventlist(future).await?;
    }
}

#[tracing::instrument(skip_all, name = "Flush Eventlist")]
async fn flush_eventlist(future: DeliveryFuture) -> Option<()> {
    produce_eventlist_to_kafka(future).await;
    Some(())
}

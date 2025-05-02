mod data;
mod frame;

use crate::data::EventData;
use clap::Parser;
use frame::{AggregatedFrame, FrameCache};
use metrics::counter;
use metrics_exporter_prometheus::PrometheusBuilder;
use rdkafka::{
    consumer::{CommitMode, Consumer},
    message::{BorrowedMessage, Message},
    producer::{FutureProducer, FutureRecord},
    util::Timeout,
};
use std::{fmt::Debug, net::SocketAddr, time::Duration};
use supermusr_common::{
    init_tracer,
    metrics::{
        failures::{self, FailureKind},
        messages_received::{self, MessageKind},
        metric_names::{FAILURES, FRAMES_SENT, MESSAGES_PROCESSED, MESSAGES_RECEIVED},
    },
    record_metadata_fields_to_span,
    spanned::Spanned,
    tracer::{FutureRecordTracerExt, OptionalHeaderTracerExt, TracerEngine, TracerOptions},
    CommonKafkaOpts, DigitizerId,
};
use supermusr_streaming_types::{
    dev2_digitizer_event_v2_generated::{
        digitizer_event_list_message_buffer_has_identifier, root_as_digitizer_event_list_message,
        DigitizerEventListMessage,
    },
    flatbuffers::InvalidFlatbuffer,
};
use tokio::{
    select,
    signal::unix::{signal, Signal, SignalKind},
    sync::mpsc::{error::SendError, Receiver, Sender},
    task::JoinHandle,
};
use tracing::{debug, error, info, info_span, instrument, warn};

const PRODUCER_TIMEOUT: Timeout = Timeout::After(Duration::from_millis(100));

type AggregatedFrameToBufferSender = Sender<AggregatedFrame<EventData>>;
type SendAggregatedFrameError = SendError<AggregatedFrame<EventData>>;

#[derive(Debug, Parser)]
#[clap(author, version, about)]
struct Cli {
    #[clap(flatten)]
    common_kafka_options: CommonKafkaOpts,

    /// Kafka consumer group
    #[clap(long = "group")]
    consumer_group: String,

    /// Kafka topic on which to listen for per digitiser event messages
    #[clap(long)]
    input_topic: String,

    /// Kafka topic on which to emit frame assembled event messages
    #[clap(long)]
    output_topic: String,

    /// A list of expected digitiser IDs.
    /// Can be passed as `-d0 -d1 ...` or `-d=0,1,...`
    /// A frame is only "complete" when a message has been received from each of these IDs.
    #[clap(short, long, value_delimiter = ',')]
    digitiser_ids: Vec<DigitizerId>,

    /// Frame TTL in milliseconds.
    /// The time in which messages for a given frame must have been received from all digitisers.
    #[clap(long, default_value = "500")]
    frame_ttl_ms: u64,

    /// Frame cache poll interval in milliseconds.
    /// This may affect the rate at which incomplete frames are transmitted.
    #[clap(long, default_value = "500")]
    cache_poll_ms: u64,

    /// Size of the send frame buffer.
    /// If this limit is exceeded, the component will exit.
    #[clap(long, default_value = "1024")]
    send_frame_buffer_size: usize,

    /// Endpoint on which Prometheus text format metrics are available
    #[clap(long, env, default_value = "127.0.0.1:9090")]
    observability_address: SocketAddr,

    /// If set, then OpenTelemetry data is sent to the URL specified, otherwise the standard tracing subscriber is used
    #[clap(long)]
    otel_endpoint: Option<String>,

    /// All OpenTelemetry spans are emitted with this as the "service.namespace" property. Can be used to track different instances of the pipeline running in parallel.
    #[clap(long, default_value = "")]
    otel_namespace: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Cli::parse();

    let tracer = init_tracer!(TracerOptions::new(
        args.otel_endpoint.as_deref(),
        args.otel_namespace
    ));

    let kafka_opts = args.common_kafka_options;

    let consumer = supermusr_common::create_default_consumer(
        &kafka_opts.broker,
        &kafka_opts.username,
        &kafka_opts.password,
        &args.consumer_group,
        Some(&[args.input_topic.as_str()])
    )?;

    let producer: FutureProducer = supermusr_common::generate_kafka_client_config(
        &kafka_opts.broker,
        &kafka_opts.username,
        &kafka_opts.password,
    )
    .create()?;

    let ttl = Duration::from_millis(args.frame_ttl_ms);

    let mut cache = FrameCache::<EventData>::new(ttl, args.digitiser_ids.clone());

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
    metrics::describe_counter!(
        FRAMES_SENT,
        metrics::Unit::Count,
        "Number of complete frames sent by the aggregator"
    );

    let mut cache_poll_interval = tokio::time::interval(Duration::from_millis(args.cache_poll_ms));

    // Creates Send-Frame thread and returns channel sender
    let (channel_send, producer_task_handle) = create_producer_task(
        tracer.use_otel(),
        args.send_frame_buffer_size,
        &producer,
        &args.output_topic,
    )?;

    // Is used to await any sigint signals
    let mut sigint = signal(SignalKind::interrupt())?;

    loop {
        tokio::select! {
            event = consumer.recv() => {
                match event {
                    Ok(msg) => {
                        process_kafka_message(tracer.use_otel(), &channel_send, &mut cache, &msg).await?;
                        consumer.commit_message(&msg, CommitMode::Async)
                            .expect("Message should commit");
                    }
                    Err(e) => warn!("Kafka error: {}", e),
                };
            }
            _ = cache_poll_interval.tick() => {
                cache_poll(&channel_send, &mut cache).await?;
            }
            _ = sigint.recv() => {
                //  Wait for the channel to close and
                //  all pending production tasks to finish
                producer_task_handle.await?;
                return Ok(());
            }
        }
    }
}

///  This function wraps the `root_as_digitizer_event_list_message` function, allowing it to be instrumented.
#[instrument(skip_all, level = "trace", err(level = "warn"))]
fn spanned_root_as_digitizer_event_list_message(
    payload: &[u8],
) -> Result<DigitizerEventListMessage<'_>, InvalidFlatbuffer> {
    root_as_digitizer_event_list_message(payload)
}

#[instrument(skip_all, level = "debug", err(level = "warn"))]
async fn process_kafka_message(
    use_otel: bool,
    channel_send: &AggregatedFrameToBufferSender,
    cache: &mut FrameCache<EventData>,
    msg: &BorrowedMessage<'_>,
) -> Result<(), SendAggregatedFrameError> {
    msg.headers().conditional_extract_to_current_span(use_otel);

    if let Some(payload) = msg.payload() {
        if digitizer_event_list_message_buffer_has_identifier(payload) {
            counter!(
                MESSAGES_RECEIVED,
                &[messages_received::get_label(MessageKind::Event)]
            )
            .increment(1);
            match spanned_root_as_digitizer_event_list_message(payload) {
                Ok(data) => {
                    let kafka_timestamp_ms = msg.timestamp().to_millis().unwrap_or(-1);
                    process_digitiser_event_list_message(
                        channel_send,
                        cache,
                        kafka_timestamp_ms,
                        data,
                    )
                    .await?;
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
            warn!("Unexpected message type on topic \"{}\"", msg.topic());
            debug!("Message: {msg:?}");
            debug!("Payload size: {}", payload.len());
            counter!(
                MESSAGES_RECEIVED,
                &[messages_received::get_label(MessageKind::Unexpected)]
            )
            .increment(1);
        }
    }
    Ok(())
}

#[tracing::instrument(skip_all, fields(
    digitiser_id = message.digitizer_id(),
    kafka_message_timestamp_ms=kafka_message_timestamp_ms,
    metadata_timestamp,
    metadata_frame_number,
    metadata_period_number,
    metadata_veto_flags,
    metadata_protons_per_pulse,
    metadata_running,
    num_cached_frames = cache.get_num_partial_frames(),
    timestamp_too_early = false,
    id_already_present = false,
))]
async fn process_digitiser_event_list_message(
    channel_send: &AggregatedFrameToBufferSender,
    cache: &mut FrameCache<EventData>,
    kafka_message_timestamp_ms: i64,
    message: DigitizerEventListMessage<'_>,
) -> Result<(), SendAggregatedFrameError> {
    match message.metadata().try_into() {
        Ok(metadata) => {
            debug!("Event packet: metadata: {:?}", message.metadata());

            // Push the current digitiser message to the frame cache, possibly creating a new partial frame
            if let Err(err) = cache.push(message.digitizer_id(), &metadata, message.into()) {
                tracing::Span::current().record(err.into(), true);
            }

            record_metadata_fields_to_span!(&metadata, tracing::Span::current());

            cache_poll(channel_send, cache).await?;
        }
        Err(e) => {
            warn!("Invalid Metadata: {e}");
            counter!(
                FAILURES,
                &[failures::get_label(FailureKind::InvalidMetadata)]
            )
            .increment(1);
        }
    }
    Ok(())
}

#[tracing::instrument(skip_all, level = "trace")]
async fn cache_poll(
    channel_send: &AggregatedFrameToBufferSender,
    cache: &mut FrameCache<EventData>,
) -> Result<(), SendAggregatedFrameError> {
    while let Some(frame) = cache.poll() {
        let span = info_span!("Frame Completed");
        span.follows_from(
            frame
                .span()
                .get()
                .expect("Span should exist, this should never fail"),
        );
        let _guard = span.enter();

        // Reserves space in the message queue if it is available
        // Or waits for space if none is available.
        match channel_send.reserve().await {
            Ok(permit) => permit.send(frame),
            Err(_) => {
                error!("Send-Frame Error");
                return Err(SendError(frame));
            }
        }
    }
    Ok(())
}

// The following functions control the kafka producer thread
fn create_producer_task(
    use_otel: bool,
    send_frame_buffer_size: usize,
    producer: &FutureProducer,
    output_topic: &str,
) -> std::io::Result<(AggregatedFrameToBufferSender, JoinHandle<()>)> {
    let (channel_send, channel_recv) =
        tokio::sync::mpsc::channel::<AggregatedFrame<EventData>>(send_frame_buffer_size);

    let sigint = signal(SignalKind::interrupt())?;
    let handle = tokio::spawn(produce_to_kafka(
        use_otel,
        channel_recv,
        producer.to_owned(),
        output_topic.to_owned(),
        sigint,
    ));
    Ok((channel_send, handle))
}

async fn produce_to_kafka(
    use_otel: bool,
    mut channel_recv: Receiver<AggregatedFrame<EventData>>,
    producer: FutureProducer,
    output_topic: String,
    mut sigint: Signal,
) {
    loop {
        select! {
            message = channel_recv.recv() => {
                // Blocks until a frame is received
                match message {
                    Some(frame) => {
                        produce_frame_to_kafka(use_otel, frame, &producer, &output_topic).await;
                    }
                    None => {
                        info!("Send-Frame channel closed");
                        return;
                    }
                }
            }
            _ = sigint.recv() => {
                close_and_flush_producer_channel(use_otel,&mut channel_recv,&producer,&output_topic).await;
            }
        }
    }
}

#[tracing::instrument(skip_all, name = "Closing", level = "info", fields(capacity = channel_recv.capacity(), max_capacity = channel_recv.max_capacity()))]
async fn close_and_flush_producer_channel(
    use_otel: bool,
    channel_recv: &mut Receiver<AggregatedFrame<EventData>>,
    producer: &FutureProducer,
    output_topic: &str,
) -> Option<()> {
    channel_recv.close();

    loop {
        let frame = channel_recv.recv().await?;
        flush_frame(use_otel, frame, producer, output_topic).await?;
    }
}

#[tracing::instrument(skip_all, name = "Flush Frame")]
async fn flush_frame(
    use_otel: bool,
    frame: AggregatedFrame<EventData>,
    producer: &FutureProducer,
    output_topic: &str,
) -> Option<()> {
    produce_frame_to_kafka(use_otel, frame, producer, output_topic).await;
    Some(())
}

async fn produce_frame_to_kafka(
    use_otel: bool,
    frame: AggregatedFrame<EventData>,
    producer: &FutureProducer,
    output_topic: &str,
) {
    let frame_span = frame.span().get().expect("Span should exist").clone();
    let data: Vec<u8> = frame.into();

    let future_record = FutureRecord::to(output_topic)
        .payload(data.as_slice())
        .conditional_inject_span_into_headers(use_otel, &frame_span)
        .key("Frame Events List");

    match producer.send(future_record, PRODUCER_TIMEOUT).await {
        Ok(r) => {
            debug!("Delivery: {:?}", r);
            counter!(FRAMES_SENT).increment(1)
        }
        Err(e) => {
            error!("Delivery failed: {:?}", e);
            counter!(
                FAILURES,
                &[failures::get_label(FailureKind::KafkaPublishFailed)]
            )
            .increment(1);
        }
    }
}

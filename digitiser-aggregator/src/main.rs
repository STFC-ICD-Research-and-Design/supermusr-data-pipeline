mod data;
mod frame;

use crate::data::EventData;
use clap::Parser;
use frame::FrameCache;
use metrics::counter;
use metrics_exporter_prometheus::PrometheusBuilder;
use rdkafka::{
    consumer::{CommitMode, Consumer},
    message::{BorrowedHeaders, BorrowedMessage, Message},
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
use tokio::task::JoinSet;
use tracing::{debug, error, info_span, instrument, level_filters::LevelFilter, warn, Instrument};

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
    /// A frame is only "complete" when a message has been received from each of these IDs.
    #[clap(short, long)]
    digitiser_ids: Vec<DigitizerId>,

    /// Frame TTL in milliseconds.
    /// The time in which messages for a given frame must have been received from all digitisers.
    #[clap(long, default_value = "500")]
    frame_ttl_ms: u64,

    /// Frame cache poll interval in milliseconds.
    /// This may affect the rate at which incomplete frames are transmitted.
    #[clap(long, default_value = "500")]
    cache_poll_ms: u64,

    /// Endpoint on which Prometheus text format metrics are available
    #[clap(long, env, default_value = "127.0.0.1:9090")]
    observability_address: SocketAddr,

    /// If set, then OpenTelemetry data is sent to the URL specified, otherwise the standard tracing subscriber is used
    #[clap(long)]
    otel_endpoint: Option<String>,

    /// The reporting level to use for OpenTelemetry
    #[clap(long, default_value = "info")]
    otel_level: LevelFilter,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Cli::parse();

    let tracer = init_tracer!(TracerOptions::new(
        args.otel_endpoint.as_deref(),
        args.otel_level
    ));

    let kafka_opts = args.common_kafka_options;

    let consumer = supermusr_common::create_default_consumer(
        &kafka_opts.broker,
        &kafka_opts.username,
        &kafka_opts.password,
        &args.consumer_group,
        &[args.input_topic.as_str()],
    );

    let producer = supermusr_common::generate_kafka_client_config(
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

    let mut kafka_producer_thread_set = JoinSet::new();

    let mut cache_poll_interval = tokio::time::interval(Duration::from_millis(args.cache_poll_ms));
    loop {
        tokio::select! {
            event = consumer.recv() => {
                match event {
                    Ok(msg) => {
                        process_kafka_message(tracer.use_otel(), &mut kafka_producer_thread_set, &mut cache, &producer, &args.output_topic, &msg).await;
                        consumer.commit_message(&msg, CommitMode::Async)
                            .expect("Message should commit");
                    }
                    Err(e) => warn!("Kafka error: {}", e),
                };
            }
            _ = cache_poll_interval.tick() => {
                cache_poll(tracer.use_otel(), &mut kafka_producer_thread_set, &mut cache, &producer, &args.output_topic).await;
            }
        }
    }
}

///  This function wraps the `root_as_digitizer_event_list_message` function, allowing it to be instrumented.
#[instrument(skip_all, target = "otel")]
fn spanned_root_as_digitizer_event_list_message(
    payload: &[u8],
) -> Result<DigitizerEventListMessage<'_>, InvalidFlatbuffer> {
    root_as_digitizer_event_list_message(payload)
}

#[instrument(skip_all, level = "info", fields(kafka_message_timestamp_ms = msg.timestamp().to_millis()))]
async fn process_kafka_message(
    use_otel: bool,
    kafka_producer_thread_set: &mut JoinSet<()>,
    cache: &mut FrameCache<EventData>,
    producer: &FutureProducer,
    output_topic: &str,
    msg: &BorrowedMessage<'_>,
) {
    if let Some(payload) = msg.payload() {
        if digitizer_event_list_message_buffer_has_identifier(payload) {
            counter!(
                MESSAGES_RECEIVED,
                &[messages_received::get_label(MessageKind::Event)]
            )
            .increment(1);
            let headers = msg.headers();
            match spanned_root_as_digitizer_event_list_message(payload) {
                Ok(msg) => {
                    process_digitiser_event_list_message(
                        use_otel,
                        headers,
                        kafka_producer_thread_set,
                        cache,
                        producer,
                        output_topic,
                        msg,
                    )
                    .await;
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
}

#[tracing::instrument(skip_all, fields(
    digitiser_id = msg.digitizer_id(),
    metadata_timestamp = tracing::field::Empty,
    metadata_frame_number = tracing::field::Empty,
    metadata_period_number = tracing::field::Empty,
    metadata_veto_flags = tracing::field::Empty,
    metadata_protons_per_pulse = tracing::field::Empty,
    metadata_running = tracing::field::Empty,
    num_cached_frames = cache.get_num_partial_frames(),
))]
async fn process_digitiser_event_list_message(
    use_otel: bool,
    headers: Option<&BorrowedHeaders>,
    kafka_producer_thread_set: &mut JoinSet<()>,
    cache: &mut FrameCache<EventData>,
    producer: &FutureProducer,
    output_topic: &str,
    msg: DigitizerEventListMessage<'_>,
) {
    match msg.metadata().try_into() {
        Ok(metadata) => {
            debug!("Event packet: metadata: {:?}", msg.metadata());

            // Push the current digitiser message to the frame cache, possibly creating a new partial frame
            cache.push(msg.digitizer_id(), &metadata, msg.into());

            record_metadata_fields_to_span!(&metadata, tracing::Span::current());
            headers.conditional_extract_to_current_span(use_otel);

            cache_poll(
                use_otel,
                kafka_producer_thread_set,
                cache,
                producer,
                output_topic,
            )
            .await;
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
}

#[tracing::instrument(skip_all, level = "trace")]
async fn cache_poll(
    use_otel: bool,
    kafka_producer_thread_set: &mut JoinSet<()>,
    cache: &mut FrameCache<EventData>,
    producer: &FutureProducer,
    output_topic: &str,
) {
    if let Some(frame) = cache.poll() {
        let span = info_span!(target: "otel", "Frame Complete");
        let future = span.in_scope(|| {
            let frame_span = frame.span().get().expect("Span should exist").clone();
            let data: Vec<u8> = frame.into();

            let producer = producer.to_owned();
            let output_topic = output_topic.to_owned();
            async move {
                let future_record = FutureRecord::to(&output_topic)
                    .payload(data.as_slice())
                    .conditional_inject_span_into_headers(use_otel, &frame_span)
                    .key("Frame Events List");

                match producer
                    .send(future_record, Timeout::After(Duration::from_millis(100)))
                    .await
                {
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
        });
        kafka_producer_thread_set.spawn(
            future.instrument(info_span!(target: "otel", parent: &span, "Message Producer")),
        );
    }
}

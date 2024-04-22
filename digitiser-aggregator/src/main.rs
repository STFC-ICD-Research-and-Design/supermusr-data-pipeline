mod data;
mod frame;
mod spanned_frame;

use crate::data::EventData;
use clap::Parser;
use rdkafka::{
    consumer::{stream_consumer::StreamConsumer, CommitMode, Consumer},
    message::{BorrowedMessage, Message, OwnedHeaders},
    producer::{FutureProducer, FutureRecord},
    util::Timeout,
};
use std::{fmt::Debug, net::SocketAddr, time::Duration};
use supermusr_common::{tracer::OtelTracer, DigitizerId};
use supermusr_streaming_types::dev1_digitizer_event_v1_generated::{
    digitizer_event_list_message_buffer_has_identifier, root_as_digitizer_event_list_message,
};
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

    #[clap(long = "group")]
    consumer_group: String,

    #[clap(long)]
    input_topic: String,

    #[clap(long)]
    output_topic: String,

    #[clap(short, long)]
    digitiser_ids: Vec<DigitizerId>,

    #[clap(long, default_value = "500")]
    frame_ttl_ms: u64,

    #[clap(long, default_value = "500")]
    cache_poll_ms: u64,

    #[clap(long, default_value = "127.0.0.1:9090")]
    observability_address: SocketAddr,

    /// If set, then open-telemetry data is sent to the URL specified, otherwise the standard tracing subscriber is used
    #[clap(long)]
    otel_endpoint: Option<String>,
}

type FrameCache<D> = spanned_frame::SpannedFrameCache<D>;

#[tokio::main]
async fn main() {
    let args = Cli::parse();

    let _tracer = init_tracer(args.otel_endpoint.as_deref());
    let root_span = trace_span!("Root");

    let consumer: StreamConsumer = supermusr_common::generate_kafka_client_config(
        &args.broker,
        &args.username,
        &args.password,
    )
    .set("group.id", &args.consumer_group)
    .set("enable.partition.eof", "false")
    .set("session.timeout.ms", "6000")
    .set("enable.auto.commit", "false")
    .create()
    .expect("kafka consumer should be created");

    consumer
        .subscribe(&[&args.input_topic])
        .expect("kafka topic should be subscribed");

    let producer = supermusr_common::generate_kafka_client_config(
        &args.broker,
        &args.username,
        &args.password,
    )
    .create()
    .unwrap();

    let ttl = Duration::from_millis(args.frame_ttl_ms);

    let mut cache = FrameCache::<EventData>::new(ttl, args.digitiser_ids.clone());

    let mut cache_poll_interval = tokio::time::interval(Duration::from_millis(args.cache_poll_ms));
    loop {
        tokio::select! {
            event = consumer.recv() => {
                match event {
                    Ok(msg) => {
                        on_message(&root_span, &args, &mut cache, &producer, &msg).await;
                        consumer.commit_message(&msg, CommitMode::Async).unwrap();
                    }
                    Err(e) => warn!("Kafka error: {}", e),
                };
            }
            _ = cache_poll_interval.tick() => {
                cache_poll(&args, &mut cache, &producer).await;
            }
        }
    }
}

async fn on_message(
    root_span: &Span,
    args: &Cli,
    cache: &mut FrameCache<EventData>,
    producer: &FutureProducer,
    msg: &BorrowedMessage<'_>,
) {
    let span = trace_span!("Event Formation Message");
    let _guard = span.enter();
    
    if args.otel_endpoint.is_some() {
        if let Some(headers) = msg.headers() {
            debug!("Kafka Header Found");
            OtelTracer::extract_context_from_kafka_to_span(headers, &tracing::Span::current());
        }
    }

    if let Some(payload) = msg.payload() {
        if digitizer_event_list_message_buffer_has_identifier(payload) {
            match root_as_digitizer_event_list_message(payload) {
                Ok(msg) => {
                    debug!("Event packet: metadata: {:?}", msg.metadata());
                    cache.push(
                        msg.digitizer_id(),
                        msg.metadata().try_into().unwrap(),
                        msg.into(),
                    );
                    if let Some(frame) = cache.find(msg.metadata().try_into().unwrap()) {
                        OtelTracer::set_span_parent_to(&frame.span, root_span);
                        let cur_span = tracing::Span::current();
                        frame.span.in_scope(|| {
                            let span = trace_span!("Digitiser Event List");
                            span.follows_from(cur_span);
                        });
                    }
                    cache_poll(args, cache, producer).await;
                }
                Err(e) => {
                    warn!("Failed to parse message: {}", e);
                }
            }
        } else {
            warn!("Unexpected message type on topic \"{}\"", msg.topic());
        }
    }
}

async fn cache_poll(
    args: &Cli,
    cache: &mut FrameCache<EventData>,
    producer: &FutureProducer,
) {
    if let Some(frame) = cache.poll() {
        let data: Vec<u8> = frame.value.into();

        let future_record = {
            if args.otel_endpoint.is_some() {
                let mut headers = OwnedHeaders::new();
                OtelTracer::inject_context_from_span_into_kafka(&frame.span, &mut headers);

                FutureRecord::to(&args.output_topic)
                    .payload(&data)
                    .headers(headers)
                    .key("FrameAssembledEventsList")
            } else {
                FutureRecord::to(&args.output_topic)
                    .payload(&data)
                    .key("FrameAssembledEventsList")
            }
        };

        match producer
            .send(future_record, Timeout::After(Duration::from_millis(100)))
            .await
        {
            Ok(r) => debug!("Delivery: {:?}", r),
            Err(e) => error!("Delivery failed: {:?}", e),
        };
    }
}

fn init_tracer(otel_endpoint: Option<&str>) -> Option<OtelTracer> {
    otel_endpoint
        .map(|otel_endpoint| {
            OtelTracer::new(
                otel_endpoint,
                "Digitiser Aggregator",
                Some(("digitiser_aggregator", LevelFilter::TRACE)),
            )
            .expect("Open Telemetry Tracer is created")
        })
        .or_else(|| {
            tracing_subscriber::fmt::init();
            None
        })
}

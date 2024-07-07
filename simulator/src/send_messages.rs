use rdkafka::{
    producer::{FutureProducer, FutureRecord},
    util::Timeout,
};
use std::time::Duration;
use supermusr_common::{tracer::FutureRecordTracerExt, Channel, DigitizerId};
use supermusr_streaming_types::{
    flatbuffers::FlatBufferBuilder, frame_metadata_v2_generated::GpsTime, FrameMetadata,
};
use tokio::task::JoinSet;
use tracing::{debug, debug_span, error, Span};

use crate::integrated::engine::Cache;
use crate::integrated::{
    build_messages::build_trace_message, schedule::Source, simulation::Simulation,
};
use anyhow::Result;

struct SendMessageArgs<'a> {
    use_otel: bool,
    producer: FutureProducer,
    fbb: FlatBufferBuilder<'a>,
    topic: String,
    span: Span,
    key: &'static str,
}

impl<'a> SendMessageArgs<'a> {
    fn new(
        use_otel: bool,
        fbb: FlatBufferBuilder<'a>,
        producer: &FutureProducer,
        topic: &str,
        key: &'static str,
    ) -> Self {
        Self {
            use_otel,
            fbb,
            producer: producer.to_owned(),
            topic: topic.to_owned(),
            span: tracing::Span::current(),
            key,
        }
    }
}

async fn send_message(args: SendMessageArgs<'_>) {
    let span = debug_span!(parent: &args.span, "Send Message Thread");
    let _guard = span.enter();

    let future_record = FutureRecord::to(&args.topic)
        .payload(args.fbb.finished_data())
        .conditional_inject_span_into_headers(args.use_otel, &args.span)
        .key(args.key);

    let timeout = Timeout::After(Duration::from_millis(100));
    match args.producer.send(future_record, timeout).await {
        Ok(r) => debug!("Delivery: {:?}", r),
        Err(e) => error!("Delivery failed: {:?}", e.0),
    };
}

#[tracing::instrument(skip_all, target = "otel", fields(digitizer_id = digitizer_id))]
pub(crate) fn send_trace_message(
    use_otel: bool,
    producer: &FutureProducer,
    kafka_producer_thread_set: &mut JoinSet<()>,
    trace_topic: &str,
    simulation: &Simulation,
    cache: &mut Cache,
    timestamp: GpsTime,
    metadata: &FrameMetadata,
    digitizer_id: DigitizerId,
    channels: &[Channel],
    source: &Source,
) -> Result<()> {
    let mut fbb = FlatBufferBuilder::new();

    build_trace_message(
        &mut fbb,
        simulation,
        cache,
        timestamp,
        metadata,
        digitizer_id,
        channels,
        source,
    )
    .unwrap();

    let send_args = SendMessageArgs::new(use_otel, fbb, producer, trace_topic, "Simulated Trace");
    kafka_producer_thread_set.spawn(send_message(send_args));
    Ok(())
}

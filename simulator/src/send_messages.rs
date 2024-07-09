use chrono::{DateTime, Utc};
use rdkafka::{
    producer::{FutureProducer, FutureRecord},
    util::Timeout,
};
use std::time::Duration;
use supermusr_common::{tracer::FutureRecordTracerExt, Channel, DigitizerId};
use supermusr_streaming_types::{
    ecs_6s4t_run_stop_generated::{finish_run_stop_buffer, RunStop, RunStopArgs},
    ecs_pl72_run_start_generated::{finish_run_start_buffer, RunStart, RunStartArgs},
    flatbuffers::FlatBufferBuilder,
    FrameMetadata,
};
use tracing::{debug, debug_span, error, Span};

use crate::integrated::{
    build_messages::{
        build_aggregated_event_list_message, build_digitiser_event_list_message,
        build_trace_message,
    },
    engine::SimulationEngineImmutableProperties,
    schedule::{SelectionModeOptions, SourceOptions},
};
use crate::integrated::{
    engine::SimulationEngineCache,
    run_messages::{SendAlarm, SendRunLogData, SendRunStart, SendRunStop, SendSampleEnvLog},
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

#[tracing::instrument(skip_all, target = "otel")]
pub(crate) fn send_run_start_command(
    immutable: &mut SimulationEngineImmutableProperties,
    status: &SendRunStart,
    topic: &str,
    timestamp: &DateTime<Utc>,
) -> Result<()> {
    let mut fbb = FlatBufferBuilder::new();
    let run_start = RunStartArgs {
        start_time: timestamp
            .signed_duration_since(DateTime::UNIX_EPOCH)
            .num_milliseconds()
            .try_into()?,
        run_name: Some(fbb.create_string(&status.name)),
        instrument_name: Some(fbb.create_string(&status.instrument)),
        ..Default::default()
    };
    let message = RunStart::create(&mut fbb, &run_start);
    finish_run_start_buffer(&mut fbb, message);

    let send_args = SendMessageArgs::new(
        immutable.use_otel,
        fbb,
        immutable.producer,
        topic,
        "Simulated Run Start",
    );
    immutable
        .kafka_producer_thread_set
        .spawn(send_message(send_args));
    Ok(())
}

#[tracing::instrument(skip_all, target = "otel")]
pub(crate) fn send_run_stop_command(
    immutable: &mut SimulationEngineImmutableProperties,
    status: &SendRunStop,
    topic: &str,
    timestamp: &DateTime<Utc>,
) -> Result<()> {
    let mut fbb = FlatBufferBuilder::new();
    let run_stop = RunStopArgs {
        stop_time: timestamp
            .signed_duration_since(DateTime::UNIX_EPOCH)
            .num_milliseconds()
            .try_into()?,
        run_name: Some(fbb.create_string(&status.name)),
        ..Default::default()
    };
    let message = RunStop::create(&mut fbb, &run_stop);
    finish_run_stop_buffer(&mut fbb, message);

    let send_args = SendMessageArgs::new(
        immutable.use_otel,
        fbb,
        immutable.producer,
        topic,
        "Simulated Run Stop",
    );
    immutable
        .kafka_producer_thread_set
        .spawn(send_message(send_args));
    Ok(())
}

#[tracing::instrument(skip_all, target = "otel", fields(digitizer_id = digitizer_id))]
pub(crate) fn send_trace_message(
    immutable: &mut SimulationEngineImmutableProperties,
    trace_topic: &str,
    sample_rate: u64,
    cache: &mut SimulationEngineCache,
    metadata: &FrameMetadata,
    digitizer_id: DigitizerId,
    channels: &[Channel],
    selection_mode: SelectionModeOptions,
) -> Result<()> {
    let mut fbb = FlatBufferBuilder::new();

    build_trace_message(
        &mut fbb,
        sample_rate,
        cache,
        metadata,
        digitizer_id,
        channels,
        selection_mode,
    )
    .unwrap();

    let send_args = SendMessageArgs::new(
        immutable.use_otel,
        fbb,
        immutable.producer,
        trace_topic,
        "Simulated Trace",
    );
    immutable
        .kafka_producer_thread_set
        .spawn(send_message(send_args));
    Ok(())
}

#[tracing::instrument(skip_all, target = "otel", fields(digitizer_id = digitizer_id))]
pub(crate) fn send_digitiser_event_list_message(
    immutable: &mut SimulationEngineImmutableProperties,
    event_topic: &str,
    cache: &mut SimulationEngineCache,
    metadata: &FrameMetadata,
    digitizer_id: DigitizerId,
    channels: &[Channel],
    source_options: &SourceOptions,
) -> Result<()> {
    let mut fbb = FlatBufferBuilder::new();

    build_digitiser_event_list_message(
        &mut fbb,
        cache,
        metadata,
        digitizer_id,
        channels,
        source_options,
    )
    .unwrap();

    let send_args = SendMessageArgs::new(
        immutable.use_otel,
        fbb,
        immutable.producer,
        event_topic,
        "Simulated Digitiser Event List",
    );
    immutable
        .kafka_producer_thread_set
        .spawn(send_message(send_args));
    Ok(())
}

#[tracing::instrument(skip_all, target = "otel")]
pub(crate) fn send_aggregated_frame_event_list_message(
    immutable: &mut SimulationEngineImmutableProperties,
    frame_event_topic: &str,
    cache: &mut SimulationEngineCache,
    metadata: &FrameMetadata,
    channels: &[Channel],
    source_options: &SourceOptions,
) -> Result<()> {
    let mut fbb = FlatBufferBuilder::new();

    build_aggregated_event_list_message(
        &mut fbb,
        cache,
        metadata,
        channels,
        source_options,
    )
    .unwrap();

    let send_args = SendMessageArgs::new(
        immutable.use_otel,
        fbb,
        immutable.producer,
        frame_event_topic,
        "Simulated Digitiser Event List",
    );
    immutable
        .kafka_producer_thread_set
        .spawn(send_message(send_args));
    Ok(())
}

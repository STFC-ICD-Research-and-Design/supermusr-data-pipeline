use super::build_messages::{
    build_aggregated_event_list_message, build_digitiser_event_list_message, build_trace_message,
};
use crate::{
    integrated::{
        simulation_elements::{
            run_messages::{
                SendAlarm, SendRunLogData, SendRunStart, SendRunStop, SendSampleEnvLog,
            },
            EventList, Trace,
        },
        simulation_engine::actions::{SelectionModeOptions, SourceOptions},
        simulation_engine::SimulationEngineExternals,
    },
    runs::{runlog, sample_environment},
};
use chrono::{DateTime, Utc};
use rdkafka::{
    producer::{FutureProducer, FutureRecord},
    util::Timeout,
    Message,
};
use std::{collections::VecDeque, time::Duration};
use supermusr_common::{tracer::FutureRecordTracerExt, Channel, DigitizerId};
use supermusr_streaming_types::{
    ecs_6s4t_run_stop_generated::{finish_run_stop_buffer, RunStop, RunStopArgs},
    ecs_al00_alarm_generated::{finish_alarm_buffer, Alarm, AlarmArgs},
    ecs_f144_logdata_generated::{f144_LogData, f144_LogDataArgs, finish_f_144_log_data_buffer},
    ecs_pl72_run_start_generated::{finish_run_start_buffer, RunStart, RunStartArgs},
    ecs_se00_data_generated::{
        finish_se_00_sample_environment_data_buffer, se00_SampleEnvironmentData,
        se00_SampleEnvironmentDataArgs,
    },
    flatbuffers::FlatBufferBuilder,
    FrameMetadata,
};
use tracing::{debug, debug_span, error, Span};

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
        Err(e) => error!(
            "Delivery failed: {:?}. Message Size: {}",
            e.0,
            e.1.payload().unwrap_or(&[]).len()
        ),
    };
}

fn get_time_since_epoch_ms(
    timestamp: &DateTime<Utc>,
) -> anyhow::Result<u64, <i64 as TryInto<u64>>::Error> {
    timestamp.timestamp_millis().try_into()
}

fn get_time_since_epoch_ns(timestamp: &DateTime<Utc>) -> anyhow::Result<i64> {
    timestamp
        .timestamp_nanos_opt()
        .ok_or(anyhow::anyhow!("Invalid Run Log Timestamp {timestamp}"))
}

#[tracing::instrument(skip_all, target = "otel")]
pub(crate) fn send_run_start_command(
    externals: &mut SimulationEngineExternals,
    status: &SendRunStart,
    timestamp: &DateTime<Utc>,
) -> anyhow::Result<()> {
    let mut fbb = FlatBufferBuilder::new();
    let run_start = RunStartArgs {
        start_time: get_time_since_epoch_ms(timestamp)?,
        run_name: Some(fbb.create_string(&status.name)),
        instrument_name: Some(fbb.create_string(&status.instrument)),
        ..Default::default()
    };
    let message = RunStart::create(&mut fbb, &run_start);
    finish_run_start_buffer(&mut fbb, message);

    let send_args = SendMessageArgs::new(
        externals.use_otel,
        fbb,
        externals.producer,
        externals.topics.run_controls,
        "Simulated Run Start",
    );
    externals
        .kafka_producer_thread_set
        .spawn(send_message(send_args));
    Ok(())
}

#[tracing::instrument(skip_all, target = "otel")]
pub(crate) fn send_run_stop_command(
    externals: &mut SimulationEngineExternals,
    status: &SendRunStop,
    timestamp: &DateTime<Utc>,
) -> anyhow::Result<()> {
    let mut fbb = FlatBufferBuilder::new();
    let run_stop = RunStopArgs {
        stop_time: get_time_since_epoch_ms(timestamp)?,
        run_name: Some(fbb.create_string(&status.name)),
        ..Default::default()
    };
    let message = RunStop::create(&mut fbb, &run_stop);
    finish_run_stop_buffer(&mut fbb, message);

    let send_args = SendMessageArgs::new(
        externals.use_otel,
        fbb,
        externals.producer,
        externals.topics.run_controls,
        "Simulated Run Stop",
    );
    externals
        .kafka_producer_thread_set
        .spawn(send_message(send_args));
    Ok(())
}

#[tracing::instrument(skip_all, target = "otel")]
pub(crate) fn send_run_log_command(
    externals: &mut SimulationEngineExternals,
    timestamp: &DateTime<Utc>,
    status: &SendRunLogData,
) -> anyhow::Result<()> {
    let value_type = status.value_type.clone().into();

    let mut fbb = FlatBufferBuilder::new();
    let run_log_args = f144_LogDataArgs {
        source_name: Some(fbb.create_string(&status.source_name)),
        timestamp: get_time_since_epoch_ns(timestamp)?,
        value_type,
        value: Some(runlog::make_value(&mut fbb, value_type, &status.value)?),
    };
    let message = f144_LogData::create(&mut fbb, &run_log_args);
    finish_f_144_log_data_buffer(&mut fbb, message);

    let send_args = SendMessageArgs::new(
        externals.use_otel,
        fbb,
        externals.producer,
        externals.topics.runlog,
        "Simulated Run Log Data",
    );
    externals
        .kafka_producer_thread_set
        .spawn(send_message(send_args));
    Ok(())
}

#[tracing::instrument(skip_all, target = "otel")]
pub(crate) fn send_se_log_command(
    externals: &mut SimulationEngineExternals,
    timestamp: &DateTime<Utc>,
    sample_env: &SendSampleEnvLog,
) -> anyhow::Result<()> {
    let mut fbb = FlatBufferBuilder::new();

    let timestamp_location = sample_env.location.clone().into();
    let values_type = sample_env.values_type.clone().into();
    let packet_timestamp = get_time_since_epoch_ns(timestamp)?;

    let timestamps = sample_env
        .timestamps
        .as_ref()
        .and_then(|timestamp_data| {
            timestamp_data
                .iter()
                .map(|ts| ts.timestamp_nanos_opt())
                .collect::<Option<Vec<_>>>()
        })
        .map(|timestamps| fbb.create_vector(&timestamps));

    let values = Some(sample_environment::make_value(
        &mut fbb,
        values_type,
        &sample_env.values,
    ));

    let se_log_args = se00_SampleEnvironmentDataArgs {
        name: Some(fbb.create_string(&sample_env.name)),
        channel: sample_env.channel.unwrap_or(-1),
        time_delta: sample_env.time_delta.unwrap_or(0.0),
        timestamp_location,
        timestamps,
        message_counter: sample_env.message_counter.unwrap_or_default(),
        packet_timestamp,
        values_type,
        values,
    };
    let message = se00_SampleEnvironmentData::create(&mut fbb, &se_log_args);
    finish_se_00_sample_environment_data_buffer(&mut fbb, message);

    let send_args = SendMessageArgs::new(
        externals.use_otel,
        fbb,
        externals.producer,
        externals.topics.selog,
        "Simulated Sample Environment Log",
    );
    externals
        .kafka_producer_thread_set
        .spawn(send_message(send_args));
    Ok(())
}

#[tracing::instrument(skip_all, target = "otel")]
pub(crate) fn send_alarm_command(
    externals: &mut SimulationEngineExternals,
    timestamp: &DateTime<Utc>,
    alarm: &SendAlarm,
) -> anyhow::Result<()> {
    let mut fbb = FlatBufferBuilder::new();
    let severity = alarm.severity.clone().into();
    let alarm_args = AlarmArgs {
        source_name: Some(fbb.create_string(&alarm.source_name)),
        timestamp: get_time_since_epoch_ns(timestamp)?,
        severity,
        message: Some(fbb.create_string(&alarm.message)),
    };
    let message = Alarm::create(&mut fbb, &alarm_args);
    finish_alarm_buffer(&mut fbb, message);

    let send_args = SendMessageArgs::new(
        externals.use_otel,
        fbb,
        externals.producer,
        externals.topics.alarm,
        "Simulated Alarm",
    );
    externals
        .kafka_producer_thread_set
        .spawn(send_message(send_args));
    Ok(())
}

#[tracing::instrument(skip_all, target = "otel", fields(digitizer_id = digitizer_id))]
pub(crate) fn send_trace_message(
    externals: &mut SimulationEngineExternals,
    sample_rate: u64,
    cache: &mut VecDeque<Trace>,
    metadata: &FrameMetadata,
    digitizer_id: DigitizerId,
    channels: &[Channel],
    selection_mode: SelectionModeOptions,
) -> anyhow::Result<()> {
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
        externals.use_otel,
        fbb,
        externals.producer,
        externals.topics.traces,
        "Simulated Trace",
    );
    externals
        .kafka_producer_thread_set
        .spawn(send_message(send_args));
    Ok(())
}

#[tracing::instrument(skip_all, target = "otel", fields(digitizer_id = digitizer_id))]
pub(crate) fn send_digitiser_event_list_message(
    externals: &mut SimulationEngineExternals,
    cache: &mut VecDeque<EventList<'_>>,
    metadata: &FrameMetadata,
    digitizer_id: DigitizerId,
    channels: &[Channel],
    source_options: &SourceOptions,
) -> anyhow::Result<()> {
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
        externals.use_otel,
        fbb,
        externals.producer,
        externals.topics.events,
        "Simulated Digitiser Event List",
    );
    externals
        .kafka_producer_thread_set
        .spawn(send_message(send_args));
    Ok(())
}

#[tracing::instrument(skip_all, target = "otel")]
pub(crate) fn send_aggregated_frame_event_list_message(
    externals: &mut SimulationEngineExternals,
    cache: &mut VecDeque<EventList<'_>>,
    metadata: &FrameMetadata,
    channels: &[Channel],
    source_options: &SourceOptions,
) -> anyhow::Result<()> {
    let mut fbb = FlatBufferBuilder::new();

    build_aggregated_event_list_message(&mut fbb, cache, metadata, channels, source_options)
        .unwrap();

    let send_args = SendMessageArgs::new(
        externals.use_otel,
        fbb,
        externals.producer,
        externals.topics.frame_events,
        "Simulated Digitiser Event List",
    );
    externals
        .kafka_producer_thread_set
        .spawn(send_message(send_args));
    Ok(())
}

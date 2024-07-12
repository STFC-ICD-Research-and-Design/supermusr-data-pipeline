use anyhow::anyhow;
use chrono::{DateTime, Utc};
use rdkafka::{
    producer::{FutureProducer, FutureRecord},
    util::Timeout, Message,
};
use std::{collections::VecDeque, time::Duration};
use supermusr_common::{tracer::FutureRecordTracerExt, Channel, DigitizerId};
use supermusr_streaming_types::{
    ecs_6s4t_run_stop_generated::{finish_run_stop_buffer, RunStop, RunStopArgs},
    ecs_al00_alarm_generated::{finish_alarm_buffer, Alarm, AlarmArgs, Severity},
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

use crate::integrated::simulation_elements::run_messages::{
    SendAlarm, SendRunLogData, SendRunStart, SendRunStop, SendSampleEnvLog,
};
use crate::{
    integrated::{
        build_messages::{
            build_aggregated_event_list_message, build_digitiser_event_list_message,
            build_trace_message,
        },
        simulation_engine::actions::{SelectionModeOptions, SourceOptions},
        simulation_engine::SimulationEngineExternals,
    },
    runs::{runlog, sample_environment},
};
use anyhow::Result;

use super::simulation_elements::event_list::{EventList, Trace};

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
        Err(e) => error!("Delivery failed: {:?}. Message Size: {}", e.0, e.1.payload().unwrap_or(&[]).len()),
    };
}

fn get_time_since_epoch_ms(timestamp : &DateTime<Utc>) -> Result<u64, <i64 as TryInto<u64>>::Error> {
    timestamp.timestamp_millis()
        .try_into()
}

fn get_time_since_epoch_ns(timestamp : &DateTime<Utc>) -> Result<i64> {
    timestamp.timestamp_nanos_opt()
        .ok_or(anyhow!("Invalid Run Log Timestamp {timestamp}"))
}

#[tracing::instrument(skip_all, target = "otel")]
pub(crate) fn send_run_start_command(
    externals: &mut SimulationEngineExternals,
    status: &SendRunStart,
    topic: &str,
    timestamp: &DateTime<Utc>,
) -> Result<()> {
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
        topic,
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
    topic: &str,
    timestamp: &DateTime<Utc>,
) -> Result<()> {
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
        topic,
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
    topic: &str,
) -> Result<()> {
    let value_type = runlog::value_type(&status.value_type)?;

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
        topic,
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
    topic: &str,
) -> Result<()> {
    let mut fbb = FlatBufferBuilder::new();

    let timestamp_location = sample_environment::location(&sample_env.location)?;
    let values_type = sample_environment::values_union_type(&sample_env.values_type)?;
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
        topic,
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
    topic: &str,
) -> Result<()> {
    let mut fbb = FlatBufferBuilder::new();
    let severity = match alarm.severity.as_str() {
        "OK" => Severity::OK,
        "MINOR" => Severity::MINOR,
        "MAJOR" => Severity::MAJOR,
        "INVALID" => Severity::INVALID,
        _ => return Err(anyhow!("Unable to read severity")),
    };
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
        topic,
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
    topic: &str,
    sample_rate: u64,
    cache: &mut VecDeque<Trace>,
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
        externals.use_otel,
        fbb,
        externals.producer,
        topic,
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
    topic: &str,
    cache: &mut VecDeque<EventList<'_>>,
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
        externals.use_otel,
        fbb,
        externals.producer,
        topic,
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
    topic: &str,
    cache: &mut VecDeque<EventList<'_>>,
    metadata: &FrameMetadata,
    channels: &[Channel],
    source_options: &SourceOptions,
) -> Result<()> {
    let mut fbb = FlatBufferBuilder::new();

    build_aggregated_event_list_message(&mut fbb, cache, metadata, channels, source_options)
        .unwrap();

    let send_args = SendMessageArgs::new(
        externals.use_otel,
        fbb,
        externals.producer,
        topic,
        "Simulated Digitiser Event List",
    );
    externals
        .kafka_producer_thread_set
        .spawn(send_message(send_args));
    Ok(())
}

mod runlog;
mod sample_environment;

use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use clap::{Parser, Subcommand};
use rdkafka::{
    producer::{FutureProducer, FutureRecord},
    util::Timeout,
};
use std::time::Duration;
use supermusr_common::tracer::FutureRecordTracerExt;
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
};
use tracing::{debug, error};

#[derive(Clone, Parser)]
pub(crate) struct Start {
    /// Topic to publish command to
    #[clap(long)]
    topic: String,

    /// Timestamp of the command, defaults to now, if not given.
    #[clap(long)]
    time: Option<DateTime<Utc>>,

    /// Unique name of the run
    #[clap(long)]
    run_name: String,

    /// Name of the instrument being run
    #[clap(long)]
    instrument_name: String,
}
#[derive(Clone, Parser)]
pub(crate) struct Stop {
    /// Topic to publish command to
    #[clap(long)]
    topic: String,

    /// Timestamp of the command, defaults to now, if not given.
    #[clap(long)]
    time: Option<DateTime<Utc>>,

    /// Unique name of the run
    #[clap(long)]
    run_name: String,
}

#[derive(Clone, Debug, Parser)]
pub(crate) struct RunLogData {
    /// Topic to publish command to
    #[clap(long)]
    topic: String,

    /// Timestamp of the command, defaults to now, if not given.
    #[clap(long)]
    time: Option<DateTime<Utc>>,

    /// Name of the source being logged
    #[clap(long)]
    source_name: String,

    /// Type of the logdata
    #[clap(long)]
    value_type: String,

    /// Value of the logdata
    #[clap()]
    value: Vec<String>,
}

#[derive(Clone, Debug, Parser)]
pub(crate) struct SampleEnvData {
    /// Topic to publish command to
    #[clap(long)]
    topic: String,

    /// Timestamp of the command, defaults to now, if not given.
    #[clap(long)]
    time: Option<DateTime<Utc>>,

    /// Name of the source being logged
    #[clap(long)]
    name: String,

    #[clap(long)]
    channel: Option<i32>,

    /// Optional: time between each sample in ns
    #[clap(long)]
    time_delta: Option<f64>,

    /// Type of the sample value
    #[clap(long, default_value = "int64")]
    values_type: String,

    /// Incrementing counter
    #[clap(long)]
    message_counter: Option<i64>,

    #[clap(long)]
    location: String,

    /// Value of samples
    #[clap()]
    values: Vec<String>,

    #[command(subcommand)]
    timestamps: Option<SampleEnvTimestamp>,
}

#[derive(Clone, Debug, Subcommand)]
enum SampleEnvTimestamp {
    Timestamps(SampleEnvTimestampData),
}
#[derive(Clone, Debug, Parser)]
pub(crate) struct SampleEnvTimestampData {
    #[clap()]
    timestamps: Vec<DateTime<Utc>>,
}

#[derive(Clone, Debug, Parser)]
pub(crate) struct AlarmData {
    /// Topic to publish command to
    #[clap(long)]
    topic: String,

    /// Timestamp of the command, defaults to now, if not given.
    #[clap(long)]
    time: Option<DateTime<Utc>>,

    /// Source Name
    #[clap(long)]
    source_name: String,

    #[clap(long)]
    severity: String,

    #[clap(long)]
    message: String,
}

#[tracing::instrument(skip_all)]
pub(crate) async fn create_run_start_command(
    use_otel: bool,
    status: &Start,
    producer: &FutureProducer,
) -> Result<()> {
    let mut fbb = FlatBufferBuilder::new();
    let run_start = RunStartArgs {
        start_time: status
            .time
            .unwrap_or(Utc::now())
            .signed_duration_since(DateTime::UNIX_EPOCH)
            .num_milliseconds()
            .try_into()?,
        run_name: Some(fbb.create_string(&status.run_name)),
        instrument_name: Some(fbb.create_string(&status.instrument_name)),
        ..Default::default()
    };
    let message = RunStart::create(&mut fbb, &run_start);
    finish_run_start_buffer(&mut fbb, message);

    let future_record = FutureRecord::to(&status.topic)
        .payload(fbb.finished_data())
        .conditional_inject_current_span_into_headers(use_otel)
        .key("Simulated Event");

    let timeout = Timeout::After(Duration::from_millis(100));
    match producer.send(future_record, timeout).await {
        Ok(r) => debug!("Delivery: {:?}", r),
        Err(e) => error!("Delivery failed: {:?}", e),
    };
    Ok(())
}

#[tracing::instrument(skip_all)]
pub(crate) async fn create_run_stop_command(
    use_otel: bool,
    stop: &Stop,
    producer: &FutureProducer,
) -> Result<()> {
    let mut fbb = FlatBufferBuilder::new();
    let run_stop = RunStopArgs {
        stop_time: stop
            .time
            .unwrap_or(Utc::now())
            .signed_duration_since(DateTime::UNIX_EPOCH)
            .num_milliseconds()
            .try_into()?,
        run_name: Some(fbb.create_string(&stop.run_name)),
        ..Default::default()
    };
    let message = RunStop::create(&mut fbb, &run_stop);
    finish_run_stop_buffer(&mut fbb, message);

    let future_record = FutureRecord::to(&stop.topic)
        .payload(fbb.finished_data())
        .conditional_inject_current_span_into_headers(use_otel)
        .key("Simulated Event");

    let timeout = Timeout::After(Duration::from_millis(100));
    match producer.send(future_record, timeout).await {
        Ok(r) => debug!("Delivery: {:?}", r),
        Err(e) => error!("Delivery failed: {:?}", e),
    };
    Ok(())
}

#[tracing::instrument(skip_all)]
pub(crate) async fn create_runlog_command(
    use_otel: bool,
    runlog: &RunLogData,
    producer: &FutureProducer,
) -> Result<()> {
    let value_type = runlog::value_type(&runlog.value_type)?;

    let timestamp = runlog.time.unwrap_or(Utc::now());
    let mut fbb = FlatBufferBuilder::new();
    let run_log_args = f144_LogDataArgs {
        source_name: Some(fbb.create_string(&runlog.source_name)),
        timestamp: timestamp
            .signed_duration_since(DateTime::UNIX_EPOCH)
            .num_nanoseconds()
            .ok_or(anyhow!("Invalid Run Log Timestamp {timestamp}"))?,
        value_type,
        value: Some(runlog::make_value(&mut fbb, value_type, &runlog.value)?),
    };
    let message = f144_LogData::create(&mut fbb, &run_log_args);
    finish_f_144_log_data_buffer(&mut fbb, message);

    let future_record = FutureRecord::to(&runlog.topic)
        .payload(fbb.finished_data())
        .conditional_inject_current_span_into_headers(use_otel)
        .key("Simulated Event");

    let timeout = Timeout::After(Duration::from_millis(100));
    match producer.send(future_record, timeout).await {
        Ok(r) => debug!("Delivery: {:?}", r),
        Err(e) => error!("Delivery failed: {:?}", e),
    };
    Ok(())
}

#[tracing::instrument(skip_all)]
pub(crate) async fn create_sample_environment_command(
    use_otel: bool,
    sample_env: &SampleEnvData,
    producer: &FutureProducer,
) -> Result<()> {
    let mut fbb = FlatBufferBuilder::new();
    let timestamp_location = sample_environment::location(&sample_env.location)?;
    let values_type = sample_environment::values_union_type(&sample_env.values_type)?;
    let packet_timestamp = sample_env.time.unwrap_or(Utc::now());
    let packet_timestamp = packet_timestamp
        .signed_duration_since(DateTime::UNIX_EPOCH)
        .num_nanoseconds()
        .ok_or(anyhow!(
            "Invalid Sample Environment Log Timestamp {packet_timestamp}"
        ))?;

    let timestamps = sample_env
        .timestamps
        .as_ref()
        .and_then(|SampleEnvTimestamp::Timestamps(timestamp_data)| {
            timestamp_data
                .timestamps
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

    let future_record = FutureRecord::to(&sample_env.topic)
        .payload(fbb.finished_data())
        .conditional_inject_current_span_into_headers(use_otel)
        .key("Simulated Event");

    let timeout = Timeout::After(Duration::from_millis(100));
    match producer.send(future_record, timeout).await {
        Ok(r) => debug!("Delivery: {:?}", r),
        Err(e) => error!("Delivery failed: {:?}", e),
    };
    Ok(())
}

#[tracing::instrument(skip_all)]
pub(crate) async fn create_alarm_command(
    use_otel: bool,
    alarm: &AlarmData,
    producer: &FutureProducer,
) -> Result<()> {
    let mut fbb = FlatBufferBuilder::new();
    let severity = match alarm.severity.as_str() {
        "OK" => Severity::OK,
        "MINOR" => Severity::MINOR,
        "MAJOR" => Severity::MAJOR,
        "INVALID" => Severity::INVALID,
        _ => return Err(anyhow!("Unable to read severity")),
    };
    let timestamp = alarm.time.unwrap_or(Utc::now());
    let alarm_args = AlarmArgs {
        source_name: Some(fbb.create_string(&alarm.source_name)),
        timestamp: timestamp.timestamp_nanos_opt().ok_or(anyhow!("No nanos"))?,
        severity,
        message: Some(fbb.create_string(&alarm.message)),
    };
    let message = Alarm::create(&mut fbb, &alarm_args);
    finish_alarm_buffer(&mut fbb, message);

    let future_record = FutureRecord::to(&alarm.topic)
        .payload(fbb.finished_data())
        .conditional_inject_current_span_into_headers(use_otel)
        .key("Simulated Event");

    let timeout = Timeout::After(Duration::from_millis(100));
    match producer.send(future_record, timeout).await {
        Ok(r) => debug!("Delivery: {:?}", r),
        Err(e) => error!("Delivery failed: {:?}", e),
    };
    Ok(())
}
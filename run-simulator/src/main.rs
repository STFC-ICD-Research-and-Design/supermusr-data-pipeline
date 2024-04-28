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
use supermusr_common::{
    conditional_init_tracer,
    tracer::{FutureRecordTracerExt, OtelTracer},
};
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
use tracing::{debug, error, info, level_filters::LevelFilter, trace_span};

#[derive(Clone, Parser)]
#[clap(author, version, about)]
struct Cli {
    /// Kafka broker address
    #[clap(long = "broker")]
    broker_address: String,

    /// Kafka username
    #[clap(long)]
    username: Option<String>,

    /// Kafka password
    #[clap(long)]
    password: Option<String>,

    /// Topic to publish command to
    #[clap(long)]
    topic: String,

    /// Unique name of the run
    #[clap(long)]
    run_name: String,

    /// If set, then open-telemetry data is sent to the URL specified, otherwise the standard tracing subscriber is used
    #[clap(long)]
    otel_endpoint: Option<String>,

    /// Timestamp of the command, defaults to now, if not given.
    #[clap(long)]
    time: Option<DateTime<Utc>>,

    #[command(subcommand)]
    mode: Mode,
}

#[derive(Clone, Subcommand)]
enum Mode {
    /// Send a single RunStart command
    Start(Status),

    /// Send a single RunStop command
    Stop,

    /// Send a single RunStop command
    Log(RunLogData),

    /// Send a single SampleEnv command
    SampleEnv(SampleEnvData),

    /// Send a single Alarm command
    Alarm(AlarmData),
}

#[derive(Clone, Parser)]
struct Status {
    /// Name of the instrument being run
    #[clap(long)]
    instrument_name: String,
}

#[derive(Clone, Debug, Parser)]
struct RunLogData {
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
struct SampleEnvData {
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
struct SampleEnvTimestampData {
    #[clap()]
    timestamps: Vec<DateTime<Utc>>,
}

#[derive(Clone, Debug, Parser)]
struct AlarmData {
    /// Source Name
    #[clap(long)]
    source_name: String,

    #[clap(long)]
    severity: String,

    #[clap(long)]
    message: String,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let tracer = conditional_init_tracer!(cli.otel_endpoint.as_deref(), LevelFilter::TRACE);

    let span = match cli.mode {
        Mode::Start(_) => trace_span!("RunStart"),
        Mode::Stop => trace_span!("RunStop"),
        Mode::Log(_) => trace_span!("RunLog"),
        Mode::SampleEnv(_) => trace_span!("SampleEnvironmentLog"),
        Mode::Alarm(_) => trace_span!("AlarmLog"),
    };
    let _guard = span.enter();

    let client_config = supermusr_common::generate_kafka_client_config(
        &cli.broker_address,
        &cli.username,
        &cli.password,
    );
    let producer: FutureProducer = client_config.create().unwrap();

    let mut fbb = FlatBufferBuilder::new();
    let time = cli.time.unwrap_or(Utc::now());
    match cli.mode.clone() {
        Mode::Start(status) => {
            create_run_start_command(&mut fbb, time, &cli.run_name, &status.instrument_name)
                .expect("RunStart created")
        }
        Mode::Stop => {
            create_run_stop_command(&mut fbb, time, &cli.run_name).expect("RunStop created")
        }
        Mode::Log(run_log) => {
            create_runlog_command(&mut fbb, time, &run_log).expect("RunLog created")
        }
        Mode::SampleEnv(sample_env) => {
            info!("Creating run log");
            create_sample_environment_command(&mut fbb, time, &sample_env).expect("SELog created")
        }
        Mode::Alarm(alarm) => {
            info!("Creating alarm log");
            create_alarm_command(&mut fbb, time, &alarm).expect("AlarmLog created")
        }
    }

    // Prepare the kafka message
    let future_record = FutureRecord::to(&cli.topic)
        .payload(fbb.finished_data())
        .conditional_inject_span_into_headers(tracer.is_some(), &span)
        .key("Run Command");

    let timeout = Timeout::After(Duration::from_millis(100));
    match producer.send(future_record, timeout).await {
        Ok(r) => debug!("Delivery: {:?}", r),
        Err(e) => error!("Delivery failed: {:?}", e),
    };

    info!("Run command send");
}

#[tracing::instrument]
pub(crate) fn create_run_start_command(
    fbb: &mut FlatBufferBuilder<'_>,
    start_time: DateTime<Utc>,
    run_name: &str,
    instrument_name: &str,
) -> Result<()> {
    let run_start = RunStartArgs {
        start_time: start_time
            .signed_duration_since(DateTime::UNIX_EPOCH)
            .num_milliseconds() as u64,
        run_name: Some(fbb.create_string(run_name)),
        instrument_name: Some(fbb.create_string(instrument_name)),
        ..Default::default()
    };
    let message = RunStart::create(fbb, &run_start);
    finish_run_start_buffer(fbb, message);
    Ok(())
}

#[tracing::instrument]
pub(crate) fn create_run_stop_command(
    fbb: &mut FlatBufferBuilder<'_>,
    stop_time: DateTime<Utc>,
    run_name: &str,
) -> Result<()> {
    let run_stop = RunStopArgs {
        stop_time: stop_time
            .signed_duration_since(DateTime::UNIX_EPOCH)
            .num_milliseconds() as u64,
        run_name: Some(fbb.create_string(run_name)),
        ..Default::default()
    };
    let message = RunStop::create(fbb, &run_stop);
    finish_run_stop_buffer(fbb, message);
    Ok(())
}

#[tracing::instrument(skip(fbb))]
pub(crate) fn create_runlog_command(
    fbb: &mut FlatBufferBuilder<'_>,
    timestamp: DateTime<Utc>,
    run_log: &RunLogData,
) -> Result<()> {
    let value_type = runlog::value_type(&run_log.value_type)?;

    let run_log = f144_LogDataArgs {
        source_name: Some(fbb.create_string(&run_log.source_name)),
        timestamp: timestamp
            .signed_duration_since(DateTime::UNIX_EPOCH)
            .num_nanoseconds()
            .ok_or(anyhow!("Invalid Run Log Timestamp {timestamp}"))?,
        value_type,
        value: Some(runlog::make_value(fbb, value_type, &run_log.value)?),
    };
    let message = f144_LogData::create(fbb, &run_log);
    finish_f_144_log_data_buffer(fbb, message);
    Ok(())
}

#[tracing::instrument(skip(fbb))]
pub(crate) fn create_sample_environment_command(
    fbb: &mut FlatBufferBuilder<'_>,
    packet_timestamp: DateTime<Utc>,
    sample_env: &SampleEnvData,
) -> Result<()> {
    let timestamp_location = sample_environment::location(&sample_env.location)?;
    let values_type = sample_environment::values_union_type(&sample_env.values_type)?;
    let packet_timestamp = packet_timestamp
        .signed_duration_since(DateTime::UNIX_EPOCH)
        .num_nanoseconds()
        .ok_or(anyhow!(
            "Invalid Sample Environment Log Timestamp {packet_timestamp}"
        ))?;

    let timestamps = sample_env
        .timestamps
        .as_ref()
        .map(|SampleEnvTimestamp::Timestamps(timestamp_data)| {
            timestamp_data
                .timestamps
                .iter()
                .map(|ts| ts.timestamp_nanos_opt())
                .collect::<Option<Vec<_>>>()
        })
        .flatten()
        .map(|timestamps| fbb.create_vector(&timestamps));

    let values = Some(sample_environment::make_value(
        fbb,
        values_type,
        &sample_env.values,
    ));

    let se_log = se00_SampleEnvironmentDataArgs {
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
    let message = se00_SampleEnvironmentData::create(fbb, &se_log);
    finish_se_00_sample_environment_data_buffer(fbb, message);
    Ok(())
}

#[tracing::instrument(skip(fbb))]
pub(crate) fn create_alarm_command(
    fbb: &mut FlatBufferBuilder<'_>,
    timestamp: DateTime<Utc>,
    alarm: &AlarmData,
) -> Result<()> {
    let severity = match alarm.severity.as_str() {
        "OK" => Severity::OK,
        "MINOR" => Severity::MINOR,
        "MAJOR" => Severity::MAJOR,
        "INVALID" => Severity::INVALID,
        _ => return Err(anyhow!("Unable to read severity")),
    };
    let alarm_log = AlarmArgs {
        source_name: Some(fbb.create_string(&alarm.source_name)),
        timestamp: timestamp.timestamp_nanos_opt().ok_or(anyhow!("No nanos"))?,
        severity,
        message: Some(fbb.create_string(&alarm.message)),
    };
    let message = Alarm::create(fbb, &alarm_log);
    finish_alarm_buffer(fbb, message);
    Ok(())
}

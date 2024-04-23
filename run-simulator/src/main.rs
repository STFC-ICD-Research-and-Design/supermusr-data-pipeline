mod runlog;
mod sample_environment;

use chrono::{DateTime, Utc};
use clap::{Parser, Subcommand};
use rdkafka::{
    message::OwnedHeaders,
    producer::{FutureProducer, FutureRecord},
    util::Timeout,
};
use anyhow::{anyhow, Result};
use std::time::Duration;
use supermusr_common::tracer::OtelTracer;
use supermusr_streaming_types::{
    ecs_6s4t_run_stop_generated::{finish_run_stop_buffer, RunStop, RunStopArgs},
    ecs_f144_logdata_generated::{f144_LogData, f144_LogDataArgs, finish_f_144_log_data_buffer},
    ecs_pl72_run_start_generated::{finish_run_start_buffer, RunStart, RunStartArgs},
    ecs_se00_data_generated::{finish_se_00_sample_environment_data_buffer, se00_SampleEnvironmentData, se00_SampleEnvironmentDataArgs},
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

    /// Value of
    #[clap()]
    values: Vec<String>,
    
    #[clap(long)]
    timestamp: DateTime<Utc>,
    
    #[clap(long)]
    channel: i32,
    
    #[clap(long)]
    time_delta: f64,

    /// Type of the logdata
    #[clap(long, default_value = "int64")]
    values_type: String,
    
    #[clap(long)]
    message_counter: i64,
    
    #[clap(long)]
    location: String,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let _tracer = init_tracer(cli.otel_endpoint.as_deref());

    let span = match cli.mode {
        Mode::Start(_) => trace_span!("RunStart"),
        Mode::Stop => trace_span!("RunStop"),
        Mode::Log(_) => trace_span!("RunLog"),
        Mode::SampleEnv(_) => trace_span!("SampleEnvironmentLog"),
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
    }

    // Send bytes to the broker
    let future_record = {
        if cli.otel_endpoint.is_some() {
            let mut headers = OwnedHeaders::new();
            OtelTracer::inject_context_from_span_into_kafka(&span, &mut headers);

            FutureRecord::to(&cli.topic)
                .payload(fbb.finished_data())
                .headers(headers)
                .key("RunCommand")
        } else {
            FutureRecord::to(&cli.topic)
                .payload(fbb.finished_data())
                .key("RunCommand")
        }
    };

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
    run_log: &RunLogData
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
    let location = sample_environment::location(&sample_env.location)?;
    let values_type = sample_environment::values_union_type(&sample_env.values_type)?;
    let packet_timestamp = packet_timestamp
        .signed_duration_since(DateTime::UNIX_EPOCH)
        .num_nanoseconds()
        .ok_or(anyhow!("Invalid Sample Environment Log Timestamp {packet_timestamp}"))?;

    let se_log = se00_SampleEnvironmentDataArgs {
        name: Some(fbb.create_string(&sample_env.name)),
        channel: sample_env.channel,
        time_delta: sample_env.time_delta,
        timestamp_location: location,
        timestamps: None,
        message_counter: sample_env.message_counter,
        packet_timestamp,
        values_type,
        values: Some(sample_environment::make_value(fbb, values_type, &sample_env.values)),
    };
    let message = se00_SampleEnvironmentData::create(fbb, &se_log);
    finish_se_00_sample_environment_data_buffer(fbb, message);
    Ok(())
}

fn init_tracer(otel_endpoint: Option<&str>) -> Option<OtelTracer> {
    otel_endpoint
        .map(|otel_endpoint| {
            OtelTracer::new(
                otel_endpoint,
                "Run Simulator",
                Some(("run_simulator", LevelFilter::TRACE)),
            )
            .expect("Open Telemetry Tracer is created")
        })
        .or_else(|| {
            tracing_subscriber::fmt::init();
            None
        })
}

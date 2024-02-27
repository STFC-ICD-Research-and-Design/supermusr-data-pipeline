use anyhow::Result;
use chrono::{DateTime, Utc};
use clap::{Parser, Subcommand};
use rdkafka::{
    producer::{FutureProducer, FutureRecord},
    util::Timeout,
};
use std::time::Duration;
use supermusr_streaming_types::{
    ecs_6s4t_run_stop_generated::{finish_run_stop_buffer, RunStop, RunStopArgs},
    ecs_pl72_run_start_generated::{finish_run_start_buffer, RunStart, RunStartArgs},
    flatbuffers::FlatBufferBuilder,
};
use tracing::{debug, error, info};

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

    /// Timestamp of the command, defaults to now, if not given.
    #[clap(long)]
    time: Option<DateTime<Utc>>,

    #[command(subcommand)]
    mode: Mode,
}

#[derive(Clone, Subcommand)]
enum Mode {
    /// Send a single RunStart command
    RunStart(Status),

    /// Send a single RunStop command
    RunStop,
}

#[derive(Clone, Parser)]
struct Status {
    /// Name of the instrument being run
    #[clap(long)]
    instrument_name: String,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    let client_config = supermusr_common::generate_kafka_client_config(
        &cli.broker_address,
        &cli.username,
        &cli.password,
    );
    let producer: FutureProducer = client_config.create().unwrap();

    let mut fbb = FlatBufferBuilder::new();
    let time = cli.time.unwrap_or(Utc::now());
    let bytes = match cli.mode.clone() {
        Mode::RunStart(status) => {
            create_run_start_command(&mut fbb, time, &cli.run_name, &status.instrument_name)
                .expect("RunStart created")
        }
        Mode::RunStop => {
            create_run_stop_command(&mut fbb, time, &cli.run_name).expect("RunStop created")
        }
    };

    // Send bytes to the broker
    match producer
        .send(
            FutureRecord::to(&cli.topic)
                .payload(&bytes)
                .key(&"Run".to_string()),
            Timeout::After(Duration::from_millis(100)),
        )
        .await
    {
        Ok(r) => debug!("Delivery: {:?}", r),
        Err(e) => error!("Delivery failed: {:?}", e),
    };

    info!("Run command send");
}

pub(crate) fn create_run_start_command(
    fbb: &mut FlatBufferBuilder<'_>,
    start_time: DateTime<Utc>,
    run_name: &str,
    instrument_name: &str,
) -> Result<Vec<u8>> {
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
    Ok(fbb.finished_data().to_owned())
}

pub(crate) fn create_run_stop_command(
    fbb: &mut FlatBufferBuilder<'_>,
    stop_time: DateTime<Utc>,
    run_name: &str,
) -> Result<Vec<u8>> {
    let run_stop = RunStopArgs {
        stop_time: stop_time
            .signed_duration_since(DateTime::UNIX_EPOCH)
            .num_milliseconds() as u64,
        run_name: Some(fbb.create_string(run_name)),
        ..Default::default()
    };
    let message = RunStop::create(fbb, &run_stop);
    finish_run_stop_buffer(fbb, message);
    Ok(fbb.finished_data().to_owned())
}

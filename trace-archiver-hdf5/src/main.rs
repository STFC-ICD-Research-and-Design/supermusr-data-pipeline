mod continuous;
mod control;
mod file;

use clap::{Args, Parser, Subcommand};
use metrics_exporter_prometheus::PrometheusBuilder;
use std::{net::SocketAddr, path::PathBuf};
use supermusr_common::CommonKafkaOpts;

#[derive(Debug, Parser)]
#[clap(author, version, about)]
struct Cli {
    /// Endpoint on which OpenMetrics flavour metrics are available
    #[clap(long, env, default_value = "127.0.0.1:9090")]
    observability_address: SocketAddr,

    #[clap(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Run tool in control mode.
    /// This requires a control topic to provided which run commands are read from.
    /// Each HDF5 file from a run is saved under the run start timestamp.
    #[clap(name = "control")]
    Control(ControlOpts),

    /// Run tool in continuous mode.
    /// This requires a filename for trace data to be saved into.
    /// Unlike control mode, all trace data is saved continuously until termination.
    #[clap(name = "continuous")]
    Continuous(ContinuousOpts),
}

#[derive(Debug, Args)]
struct CommonOpts {
    #[clap(flatten)]
    common_kafka_options: CommonKafkaOpts,

    /// Kafka consumer group
    #[clap(long = "group")]
    consumer_group: String,

    /// The Kafka topic that trace messages are consumed from
    #[clap(long)]
    trace_topic: String,

    /// Number of digitisers.
    #[clap(long)]
    digitizer_count: usize,
}

#[derive(Debug, Args)]
struct ControlOpts {
    #[clap(long)]
    control_topic: String,

    #[clap(flatten)]
    common: CommonOpts,
}

#[derive(Debug, Args)]
struct ContinuousOpts {
    #[clap(long)]
    file: PathBuf,

    #[clap(flatten)]
    common: CommonOpts,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    tracing_subscriber::fmt::init();

    let builder = PrometheusBuilder::new();
    builder
        .with_http_listener(cli.observability_address)
        .install()
        .expect("Prometheus metrics exporter should be setup");

    match cli.command {
        Commands::Control(args) => control::run(args).await,
        Commands::Continuous(args) => continuous::run(args).await,
    }
}

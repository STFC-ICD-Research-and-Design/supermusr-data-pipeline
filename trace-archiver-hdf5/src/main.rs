mod continuous;
mod control;
mod file;

use anyhow::Result;
use clap::Subcommand;
use clap::{Args, Parser};
use std::{net::SocketAddr, path::PathBuf};

#[derive(Debug, Parser)]
#[clap(author, version, about)]
struct Cli {
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
    #[clap(long)]
    broker: String,

    #[clap(long)]
    username: Option<String>,

    #[clap(long)]
    password: Option<String>,

    #[clap(long = "group")]
    consumer_group: String,

    #[clap(long)]
    trace_topic: String,

    #[clap(long)]
    digitizer_count: usize,

    #[clap(long, env, default_value = "127.0.0.1:9090")]
    observability_address: SocketAddr,
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
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Control(args) => control::run(args).await,
        Commands::Continuous(args) => continuous::run(args).await,
    }
}

mod daq_trace;
mod message_debug;

use anyhow::Result;
use clap::{Args, Parser, Subcommand};

#[derive(Debug, Parser)]
#[clap(author, version, about)]
struct Cli {
    /// The subcommand to run
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Provides metrics regarding data transmission from the digitisers via Kafka.
    #[clap(name = "daq-trace")]
    DaqTrace(DaqTraceOpts),

    /// Run message dumping tool.
    #[clap(name = "message-debug")]
    MessageDebug(CommonOpts),
}

#[derive(Debug, Args)]
struct CommonOpts {
    /// Address of Kafka broker, should have format `host:port`, e.g. `localhost:9092`
    #[clap(long)]
    broker: String,

    /// Optional Kafka username. If provided, a corresponding password is required.
    #[clap(long)]
    username: Option<String>,

    /// Optional Kafka password. If provided, a corresponding username is requred.
    #[clap(long)]
    password: Option<String>,

    /// Kafka consumer group
    #[clap(long = "group")]
    consumer_group: String,

    /// The Kafka topic to consume messages from
    #[clap(long)]
    topic: String,
}

#[derive(Debug, Args)]
struct DaqTraceOpts {
    /// The interval at which the message rate is calculated in seconds.
    #[clap(long, default_value_t = 5)]
    message_rate_interval: u64,

    #[clap(flatten)]
    common: CommonOpts,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::DaqTrace(args) => daq_trace::run(args).await,
        Commands::MessageDebug(args) => message_debug::run(args).await,
    }
}

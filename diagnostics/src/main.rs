mod daq_trace;
mod message_debug;

use clap::{Args, Parser, Subcommand};
use supermusr_common::CommonKafkaOpts;

#[derive(Debug, Parser)]
#[clap(author, version, about)]
struct Cli {
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
    #[clap(flatten)]
    common_kafka_options: CommonKafkaOpts,

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
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::DaqTrace(args) => daq_trace::run(args).await,
        Commands::MessageDebug(args) => message_debug::run(args).await,
    }
}

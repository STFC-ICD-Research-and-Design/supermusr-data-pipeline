use anyhow::Result;
use clap::Parser;
use rdkafka::producer::FutureProducer;
use std::net::SocketAddr;
use trace_reader::{dispatch_trace_file, load_trace_file};

// cargo run -- --broker localhost:19092

#[derive(Debug, Parser)]
#[clap(author, version, about)]
struct Cli {
    #[clap(long)]
    broker: String,

    #[clap(long)]
    username: Option<String>,

    #[clap(long)]
    password: Option<String>,

    #[clap(long = "group", default_value = "trace-producer")]
    consumer_group: String,

    #[clap(long, default_value = "Traces")]
    trace_topic: String,

    #[clap(long, default_value = "127.0.0.1:9090")]
    observability_address: SocketAddr,

    #[clap(long)]
    file_name: Option<String>,

    #[clap(long, default_value = "1")]
    number_of_events: usize,

    #[clap(long, default_value = "true")]
    random_sample: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let args = Cli::parse();

    let file_name = args.file_name.unwrap_or(
        //"../../Data/Traces/MuSR_A27_B28_C29_D30_Apr2021_Ag_ZF_InstDeg_Slit60_short.traces".to_owned(),
        "../../Data/Traces/MuSR_A41_B42_C43_D44_Apr2021_Ag_ZF_IntDeg_Slit60_short.traces"
            .to_owned(),
    );

    let client_config =
        common::generate_kafka_client_config(&args.broker, &args.username, &args.password);

    let producer: FutureProducer = client_config.create()?;

    let trace_file = load_trace_file(&file_name)?;
    dispatch_trace_file(trace_file, [0].into(), &producer, &args.trace_topic, 6000).await?;
    Ok(())
}

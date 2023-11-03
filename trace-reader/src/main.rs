use anyhow::Result;
use clap::Parser;
use rand::{seq::IteratorRandom, thread_rng};
use rdkafka::producer::FutureProducer;
use std::net::SocketAddr;
use trace_reader::{dispatch_trace_file, load_trace_file};

// cargo run -- --broker localhost:19092

#[derive(Debug, Parser)]
#[clap(author, version, about)]
struct Cli {
    #[clap(short, long)]
    broker: String,

    #[clap(short, long)]
    username: Option<String>,

    #[clap(short, long)]
    password: Option<String>,

    #[clap(short, long = "group", default_value = "trace-producer")]
    consumer_group: String,

    #[clap(short, long, default_value = "Traces")]
    trace_topic: String,

    #[clap(short, long, default_value = "127.0.0.1:9090")]
    observability_address: SocketAddr,

    #[clap(short, long)]
    file_name: Option<String>,

    #[clap(short, long, default_value = "1")]
    number_of_events: usize,

    #[clap(short, long, default_value = "false")]
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
    let total_events = trace_file.get_num_events();
    let num_events = if args.number_of_events == 0 {
        total_events
    } else {
        args.number_of_events
    };
    let event_indices: Vec<_> = if args.random_sample {
        (0..num_events)
            .map(|_| {
                (0..total_events)
                    .choose(&mut thread_rng())
                    .unwrap_or_default()
            })
            .collect()
    } else {
        (0..total_events).cycle().take(num_events).collect()
    };
    dispatch_trace_file(
        trace_file,
        event_indices,
        &producer,
        &args.trace_topic,
        6000,
    )
    .await?;
    Ok(())
}

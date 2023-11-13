use anyhow::Result;
use clap::Parser;
use rand::{seq::IteratorRandom, thread_rng};
use rdkafka::producer::FutureProducer;

mod processing;
mod loader;
use processing::dispatch_trace_file;
use loader::load_trace_file;

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

    #[clap(short, long)]
    file_name: Option<String>,

    #[clap(short, long, default_value = "1")]
    number_of_trace_events: usize,

    #[clap(short, long, default_value = "false")]
    random_sample: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let args = Cli::parse();

    let file_name = args.file_name.expect("Cannot load trace, invalid filename or path.");

    let client_config =
        common::generate_kafka_client_config(&args.broker, &args.username, &args.password);

    let producer: FutureProducer = client_config.create()?;

    let trace_file = load_trace_file(&file_name)?;
    let total_trace_events = trace_file.get_number_of_trace_events();
    let num_trace_events = if args.number_of_trace_events == 0 {
        total_trace_events
    } else {
        args.number_of_trace_events
    };
    let trace_event_indices: Vec<_> = if args.random_sample {
        (0..num_trace_events)
            .map(|_| {
                (0..num_trace_events)
                    .choose(&mut thread_rng())
                    .unwrap_or_default()
            })
            .collect()
    } else {
        (0..num_trace_events).cycle().take(num_trace_events).collect()
    };
    dispatch_trace_file(
        trace_file,
        trace_event_indices,
        &producer,
        &args.trace_topic,
        6000,
    )
    .await?;
    Ok(())
}

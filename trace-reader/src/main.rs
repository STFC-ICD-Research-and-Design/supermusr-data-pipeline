use anyhow::Result;
use clap::Parser;
use common::{DigitizerId, FrameNumber};
use rand::{seq::IteratorRandom, thread_rng};
use rdkafka::producer::FutureProducer;
use std::path::PathBuf;

mod loader;
mod processing;
use loader::load_trace_file;
use processing::dispatch_trace_file;

#[derive(Debug, Parser)]
#[clap(author, version, about)]
struct Cli {
    #[clap(
        short,
        long,
        help = "Kafka message broker, should have format `host:port`, e.g. `localhost:19092`"
    )]
    broker: String,

    #[clap(short, long)]
    username: Option<String>,

    #[clap(short, long)]
    password: Option<String>,

    #[clap(short, long)]
    consumer_group: String,

    #[clap(
        short,
        long,
        help = "The Kafka topic that trace messages are produced to"
    )]
    trace_topic: String,

    #[clap(short, long, help = "Relative path to the .trace file to be read")]
    file_name: PathBuf,

    #[clap(
        short,
        long,
        default_value = "0",
        help = "The frame number to assign the message."
    )]
    frame_number: FrameNumber,

    #[clap(
        short,
        long,
        default_value = "0",
        help = "The frame number to assign the message."
    )]
    digitizer_id: DigitizerId,

    #[clap(
        short,
        long,
        default_value = "1",
        help = "The number of trace events to read. If zero, then all trace events are read."
    )]
    number_of_trace_events: usize,

    #[clap(
        short,
        long,
        default_value = "false",
        help = "If set, then trace events are sampled randomly with replacement, if not set then trace events are read in order."
    )]
    random_sample: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let args = Cli::parse();

    let client_config =
        common::generate_kafka_client_config(&args.broker, &args.username, &args.password);

    let producer: FutureProducer = client_config
        .create()
        .expect("Failed to create Kafka Producer");

    let trace_file = load_trace_file(args.file_name).expect("Failed to Load Trace File");
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
        (0..num_trace_events)
            .cycle()
            .take(num_trace_events)
            .collect()
    };

    dispatch_trace_file(
        trace_file,
        trace_event_indices,
        args.frame_number,
        args.digitizer_id,
        &producer,
        &args.trace_topic,
        6000,
    )
    .await
    .expect("Failed to Dispatch Trace File");
    Ok(())
}

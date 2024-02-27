use clap::Parser;
use rand::{seq::IteratorRandom, thread_rng};
use rdkafka::producer::FutureProducer;
use std::path::PathBuf;
use supermusr_common::{DigitizerId, FrameNumber};

mod loader;
mod processing;
use loader::load_trace_file;
use processing::dispatch_trace_file;

#[derive(Debug, Parser)]
#[clap(author, version, about)]
struct Cli {
    /// Kafka message broker, should have format `host:port`, e.g. `localhost:19092`
    #[clap(long)]
    broker: String,

    /// Optional Kafka username
    #[clap(long)]
    username: Option<String>,

    /// Optional Kafka password
    #[clap(long)]
    password: Option<String>,

    /// Name of the Kafka consumer group
    #[clap(long)]
    consumer_group: String,

    /// The Kafka topic that trace messages are produced to
    #[clap(long)]
    trace_topic: String,

    /// Relative path to the .trace file to be read
    #[clap(long)]
    file_name: PathBuf,

    /// The frame number to assign the message
    #[clap(long, default_value = "0")]
    frame_number: FrameNumber,

    /// The digitizer id to assign the message
    #[clap(long, default_value = "0")]
    digitizer_id: DigitizerId,

    /// The number of trace events to read. If zero, then all trace events are read
    #[clap(long, default_value = "1")]
    number_of_trace_events: usize,

    /// If set, then trace events are sampled randomly with replacement, if not set then trace events are read in order
    #[clap(long, default_value = "false")]
    random_sample: bool,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let args = Cli::parse();

    let client_config = supermusr_common::generate_kafka_client_config(
        &args.broker,
        &args.username,
        &args.password,
    );

    let producer: FutureProducer = client_config
        .create()
        .expect("Kafka Producer should be created");

    let trace_file = load_trace_file(args.file_name).expect("Trace File should load");
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
    .expect("Trace File should be dispatched to Kafka");
}

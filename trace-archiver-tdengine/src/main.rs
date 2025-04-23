//! This crate uses the benchmarking tool for testing the performance of implementated time-series databases.
//!

mod tdengine;

use clap::Parser;
use rdkafka::{
    consumer::{CommitMode, Consumer},
    message::Message,
};
use supermusr_common::CommonKafkaOpts;
use supermusr_streaming_types::dat2_digitizer_analog_trace_v2_generated::{
    digitizer_analog_trace_message_buffer_has_identifier, root_as_digitizer_analog_trace_message,
};
use tdengine::{wrapper::TDEngine, TimeSeriesEngine};
use tracing::{debug, info, warn};

#[derive(Parser)]
#[clap(author, version, about)]
pub(crate) struct Cli {
    #[clap(flatten)]
    common_kafka_options: CommonKafkaOpts,

    /// Kafka consumer group
    #[clap(long)]
    kafka_consumer_group: String,

    /// Kafka topic on which to listen for digitiser trace messages
    #[clap(long)]
    trace_topic: String,

    /// TDengine dsn
    #[clap(long)]
    td_dsn: String,

    /// Optional TDengine username
    #[clap(long)]
    td_username: Option<String>,

    /// Optional TDengine password
    #[clap(long)]
    td_password: Option<String>,

    /// TDengine database name
    #[clap(long)]
    td_database: String,

    /// Number of expected channels in a message
    #[clap(long)]
    num_channels: usize,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let args = Cli::parse();

    debug!("Createing TDEngine instance");
    let mut tdengine: TDEngine = TDEngine::from_optional(
        args.td_dsn,
        args.td_username,
        args.td_password,
        args.td_database,
    )
    .await
    .expect("TDengine should be created");

    //  All other modes require the TDEngine to be initialised
    tdengine
        .create_database()
        .await
        .expect("TDengine database should be created");
    tdengine
        .init_with_channel_count(args.num_channels)
        .await
        .expect("TDengine should initialise with given channel count");

    //  All other modes require a kafka builder, a topic, and redpanda consumer
    debug!("Creating Kafka instance");

    let kafka_opts = args.common_kafka_options;
    let consumer = supermusr_common::create_default_consumer(
        &kafka_opts.broker,
        &kafka_opts.username,
        &kafka_opts.password,
        &args.kafka_consumer_group,
        Some(&[args.trace_topic.as_str()])
    ).expect("Topic list should be non-empty, this should never fail.");

    debug!("Begin Listening For Messages");
    loop {
        match consumer.recv().await {
            Ok(message) => {
                match message.payload() {
                    Some(payload) => {
                        if digitizer_analog_trace_message_buffer_has_identifier(payload) {
                            match root_as_digitizer_analog_trace_message(payload) {
                                Ok(message) => {
                                    info!(
                                        "Trace packet: dig. ID: {}, metadata: {:?}",
                                        message.digitizer_id(),
                                        message.metadata()
                                    );
                                    if let Err(e) = tdengine.process_message(&message).await {
                                        warn!("Error processing message : {e}");
                                    }
                                    if let Err(e) = tdengine.post_message().await {
                                        warn!("Error posting message to tdengine : {e}");
                                    }
                                }
                                Err(e) => warn!("Failed to parse message: {0}", e),
                            }
                        } else {
                            warn!("Message payload missing identifier.")
                        }
                    }
                    None => warn!("Error extracting payload from message."),
                };
                consumer
                    .commit_message(&message, CommitMode::Async)
                    .unwrap();
            }
            Err(e) => warn!("Error recieving message from server: {e}"),
        }
    }
}

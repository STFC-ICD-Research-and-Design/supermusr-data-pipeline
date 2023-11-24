//! This crate uses the benchmarking tool for testing the performance of implementated time-series databases.
//!

use clap::Parser;

use log::{debug, info, warn};

mod tdengine;

use anyhow::Result;

use tdengine::{wrapper::TDEngine, TimeSeriesEngine};

use rdkafka::{
    consumer::{stream_consumer::StreamConsumer, CommitMode, Consumer},
    message::Message,
};

use streaming_types::dat1_digitizer_analog_trace_v1_generated::{
    digitizer_analog_trace_message_buffer_has_identifier, root_as_digitizer_analog_trace_message,
};

//cargo run -- --kafka-broker=localhost:19092 --kafka-topic=Traces --td-dsn=172.16.105.238:6041 --td-database=tracelogs --num-channels=8

#[derive(Parser)]
#[clap(author, version, about)]
pub(crate) struct Cli {
    #[clap(long, short = 'b', env = "KAFKA_BROKER")]
    kafka_broker: String,

    #[clap(long, short = 'u', env = "KAFKA_USER")]
    kafka_username: Option<String>,

    #[clap(long, short = 'p', env = "KAFKA_PASSWORD")]
    kafka_password: Option<String>,

    #[clap(
        long,
        short = 'g',
        env = "KAFKA_CONSUMER_GROUP",
        default_value = "trace-consumer"
    )]
    kafka_consumer_group: String,

    #[clap(long, short = 'k', env = "KAFKA_TOPIC")]
    kafka_topic: String,

    #[clap(long, short = 'B', env = "TDENGINE_DSN")]
    td_dsn: String,

    #[clap(long, short = 'U', env = "TDENGINE_USER")]
    td_username: Option<String>,

    #[clap(long, short = 'P', env = "TDENGINE_PASSWORD")]
    td_password: Option<String>,

    #[clap(long, short = 'D', env = "TDENGINE_DATABASE")]
    td_database: String,

    #[clap(long, short = 'C', env = "NUM_CHANNELS")]
    num_channels: usize,
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let cli = Cli::parse();

    //  All other modes require a TDEngine instance
    debug!("Createing TDEngine instance");
    let mut tdengine: TDEngine = TDEngine::from_optional(
        cli.td_dsn,
        cli.td_username,
        cli.td_password,
        cli.td_database,
    )
    .await
    .expect("Cannot create TDengine");

    //  All other modes require the TDEngine to be initialised
    tdengine.create_database().await?;
    tdengine
        .init_with_channel_count(cli.num_channels)
        .await
        .expect("Cannot initialise TDengine");

    //  All other modes require a kafka builder, a topic, and redpanda consumer
    debug!("Creating Kafka instance");

    let mut client_config = common::generate_kafka_client_config(
        &cli.kafka_broker,
        &cli.kafka_username,
        &cli.kafka_password,
    );

    let consumer: StreamConsumer = client_config
        .set("group.id", &cli.kafka_consumer_group)
        .set("enable.partition.eof", "false")
        .set("session.timeout.ms", "6000")
        .set("enable.auto.commit", "false")
        .create()?;
    consumer.subscribe(&[&cli.kafka_topic])?;

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

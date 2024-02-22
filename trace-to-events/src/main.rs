mod parameters;
mod processing;
mod pulse_detection;

use clap::Parser;
use parameters::Mode;
use rdkafka::{
    consumer::{stream_consumer::StreamConsumer, CommitMode, Consumer},
    message::Message,
    producer::{FutureProducer, FutureRecord},
};
use std::{path::PathBuf, time::Duration};
use supermusr_streaming_types::dat1_digitizer_analog_trace_v1_generated::{
    digitizer_analog_trace_message_buffer_has_identifier, root_as_digitizer_analog_trace_message,
};

#[derive(Debug, Parser)]
#[clap(author, version, about)]
struct Cli {
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
    event_topic: String,

    #[clap(long)]
    save_file: Option<PathBuf>,

    #[command(subcommand)]
    pub(crate) mode: Mode,
}

#[tokio::main]
async fn main() {
    env_logger::init();

    let args = Cli::parse();

    let mut client_config = supermusr_common::generate_kafka_client_config(
        &args.broker,
        &args.username,
        &args.password,
    );

    let producer: FutureProducer = client_config
        .create()
        .expect("Kafka Producer should be created");

    let consumer: StreamConsumer = client_config
        .set("group.id", &args.consumer_group)
        .set("enable.partition.eof", "false")
        .set("session.timeout.ms", "6000")
        .set("enable.auto.commit", "false")
        .create()
        .expect("Kafka Consumer should be created");

    consumer
        .subscribe(&[&args.trace_topic])
        .expect("Kafka Consumer should subscribe to trace-topic");

    loop {
        match consumer.recv().await {
            Ok(m) => {
                log::debug!(
                    "key: '{:?}', topic: {}, partition: {}, offset: {}, timestamp: {:?}",
                    m.key(),
                    m.topic(),
                    m.partition(),
                    m.offset(),
                    m.timestamp()
                );

                if let Some(payload) = m.payload() {
                    if digitizer_analog_trace_message_buffer_has_identifier(payload) {
                        match root_as_digitizer_analog_trace_message(payload) {
                            Ok(thing) => {
                                match producer
                                    .send(
                                        FutureRecord::to(&args.event_topic)
                                            .payload(&processing::process(
                                                &thing,
                                                &args.mode,
                                                args.save_file.as_deref(),
                                            ))
                                            .key("test"),
                                        Duration::from_secs(0),
                                    )
                                    .await
                                {
                                    Ok(_) => {
                                        log::trace!("Published event message");
                                    }
                                    Err(e) => {
                                        log::error!("{:?}", e);
                                    }
                                }
                            }
                            Err(e) => {
                                log::warn!("Failed to parse message: {}", e);
                            }
                        }
                    } else {
                        log::warn!("Unexpected message type on topic \"{}\"", m.topic());
                    }
                }

                consumer.commit_message(&m, CommitMode::Async).unwrap();
            }
            Err(e) => log::warn!("Kafka error: {}", e),
        };
    }
}

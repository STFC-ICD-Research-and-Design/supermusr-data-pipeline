mod metrics;
mod processing;

use anyhow::Result;
use clap::Parser;
use common::Time;
use kagiyama::{AlwaysReady, Watcher};
use rdkafka::{
    consumer::{stream_consumer::StreamConsumer, CommitMode, Consumer},
    message::Message,
    producer::{FutureProducer, FutureRecord},
};
use std::{net::SocketAddr, time::Duration};
use streaming_types::dev1_digitizer_event_v1_generated::{
    digitizer_event_list_message_buffer_has_identifier, root_as_digitizer_event_list_message,
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
    event_topic: String,

    #[clap(long)]
    histogram_topic: String,

    #[clap(long, default_value = "127.0.0.1:9090")]
    observability_address: SocketAddr,

    #[clap(long)]
    time_start: Time,

    #[clap(long)]
    time_bin_width: Time,

    #[clap(long)]
    time_end: Time,
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let args = Cli::parse();

    let mut watcher = Watcher::<AlwaysReady>::default();
    metrics::register(&watcher);
    watcher.start_server(args.observability_address).await;

    let mut client_config =
        common::generate_kafka_client_config(&args.broker, &args.username, &args.password);

    let consumer: StreamConsumer = client_config
        .set("group.id", &args.consumer_group)
        .set("enable.partition.eof", "false")
        .set("session.timeout.ms", "6000")
        .set("enable.auto.commit", "false")
        .create()?;

    consumer.subscribe(&[&args.event_topic])?;

    let producer: FutureProducer = client_config.create()?;

    let edges = processing::make_bins_edges(args.time_start, args.time_end, args.time_bin_width);

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
                    if digitizer_event_list_message_buffer_has_identifier(payload) {
                        metrics::MESSAGES_RECEIVED
                            .get_or_create(&metrics::MessagesReceivedLabels::new(
                                metrics::MessageKind::Trace,
                            ))
                            .inc();
                        match root_as_digitizer_event_list_message(payload) {
                            Ok(thing) => {
                                match producer
                                    .send(
                                        FutureRecord::to(&args.histogram_topic)
                                            .payload(&processing::process(
                                                &thing,
                                                args.time_bin_width,
                                                edges.clone(),
                                            ))
                                            .key("test"),
                                        Duration::from_secs(0),
                                    )
                                    .await
                                {
                                    Ok(_) => {
                                        log::trace!("Published histogram message");
                                        metrics::MESSAGES_PROCESSED.inc();
                                    }
                                    Err(e) => {
                                        log::error!("{:?}", e);
                                        metrics::FAILURES
                                            .get_or_create(&metrics::FailureLabels::new(
                                                metrics::FailureKind::KafkaPublishFailed,
                                            ))
                                            .inc();
                                    }
                                }
                            }
                            Err(e) => {
                                log::warn!("Failed to parse message: {}", e);
                                metrics::FAILURES
                                    .get_or_create(&metrics::FailureLabels::new(
                                        metrics::FailureKind::UnableToDecodeMessage,
                                    ))
                                    .inc();
                            }
                        }
                    } else {
                        log::warn!("Unexpected message type on topic \"{}\"", m.topic());
                        metrics::MESSAGES_RECEIVED
                            .get_or_create(&metrics::MessagesReceivedLabels::new(
                                metrics::MessageKind::Unknown,
                            ))
                            .inc();
                    }
                }

                consumer.commit_message(&m, CommitMode::Async).unwrap();
            }
            Err(e) => log::warn!("Kafka error: {}", e),
        };
    }
}

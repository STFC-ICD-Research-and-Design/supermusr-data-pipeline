mod processing;

use anyhow::Result;
use clap::Parser;
use metrics::{self, counter};
use rdkafka::{
    consumer::{stream_consumer::StreamConsumer, CommitMode, Consumer},
    message::Message,
    producer::{FutureProducer, FutureRecord},
};
use std::{net::SocketAddr, time::Duration};
use supermusr_common::{
    metrics::{
        failures::{self, FailureKind},
        messages_received::{self, MessageKind},
        metric_names::{FAILURES, MESSAGES_PROCESSED, MESSAGES_RECEIVED},
    },
    Time,
};
use supermusr_streaming_types::dev1_digitizer_event_v1_generated::{
    digitizer_event_list_message_buffer_has_identifier, root_as_digitizer_event_list_message,
};
use tracing::{debug, error, trace, warn};

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
    tracing_subscriber::fmt::init();

    let args = Cli::parse();

    let mut client_config = supermusr_common::generate_kafka_client_config(
        &args.broker,
        &args.username,
        &args.password,
    );

    let consumer: StreamConsumer = client_config
        .set("group.id", &args.consumer_group)
        .set("enable.partition.eof", "false")
        .set("session.timeout.ms", "6000")
        .set("enable.auto.commit", "false")
        .create()?;

    consumer.subscribe(&[&args.event_topic])?;

    let producer: FutureProducer = client_config.create()?;

    let edges = processing::make_bins_edges(args.time_start, args.time_end, args.time_bin_width);

    // Metrics
    metrics::describe_counter!(
        MESSAGES_RECEIVED,
        metrics::Unit::Count,
        "Number of messages received"
    );
    metrics::describe_counter!(
        MESSAGES_PROCESSED,
        metrics::Unit::Count,
        "Number of messages processed"
    );
    metrics::describe_counter!(
        FAILURES,
        metrics::Unit::Count,
        "Number of failures encountered"
    );

    loop {
        match consumer.recv().await {
            Ok(m) => {
                debug!(
                    "key: '{:?}', topic: {}, partition: {}, offset: {}, timestamp: {:?}",
                    m.key(),
                    m.topic(),
                    m.partition(),
                    m.offset(),
                    m.timestamp()
                );

                if let Some(payload) = m.payload() {
                    if digitizer_event_list_message_buffer_has_identifier(payload) {
                        counter!(
                            MESSAGES_RECEIVED,
                            &[messages_received::get_label(MessageKind::Trace)]
                        )
                        .increment(1);
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
                                        trace!("Published histogram message");
                                        counter!(MESSAGES_PROCESSED).increment(1);
                                    }
                                    Err(e) => {
                                        error!("{:?}", e);
                                        counter!(
                                            FAILURES,
                                            &[failures::get_label(FailureKind::KafkaPublishFailed)]
                                        )
                                        .increment(1);
                                    }
                                }
                            }
                            Err(e) => {
                                warn!("Failed to parse message: {}", e);
                                counter!(
                                    FAILURES,
                                    &[failures::get_label(FailureKind::UnableToDecodeMessage)]
                                )
                                .increment(1);
                            }
                        }
                    } else {
                        warn!("Unexpected message type on topic \"{}\"", m.topic());
                        counter!(
                            MESSAGES_RECEIVED,
                            &[messages_received::get_label(MessageKind::Unknown)]
                        )
                        .increment(1);
                    }
                }

                consumer.commit_message(&m, CommitMode::Async).unwrap();
            }
            Err(e) => warn!("Kafka error: {}", e),
        };
    }
}

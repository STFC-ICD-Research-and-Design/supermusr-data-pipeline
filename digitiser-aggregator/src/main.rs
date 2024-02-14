mod data;
mod frame;

use crate::data::EventData;
use clap::Parser;
use frame::FrameCache;
use rdkafka::{
    consumer::{stream_consumer::StreamConsumer, CommitMode, Consumer},
    message::{BorrowedMessage, Message},
    producer::{FutureProducer, FutureRecord},
    util::Timeout,
};
use std::{net::SocketAddr, time::Duration};
use supermusr_common::DigitizerId;
use supermusr_streaming_types::dev1_digitizer_event_v1_generated::{
    digitizer_event_list_message_buffer_has_identifier, root_as_digitizer_event_list_message,
};
use tracing::{debug, error, warn};

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
    input_topic: String,

    #[clap(long)]
    output_topic: String,

    #[clap(short, long)]
    digitiser_ids: Vec<DigitizerId>,

    #[clap(long, default_value = "500")]
    frame_ttl_ms: u64,

    #[clap(long, default_value = "500")]
    cache_poll_ms: u64,

    #[clap(long, default_value = "127.0.0.1:9090")]
    observability_address: SocketAddr,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let args = Cli::parse();

    let consumer: StreamConsumer = supermusr_common::generate_kafka_client_config(
        &args.broker,
        &args.username,
        &args.password,
    )
    .set("group.id", &args.consumer_group)
    .set("enable.partition.eof", "false")
    .set("session.timeout.ms", "6000")
    .set("enable.auto.commit", "false")
    .create()
    .expect("kafka consumer should be created");

    consumer
        .subscribe(&[&args.input_topic])
        .expect("kafka topic should be subscribed");

    let producer = supermusr_common::generate_kafka_client_config(
        &args.broker,
        &args.username,
        &args.password,
    )
    .create()
    .unwrap();

    let ttl = Duration::from_millis(args.frame_ttl_ms);

    let mut cache = FrameCache::<EventData>::new(ttl, args.digitiser_ids);

    let mut cache_poll_interval = tokio::time::interval(Duration::from_millis(args.cache_poll_ms));
    loop {
        tokio::select! {
            event = consumer.recv() => {
                match event {
                    Ok(msg) => {
                        on_message(&mut cache, &producer, &args.output_topic, &msg).await;
                        consumer.commit_message(&msg, CommitMode::Async).unwrap();
                    }
                    Err(e) => warn!("Kafka error: {}", e),
                };
            }
            _ = cache_poll_interval.tick() => {
                cache_poll(&mut cache, &producer, &args.output_topic).await;
            }
        }
    }
}

async fn on_message(
    cache: &mut FrameCache<EventData>,
    producer: &FutureProducer,
    output_topic: &str,
    msg: &BorrowedMessage<'_>,
) {
    if let Some(payload) = msg.payload() {
        if digitizer_event_list_message_buffer_has_identifier(payload) {
            match root_as_digitizer_event_list_message(payload) {
                Ok(msg) => {
                    debug!("Event packet: metadata: {:?}", msg.metadata());
                    cache.push(msg.digitizer_id(), msg.metadata().into(), msg.into());
                    cache_poll(cache, producer, output_topic).await;
                }
                Err(e) => {
                    warn!("Failed to parse message: {}", e);
                }
            }
        } else {
            warn!("Unexpected message type on topic \"{}\"", msg.topic());
        }
    }
}

async fn cache_poll(
    cache: &mut FrameCache<EventData>,
    producer: &FutureProducer,
    output_topic: &str,
) {
    if let Some(frame) = cache.poll() {
        let data: Vec<u8> = frame.into();

        match producer
            .send(
                FutureRecord::to(output_topic)
                    .payload(&data)
                    .key(&"todo".to_string()),
                Timeout::After(Duration::from_millis(100)),
            )
            .await
        {
            Ok(r) => debug!("Delivery: {:?}", r),
            Err(e) => error!("Delivery failed: {:?}", e),
        };
    }
}

//TODO
#[allow(unused)]
mod buffer;

//TODO
#[allow(unused)]
mod config;

//TODO
#[allow(unused)]
mod event_data;

//TODO
#[allow(unused)]
mod frame;

use crate::{config::Config, frame::Frame};
use anyhow::Result;
use clap::Parser;
use rdkafka::{
    config::ClientConfig,
    consumer::{stream_consumer::StreamConsumer, CommitMode, Consumer},
    message::Message,
    producer::{FutureProducer, FutureRecord},
};
use std::time::Duration;
use streaming_types::dev1_digitizer_event_v1_generated::{
    digitizer_event_list_message_buffer_has_identifier, root_as_digitizer_event_list_message,
};

#[derive(Debug, Parser)]
#[clap(author, version, about)]
struct Cli {
    /// Path to the configuration file
    #[clap(long = "config", short = 'c', default_value = "./config.toml")]
    config_filename: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let args = Cli::parse();
    log::debug!("Args: {:?}", args);

    let config = Config::from_file(&args.config_filename)?;
    log::debug!("Config: {:?}", config);

    let consumer: StreamConsumer = ClientConfig::new()
        .set("bootstrap.servers", &config.broker.address)
        .set("security.protocol", "sasl_plaintext")
        .set("sasl.mechanisms", "SCRAM-SHA-256")
        .set("sasl.username", &config.broker.username)
        .set("sasl.password", &config.broker.password)
        .set("group.id", &config.broker.group)
        .set("enable.partition.eof", "false")
        .set("session.timeout.ms", "6000")
        .set("enable.auto.commit", "false")
        .create()?;

    consumer.subscribe(&[&config.topics.source])?;

    let producer: FutureProducer = ClientConfig::new()
        .set("bootstrap.servers", &config.broker.address)
        .set("security.protocol", "sasl_plaintext")
        .set("sasl.mechanisms", "SCRAM-SHA-256")
        .set("sasl.username", &config.broker.username)
        .set("sasl.password", &config.broker.password)
        .create()?;

    loop {
        match consumer.recv().await {
            Err(e) => log::warn!("Kafka error: {}", e),
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
                        if let Ok(thing) = root_as_digitizer_event_list_message(payload) {
                            log::info!(
                                "Dig ID: {}, Metadata: {:?}",
                                thing.digitizer_id(),
                                thing.metadata()
                            );

                            let mut frame = Frame::new(thing.metadata().into());
                            frame.push(&thing)?;

                            match frame.as_payload() {
                                Ok(payload) => {
                                    match producer
                                        .send(
                                            FutureRecord::to(&config.topics.destination)
                                                .payload(&payload)
                                                .key("test"),
                                            Duration::from_secs(0),
                                        )
                                        .await
                                    {
                                        Ok(_) => {
                                            log::trace!("Published assembled frame message");
                                        }
                                        Err(e) => {
                                            log::error!("{:?}", e);
                                        }
                                    }
                                }
                                Err(e) => {
                                    log::error!("Failed to serialise frame ({})", e);
                                }
                            }
                        }
                    } else {
                        log::warn!("Unexpected message type on topic \"{}\"", m.topic());
                    }
                }

                consumer.commit_message(&m, CommitMode::Async).unwrap();
            }
        };
    }
}

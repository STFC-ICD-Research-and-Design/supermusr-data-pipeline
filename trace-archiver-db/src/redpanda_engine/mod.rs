use crate::error::{Error, MessageError};
use anyhow::Result;

use itertools::Itertools;
use redpanda::{
    consumer::RedpandaConsumer,
    error::KafkaError,
    message::{BorrowedMessage, Message},
    RedpandaBuilder,
};
#[cfg(feature = "benchmark")]
use redpanda::{producer::RedpandaRecord, RedpandaProducer};
use streaming_types::dat1_digitizer_analog_trace_v1_generated::{
    digitizer_analog_trace_message_buffer_has_identifier, root_as_digitizer_analog_trace_message,
    DigitizerAnalogTraceMessage,
};

pub fn new_builder_from_optional(
    url: Option<String>,
    port: Option<u32>,
    user: Option<String>,
    password: Option<String>,
    group: Option<String>,
) -> Result<RedpandaBuilder> {
    let broker = format!(
        "{0}:{1}",
        url.ok_or(Error::EnvVar("Redpanda URL"))?,
        port.ok_or(Error::EnvVar("Redpanda Port"))?,
    );
    let user = user.ok_or(Error::EnvVar("Redpanda User"))?;
    let password = password.ok_or(Error::EnvVar("Redpanda Password"))?;
    let group = group.ok_or(Error::EnvVar("Redpanda Group"))?;
    let mut builder = RedpandaBuilder::default();
    builder
        .set_bootstrap_servers(&broker)
        .set_creation_timeout_ms(3000);
    if !group.is_empty() {
        builder.set_group_id(&group);
    }
    Ok(builder)
}

pub async fn create_topic(builder: &RedpandaBuilder, topic: &str) -> Result<(), KafkaError> {
    let admin = builder.build_admin_client().await?;
    admin.create_topic(topic, 1, 1).await
}

pub fn new_consumer(
    builder: &RedpandaBuilder,
    topic: &str,
) -> Result<RedpandaConsumer, KafkaError> {
    let consumer = builder.build_consumer()?;
    if let Err(e) = consumer.subscribe(&[topic]) {
        if let KafkaError::Subscription(str) = e.clone() {
            if str.eq_ignore_ascii_case(&format!("Invalid topic name {topic}")) {
                log::info!("Topic: {topic} not found.");
                return Err(e);
            } else {
                log::info!("Cannot subscribe to topic : {str}");
                return Err(e);
            }
        } else {
            log::info!("Subscription error : {e}");
            return Err(e);
        }
    }
    Ok(consumer)
}

pub async fn consumer_recv(consumer: &RedpandaConsumer) -> Result<BorrowedMessage, Error> {
    match consumer.recv().await {
        Err(e) => Err(e.into()),
        Ok(message) => match message.payload() {
            Some(payload) => {
                if digitizer_analog_trace_message_buffer_has_identifier(payload) {
                    Ok(message)
                } else {
                    Err(MessageError::UnexpectedMessageWithTopic(message.topic().to_owned()).into())
                }
            }
            None => {
                Err(MessageError::NoPayloadInMessageWithTopic(message.topic().to_owned()).into())
            }
        },
    }
}

pub fn extract_payload<'a, 'b: 'a>(
    message: &'b BorrowedMessage<'b>,
) -> Result<DigitizerAnalogTraceMessage<'a>, Error> {
    let payload = message.payload().unwrap();
    if !digitizer_analog_trace_message_buffer_has_identifier(payload) {
        return Err(MessageError::UnexpectedMessageWithTopic(message.topic().to_owned()).into());
    }
    match root_as_digitizer_analog_trace_message(payload) {
        Ok(data) => {
            log::info!(
                "Trace packet: dig. ID: {}, metadata: {:?}",
                data.digitizer_id(),
                data.metadata()
            );
            Ok(data)
        }
        Err(e) => {
            log::warn!("Failed to parse message: {0}", e);
            Err(MessageError::FailedToParseMessageWithTopic(message.topic().to_owned(), e).into())
        }
    }
}

#[cfg(feature = "benchmark")]
pub(crate) fn new_producer(builder: &RedpandaBuilder) -> Result<RedpandaProducer, KafkaError> {
    builder.build_producer()
}

#[cfg(feature = "benchmark")]
pub(crate) async fn producer_post(
    producer: &RedpandaProducer,
    topic: &str,
    message: &[u8],
) -> Result<()> {
    let bytes = message.iter().copied().collect_vec();
    let record = RedpandaRecord::new(topic, None, bytes, None);
    producer
        .send_result(&record)
        .map_err(|e| e.0)?
        .await?
        .map_err(|e| e.0)?;
    Ok(())
}

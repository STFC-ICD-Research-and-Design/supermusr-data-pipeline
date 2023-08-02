use anyhow::{anyhow, Result};

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

use crate::utils::{
    log_then_panic, log_then_panic_t, unwrap_num_or_env_var, unwrap_string_or_env_var,
};

pub fn new_builder_from_optional(
    url: Option<String>,
    port: Option<u32>,
    user: Option<String>,
    password: Option<String>,
    group: Option<String>,
) -> RedpandaBuilder {
    let broker = format!(
        "{0}:{1}",
        unwrap_string_or_env_var(url, "REDPANDA_URL"),
        unwrap_num_or_env_var(port, "REDPANDA_PORT"),
    );
    let user = unwrap_string_or_env_var(user, "REDPANDA_USER");
    let password = unwrap_string_or_env_var(password, "REDPANDA_PASSWORD");
    let group = unwrap_string_or_env_var(group, "REDPANDA_CONSUMER_GROUP");
    let mut builder = RedpandaBuilder::default();
    builder
        .set_bootstrap_servers(&broker)
        .set_creation_timeout_ms(3000);
    if !group.is_empty() {
        builder.set_group_id(&group);
    }
    builder
}

pub async fn create_topic(builder: &RedpandaBuilder, topic: &str) -> Result<()> {
    let admin = builder.build_admin_client().await?;
    Ok(admin.create_topic(topic, 1, 1).await?)
}

pub fn new_consumer(builder: &RedpandaBuilder, topic: &str) -> Option<RedpandaConsumer> {
    let consumer = builder
        .build_consumer()
        .unwrap_or_else(|e| log_then_panic_t(format!("Cannot create consumer : {e}")));
    if let Err(e) = consumer.subscribe(&[topic]) {
        if let KafkaError::Subscription(str) = e {
            if str.eq_ignore_ascii_case(&format!("Invalid topic name {topic}")) {
                log::info!("Topic: {topic} not found.");
                return None;
            } else {
                log_then_panic(format!("Cannot subscribe to topic : {str}"));
            }
        } else {
            log_then_panic(format!("Subscription error : {e}"));
        }
    }
    Some(consumer)
}

pub async fn consumer_recv(consumer: &RedpandaConsumer) -> Result<BorrowedMessage> {
    match consumer.recv().await {
        Err(e) => Err(e.clone().into()), //log::warn!("Kafka error: {}", e),
        Ok(message) => match message.payload() {
            Some(payload) => match digitizer_analog_trace_message_buffer_has_identifier(payload) {
                true => Ok(message),
                false => Err(anyhow!(
                    "Unexpected message type on topic \"{}\"",
                    message.topic()
                )),
            },
            None => Err(anyhow!(
                "No payload found in message type on topic \"{}\"",
                message.topic()
            )),
        },
    }
}

pub fn extract_payload<'a, 'b: 'a>(
    message: &'b BorrowedMessage<'b>,
) -> Result<DigitizerAnalogTraceMessage<'a>> {
    let payload = message.payload().unwrap();
    if !digitizer_analog_trace_message_buffer_has_identifier(payload) {
        return Err(anyhow!("Unexpected message"));
    }
    match root_as_digitizer_analog_trace_message(payload) {
        Ok(data) => {
            log::info!(
                "Trace packet: dig. ID: {}, metadata: {:?}",
                data.digitizer_id(),
                data.metadata()
            );
            /*metrics::MESSAGES_RECEIVED
            .get_or_create(&metrics::MessagesReceivedLabels::new(
                metrics::MessageKind::Trace,
            ))
            .inc();*/
            Ok(data)
        }
        Err(e) => {
            log::warn!("Failed to parse message: {}", e);
            /*metrics::FAILURES
                .get_or_create(&metrics::FailureLabels::new(
                    metrics::FailureKind::UnableToDecodeMessage,
                ))
                .inc();
            */
            Err(anyhow!("Failed to parse message: {}", e.clone()))
        }
    }
}

#[cfg(feature = "benchmark")]
pub(crate) fn new_producer(builder: &RedpandaBuilder) -> RedpandaProducer {
    let producer = builder
        .build_producer()
        .unwrap_or_else(|e| log_then_panic_t(format!("Cannot create producer : {e}")));
    producer
}

#[cfg(feature = "benchmark")]
pub(crate) async fn producer_post(
    producer: &RedpandaProducer,
    topic: &str,
    message: &[u8],
) -> Result<()> {
    let bytes = message.into_iter().map(|&b| b).collect_vec();
    let record = RedpandaRecord::new(topic, None, bytes, None);
    producer
        .send_result(&record)
        .map_err(|e| e.0)?
        .await?
        .map_err(|e| e.0)?;
    Ok(())
}


use anyhow::{anyhow,Result};

use redpanda::{RedpandaBuilder, consumer::RedpandaConsumer, message::{Message, BorrowedMessage}};
use streaming_types::dat1_digitizer_analog_trace_v1_generated::{digitizer_analog_trace_message_buffer_has_identifier, root_as_digitizer_analog_trace_message, DigitizerAnalogTraceMessage};


pub struct RedpandaEngine<'a> {
    builder : RedpandaBuilder,
    consumer : RedpandaConsumer,
    borrowed_message : Option<BorrowedMessage<'a>>,
}

impl<'a> RedpandaEngine<'a> {
    pub fn new() -> Self {
        let mut builder = RedpandaBuilder::default();
        builder
            .set_bootstrap_servers("localhost:19092")
            .set_creation_timeout_ms(3000);
        let consumer = builder.build_consumer().unwrap();
        consumer.subscribe(&["MyTopic"]).unwrap();
        RedpandaEngine { builder, consumer, borrowed_message:None }
    }

    pub async fn recv(&self) -> Result<BorrowedMessage> {
        match self.consumer.recv().await {
            Err(e) => Err(e.clone().into()), //log::warn!("Kafka error: {}", e),
            Ok(message) => {
                match message.payload() {
                    Some(payload) =>
                        if digitizer_analog_trace_message_buffer_has_identifier(payload) {
                            Ok(message)
                        } else {
                            Err(anyhow!("Unexpected message type on topic \"{}\"", message.topic()))
                        },
                    None => Err(anyhow!("No payload found in message type on topic \"{}\"", message.topic()))
                }
            }
        }
    }
}


pub fn extract_payload<'a, 'b : 'a>(message : &'b BorrowedMessage<'b>) -> Result<DigitizerAnalogTraceMessage<'a>> {
    let payload = message.payload().unwrap();
    match root_as_digitizer_analog_trace_message(payload) {
        Ok(data) => {
            /*log::info!(
                "Trace packet: dig. ID: {}, metadata: {:?}",
                data.digitizer_id(),
                data.metadata()
            );
            metrics::MESSAGES_RECEIVED
                .get_or_create(&metrics::MessagesReceivedLabels::new(
                    metrics::MessageKind::Trace,
                ))
                .inc();*/
            /*if let Err(e) = file::create(&args.output, data) {
                log::warn!("Failed to save file: {}", e);
                metrics::FAILURES
                    .get_or_create(&metrics::FailureLabels::new(
                        metrics::FailureKind::FileWriteFailed,
                    ))
                    .inc();
            }*/
            Ok(data)
        }
        Err(e) => {
            /*log::warn!("Failed to parse message: {}", e);
            metrics::FAILURES
                .get_or_create(&metrics::FailureLabels::new(
                    metrics::FailureKind::UnableToDecodeMessage,
                ))
                .inc();
            */
            Err(anyhow!("Failed to parse message: {}", e.clone()))
        }
    }
}
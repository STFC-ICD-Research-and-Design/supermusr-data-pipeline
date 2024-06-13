use anyhow::Result;
use metrics::counter;
use metrics_exporter_prometheus::PrometheusBuilder;
use rdkafka::{
    consumer::{CommitMode, Consumer, StreamConsumer},
    Message,
};
use supermusr_common::metrics::{
    failures::{self, FailureKind},
    messages_received::{self, MessageKind},
    metric_names::{FAILURES, MESSAGES_RECEIVED},
};
use supermusr_streaming_types::dat2_digitizer_analog_trace_v2_generated::{
    digitizer_analog_trace_message_buffer_has_identifier, root_as_digitizer_analog_trace_message,
};
use tracing::{debug, info, warn};

use crate::{file::TraceFile, ContinuousOpts};

pub(crate) async fn run(continuous_args: ContinuousOpts) -> Result<()> {
    let common_args = continuous_args.common;

    let consumer: StreamConsumer = supermusr_common::generate_kafka_client_config(
        &common_args.broker,
        &common_args.username,
        &common_args.password,
    )
    .set("group.id", &common_args.consumer_group)
    .set("enable.partition.eof", "false")
    .set("session.timeout.ms", "6000")
    .set("enable.auto.commit", "false")
    .create()?;

    // Subscribe to trace topic
    consumer.subscribe(&[&common_args.trace_topic])?;

    // Install exporter and register metrics
    let builder = PrometheusBuilder::new();
    builder
        .with_http_listener(common_args.observability_address)
        .install()
        .expect("Prometheus metrics exporter should be setup");

    metrics::describe_counter!(
        MESSAGES_RECEIVED,
        metrics::Unit::Count,
        "Number of messages received"
    );
    metrics::describe_counter!(
        FAILURES,
        metrics::Unit::Count,
        "Number of failures encountered"
    );

    let mut file = TraceFile::create(&continuous_args.file, common_args.digitizer_count)?;

    loop {
        match consumer.recv().await {
            Err(e) => warn!("Kafka error: {}", e),
            Ok(msg) => {
                debug!(
                    "key: '{:?}', topic: {}, partition: {}, offset: {}, timestamp: {:?}",
                    msg.key(),
                    msg.topic(),
                    msg.partition(),
                    msg.offset(),
                    msg.timestamp()
                );

                if let Some(payload) = msg.payload() {
                    if digitizer_analog_trace_message_buffer_has_identifier(payload) {
                        // A message has been received from the trace topic.
                        match root_as_digitizer_analog_trace_message(payload) {
                            Ok(data) => {
                                info!(
                                    "Trace packet: dig. ID: {}, metadata: {:?}",
                                    data.digitizer_id(),
                                    data.metadata()
                                );
                                counter!(
                                    MESSAGES_RECEIVED,
                                    &[messages_received::get_label(MessageKind::Trace)]
                                )
                                .increment(1);

                                info!("Writing trace data to \"{}\"", file.filename());
                                if let Err(e) = file.push(&data) {
                                    warn!("Failed to save traces to file: {}", e);
                                    counter!(
                                        FAILURES,
                                        &[failures::get_label(FailureKind::FileWriteFailed)]
                                    )
                                    .increment(1);
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
                        // The message kind is unknown.
                        warn!("Unexpected message type on topic \"{}\"", msg.topic());
                        counter!(
                            MESSAGES_RECEIVED,
                            &[messages_received::get_label(MessageKind::Unknown)]
                        )
                        .increment(1);
                    }
                }

                consumer.commit_message(&msg, CommitMode::Async).unwrap();
            }
        };
    }
}

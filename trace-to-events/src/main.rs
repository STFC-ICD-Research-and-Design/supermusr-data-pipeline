mod timer;
mod metrics;
mod parameters;
mod processing;
mod pulse_detection;

use clap::Parser;
use kagiyama::{AlwaysReady, Watcher};
use parameters::Mode;
use rdkafka::{
    consumer::{stream_consumer::StreamConsumer, CommitMode, Consumer},
    message::Message,
    producer::{FutureProducer, FutureRecord},
};
use timer::TimerSuite;
use std::{net::SocketAddr, path::PathBuf, time::Duration};
use supermusr_streaming_types::dat1_digitizer_analog_trace_v1_generated::{
    digitizer_analog_trace_message_buffer_has_identifier, root_as_digitizer_analog_trace_message,
};
// cargo run --bin trace-to-events -- --broker localhost:19092 --trace-topic Traces --event-topic Events --group trace-to-events constant-phase-discriminator --threshold-trigger=-40,1,0

// cargo run --bin trace-reader -- --broker localhost:19092 --consumer-group trace-producer --trace-topic Traces --file-name ../Data/Traces/MuSR_A41_B42_C43_D44_Apr2021_Ag_ZF_IntDeg_Slit60_short.traces --number-of-trace-events 100 --random-sample


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

    #[clap(long, default_value = "127.0.0.1:9090")]
    observability_address: SocketAddr,

    #[clap(long)]
    save_file: Option<PathBuf>,

    #[command(subcommand)]
    pub(crate) mode: Mode,
}

#[tokio::main]
async fn main() {
    env_logger::init();

    let args = Cli::parse();

    let mut watcher = Watcher::<AlwaysReady>::default();
    metrics::register(&watcher);
    watcher.start_server(args.observability_address).await;

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

    let mut timer_suite = TimerSuite::new(100);

    loop {
        if timer_suite.has_finished() {
            timer_suite.full.end();
            timer_suite.full.accumulate();
            timer_suite.print();
            return;
        }
        match consumer.recv().await {
            Ok(m) => {
                timer_suite.full.record();
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
                        let num_bytes = payload.len();
                        metrics::MESSAGES_RECEIVED
                            .get_or_create(&metrics::MessagesReceivedLabels::new(
                                metrics::MessageKind::Trace,
                            ))
                            .inc();
                        match root_as_digitizer_analog_trace_message(payload) {
                            Ok(thing) => {
                                // Begin Timer
                                timer_suite.iteration.record();
                                // Begin Timer
                                timer_suite.processing.record();
                                let event_data_fbb = processing::process(
                                    &thing,
                                    &args.mode,
                                    args.save_file.as_deref(),
                                );
                                // End Timer
                                timer_suite.processing.end();
                                match producer
                                    .send(
                                        FutureRecord::to(&args.event_topic)
                                            .payload(event_data_fbb.finished_data())
                                            .key("test"),
                                        Duration::from_secs(0),
                                    )
                                    .await
                                {
                                    Ok(_) => {
                                        log::trace!("Published event message");
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
                                // End Timer
                                timer_suite.iteration.end();
                                timer_suite.next_message(num_bytes);
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

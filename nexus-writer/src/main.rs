mod metrics;
mod nexus;

use anyhow::{anyhow, Result};
use clap::Parser;
//use kagiyama::{AlwaysReady, Watcher};
use ndarray as _;
use ndarray_stats as _;
use nexus::{EventList, Nexus};
//use kagiyama::{prometheus::metrics::info::Info, AlwaysReady, Watcher};
use rdkafka::{
    consumer::{stream_consumer::StreamConsumer, CommitMode, Consumer},
    message::Message,
};
use std::{net::SocketAddr, path::PathBuf, sync::Arc};
use supermusr_streaming_types::{
    dev1_digitizer_event_v1_generated::root_as_digitizer_event_list_message,
    ecs_6s4t_run_stop_generated::{root_as_run_stop, run_stop_buffer_has_identifier},
    ecs_pl72_run_start_generated::{root_as_run_start, run_start_buffer_has_identifier},
};
use tokio::{sync::Mutex, time};

//  To run trace-reader
// cargo run --bin trace-reader -- --broker localhost:19092 --consumer-group trace-producer --trace-topic Traces --file-name ../Data/Traces/MuSR_A41_B42_C43_D44_Apr2021_Ag_ZF_IntDeg_Slit60_short.traces --number-of-trace-events 20 --random-sample

/*
cargo run --bin run-simulator -- --broker localhost:19092 --topic Controls --run-name Test run-start --instrument-name MUSR
cargo run --bin trace-reader -- --broker localhost:19092 --consumer-group trace-producer --trace-topic Traces --file-name ../Data/Traces/MuSR_A41_B42_C43_D44_Apr2021_Ag_ZF_IntDeg_Slit60_short.traces --number-of-trace-events 20 --random-sample
cargo run --bin run-simulator -- --broker localhost:19092 --topic Controls --run-name Test run-stop

*/

// To run trace-to-events
// cargo run --bin trace-to-events -- --broker localhost:19092 --trace-topic Traces --event-topic Events --group trace-to-events constant-phase-discriminator --threshold-trigger=-40,1,0\

// To run nexus-writer
// cargo run --bin nexus-writer -- --broker localhost:19092 --consumer-group nexus-writer --control-topic Controls --event-topic Events --file-name output/Saves
#[derive(Debug, Parser)]
#[clap(author, version, about)]
struct Cli {
    #[clap(long)]
    broker: String,

    #[clap(long)]
    username: Option<String>,

    #[clap(long)]
    password: Option<String>,

    #[clap(long)]
    consumer_group: String,

    #[clap(long)]
    control_topic: String,

    #[clap(long)]
    event_topic: Option<String>,

    #[clap(long)]
    histogram_topic: Option<String>,

    #[clap(long)]
    file_name: PathBuf,

    #[clap(long)]
    digitizer_count: Option<usize>,

    #[clap(long, default_value = "1000")]
    file_write_delay: u64,

    #[clap(long, default_value = "127.0.0.1:9090")]
    observability_address: SocketAddr,
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let args = Cli::parse();
    log::debug!("Args: {:?}", args);

    //    let mut watcher = Watcher::<AlwaysReady>::default();
    //    metrics::register(&mut watcher);
    //    watcher.start_server(args.observability_address).await;

    let consumer: StreamConsumer = supermusr_common::generate_kafka_client_config(
        &args.broker,
        &args.username,
        &args.password,
    )
    .set("group.id", &args.consumer_group)
    .set("enable.partition.eof", "false")
    .set("session.timeout.ms", "6000")
    .set("enable.auto.commit", "false")
    .create()?;

    //  This line can be simplified when is it clear which topics we need
    let topics_to_subscribe = [
        Some(args.control_topic.as_str()),
        args.event_topic.as_deref(),
        args.histogram_topic.as_deref(),
    ]
    .into_iter()
    .flatten()
    .collect::<Vec<&str>>();
    if topics_to_subscribe.is_empty() {
        return Err(anyhow!(
            "Nothing to do (no message type requested to be saved)"
        ));
    }
    consumer.subscribe(&topics_to_subscribe)?;

    let nexus = Arc::new(Mutex::new(Nexus::<EventList>::new()));

    //  File writing takes place in a dedicated thread as the main loop is blocking
    {
        let nexus = nexus.clone();
        tokio::spawn(async move {
            loop {
                time::sleep(time::Duration::from_millis(100)).await;
                match nexus
                    .lock()
                    .await
                    .write_files(&args.file_name, args.file_write_delay)
                {
                    Ok(_) => (),
                    Err(e) => return Err::<(), _>(e),
                }
            }
        });
    }

    loop {
        match consumer.recv().await {
            Err(e) => log::warn!("Kafka error: {}", e),
            Ok(msg) => {
                let mut nexus = nexus.lock().await;
                log::debug!(
                    "key: '{:?}', topic: {}, partition: {}, offset: {}, timestamp: {:?}",
                    msg.key(),
                    msg.topic(),
                    msg.partition(),
                    msg.offset(),
                    msg.timestamp()
                );

                if let Some(payload) = msg.payload() {
                    if args
                        .event_topic
                        .as_deref()
                        .map(|topic| msg.topic() == topic)
                        .unwrap_or(false)
                        //  This statement should certainly be simplified
                    {
                        match root_as_digitizer_event_list_message(payload) {
                            Ok(data) => {
                                metrics::MESSAGES_RECEIVED
                                    .get_or_create(&metrics::MessagesReceivedLabels::new(
                                        metrics::MessageKind::Event,
                                    ))
                                    .inc();
                                if let Err(e) = nexus.process_message(&data) {
                                    log::warn!("Failed to save event list to file: {}", e);
                                    metrics::FAILURES
                                        .get_or_create(&metrics::FailureLabels::new(
                                            metrics::FailureKind::FileWriteFailed,
                                        ))
                                        .inc();
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
                    } else if args.control_topic == msg.topic() {
                        if run_start_buffer_has_identifier(payload) {
                            match root_as_run_start(payload) {
                                Ok(data) => {
                                    metrics::MESSAGES_RECEIVED
                                        .get_or_create(&metrics::MessagesReceivedLabels::new(
                                            metrics::MessageKind::Unknown,
                                        ))
                                        .inc();
                                    nexus
                                        .start_command(data)
                                        .expect("RunStart command is valid");
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
                        } else if run_stop_buffer_has_identifier(payload) {
                            match root_as_run_stop(payload) {
                                Ok(data) => {
                                    metrics::MESSAGES_RECEIVED
                                        .get_or_create(&metrics::MessagesReceivedLabels::new(
                                            metrics::MessageKind::Unknown,
                                        ))
                                        .inc();
                                    nexus.stop_command(data).expect("RunStop command is valid");
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
                        }
                    } else {
                        log::warn!("Unexpected message type on topic \"{}\"", msg.topic());
                        metrics::MESSAGES_RECEIVED
                            .get_or_create(&metrics::MessagesReceivedLabels::new(
                                metrics::MessageKind::Unknown,
                            ))
                            .inc();
                    }
                }
                consumer.commit_message(&msg, CommitMode::Async).unwrap();
            }
        };
    }
}

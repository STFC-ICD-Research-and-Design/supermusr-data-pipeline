mod parameters;
mod processing;
mod pulse_detection;

use chrono as _;
use clap::Parser;
use parameters::{Mode, Polarity};
use rdkafka::{
    consumer::{stream_consumer::StreamConsumer, CommitMode, Consumer},
    message::{BorrowedMessage, Header, Message, OwnedHeaders},
    producer::{FutureProducer, FutureRecord},
};
use supermusr_common::Intensity;
use std::{net::SocketAddr, time::{Duration, Instant}};
use std::path::PathBuf;
use supermusr_streaming_types::{
    dat1_digitizer_analog_trace_v1_generated::{
        digitizer_analog_trace_message_buffer_has_identifier,
        root_as_digitizer_analog_trace_message,
    },
    flatbuffers::FlatBufferBuilder,
};
use tracing::{debug, warn, error};
use tracing_subscriber as _;

use crate::parameters::DetectorSettings;
// cargo run --release --bin trace-to-events -- --broker localhost:19092 --trace-topic Traces --event-topic Events --group trace-to-events constant-phase-discriminator --threshold-trigger=-40,1,0
// cargo run --release --bin trace-to-events -- --broker localhost:19092 --trace-topic Traces --event-topic Events --group trace-to-events advanced-muon-detector --muon-onset=0.1 --muon-fall=0.1 --muon-termination=0.1 --duration=1
// RUST_LOG=off cargo run --release --bin trace-to-events -- --broker localhost:19092 --trace-topic Traces --event-topic Events --group trace-to-events advanced-muon-detector --muon-onset=0.1 --muon-fall=0.1 --muon-termination=0.1 --duration=1

// cargo run --release --bin trace-reader -- --broker localhost:19092 --consumer-group trace-producer --trace-topic Traces --file-name ../Data/Traces/MuSR_A41_B42_C43_D44_Apr2021_Ag_ZF_IntDeg_Slit60_short.traces --number-of-trace-events 500 --channel-multiplier 4 --message-multiplier 1

/*
RUST_LOG=off cargo run --release --bin simulator -- --broker localhost:19092 --trace-topic Traces --num-channels 16 --time-bins 30000 continuous --frame-time 1
*/

/* Optimizations:
    Moving the fbb object out of the processing function and taking the slice rather than copying
    Streamline the process for writing channel event data to the message channel list
    Scoped multithreading to process channels simultaneously
    Change kafka property linger.ms to 0 (why does this help?)
    ^^^ Implementing async message producing with linger.ms at 100 or other
    Dispensed with pulse assembler in the case of constant phase discriminator (no apparent affect)

    Fixes:
    sampletime doesn't do anything in find_channel_events
    trace-reader: line 83 num_trace_events to total_trace_events

    Possible Optimizations:
    Employ multithreading for message passing.
*/
/*
            |  Constant  | Advanced
16 Channels | 1.5ms(0.5) | 12ms(3.0)
 8 Channels | 2.3ms(0.4) | 6ms (2.1)
 4 Channels | 1.2ms(0.2) | 3ms (1.6)
 stddev, min, max
 compression
 GPU
*/

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

    #[clap(long)]
    polarity: Polarity,

    #[clap(long, default_value = "0")]
    baseline: Intensity,
    
    #[clap(long, env, default_value = "127.0.0.1:9090")]
    observability_address: SocketAddr,

    #[clap(long)]
    save_file: Option<PathBuf>,

    #[command(subcommand)]
    pub(crate) mode: Mode,
}

#[tokio::main]
async fn main() {

    let args = Cli::parse();

    //tracing_subscriber::fmt().init();

    let mut client_config = supermusr_common::generate_kafka_client_config(
        &args.broker,
        &args.username,
        &args.password,
    );

    let producer: FutureProducer = client_config
        .set("linger.ms", "0")
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
                    if digitizer_analog_trace_message_buffer_has_identifier(payload) {
                        match root_as_digitizer_analog_trace_message(payload) {
                            Ok(thing) => {
                                let mut fbb = FlatBufferBuilder::new();
                                let time = Instant::now();
                                processing::process(
                                    &mut fbb,
                                    &thing,
                                    &DetectorSettings { polarity: &args.polarity, baseline: args.baseline, mode: &args.mode },
                                    args.save_file.as_deref(),
                                );
                                
                                let headers = append_headers(&m, time.elapsed(), payload.len(), fbb.finished_data().len());

                                let future = producer
                                    .send_result(
                                        FutureRecord::to(&args.event_topic)
                                            .payload(fbb.finished_data())
                                            .headers(headers)
                                            .key("test"),
                                    )
                                    .expect("Producer sends");

                                tokio::spawn(async {
                                    match future.await {
                                        Ok(_) => {
                                            debug!("Published event message");
                                        }
                                        Err(e) => {
                                            error!("{:?}", e);
                                        }
                                    }
                                });
                                fbb.reset();
                            }
                            Err(e) => {
                                warn!("Failed to parse message: {}", e);
                            }
                        }
                    } else {
                        warn!("Unexpected message type on topic \"{}\"", m.topic());
                    }
                }
                consumer.commit_message(&m, CommitMode::Async).unwrap();
            }
            Err(e) => {
                warn!("Kafka error: {}", e);
            }
        }
    }
}

fn append_headers(m : &BorrowedMessage, time : Duration, bytes_in : usize, bytes_out: usize) -> OwnedHeaders {
    m.headers()
    .map(|h| h.detach())
    .unwrap_or_default()
    .insert(Header {
        key: "trace-to-events: time_ns",
        value: Some(&time.as_nanos().to_string()),
    })
    .insert(Header {
        key: "trace-to-events: size of trace",
        value: Some(&bytes_in.to_string()),
    })
    .insert(Header {
        key: "trace-to-events: size of events list",
        value: Some(&bytes_out.to_string()),
    })
}
use chrono::Utc;
use clap::{Parser, Subcommand};
use rdkafka::{
    producer::{FutureProducer, FutureRecord},
    util::Timeout,
};
use std::time::{Duration, SystemTime};
use supermusr_common::{Channel, Intensity, Time};
use supermusr_streaming_types::{
    dat1_digitizer_analog_trace_v1_generated::{
        finish_digitizer_analog_trace_message_buffer, ChannelTrace, ChannelTraceArgs,
        DigitizerAnalogTraceMessage, DigitizerAnalogTraceMessageArgs,
    },
    dev1_digitizer_event_v1_generated::{
        finish_digitizer_event_list_message_buffer, DigitizerEventListMessage,
        DigitizerEventListMessageArgs,
    },
    flatbuffers::FlatBufferBuilder,
    frame_metadata_v1_generated::{FrameMetadataV1, FrameMetadataV1Args, GpsTime},
};
use tokio::time;

#[derive(Clone, Parser)]
#[clap(author, version, about)]
struct Cli {
    /// Kafka broker address
    #[clap(long = "broker")]
    broker_address: String,

    /// Kafka username
    #[clap(long)]
    username: Option<String>,

    /// Kafka password
    #[clap(long)]
    password: Option<String>,

    /// Topic to publish event packets to
    #[clap(long)]
    event_topic: Option<String>,

    /// Topic to publish analog trace packets to
    #[clap(long)]
    trace_topic: Option<String>,

    /// Digitizer identifier to use
    #[clap(long = "did", default_value = "0")]
    digitizer_id: u8,

    /// Number of events to include in each frame
    #[clap(long = "events", default_value = "500")]
    events_per_frame: usize,

    /// Number of measurements to include in each frame
    #[clap(long = "time-bins", default_value = "500")]
    measurements_per_frame: usize,

    #[command(subcommand)]
    mode: Mode,
}

#[derive(Clone, Subcommand)]
enum Mode {
    /// Run in single shot mode, output a single frame then exit
    Single(Single),

    /// Run in continuous mode, outputting one frame every `frame-time` milliseconds
    Continuous(Continuous),
}

#[derive(Clone, Parser)]
struct Single {
    /// Number of frame to be sent
    #[clap(long = "frame", default_value = "0")]
    frame_number: u32,
}

#[derive(Clone, Parser)]
struct Continuous {
    /// Number of first frame to be sent
    #[clap(long = "start-frame", default_value = "0")]
    start_frame_number: u32,

    /// Time in milliseconds between each frame
    #[clap(long, default_value = "20")]
    frame_time: u64,
}

#[tokio::main]
async fn main() {
    env_logger::init();

    let cli = Cli::parse();

    let client_config = supermusr_common::generate_kafka_client_config(
        &cli.broker_address,
        &cli.username,
        &cli.password,
    );
    let producer = client_config.create().unwrap();

    let mut fbb = FlatBufferBuilder::new();

    match cli.mode.clone() {
        Mode::Single(m) => {
            send(
                &producer,
                cli.clone(),
                &mut fbb,
                m.frame_number,
                Duration::default(),
            )
            .await;
        }
        Mode::Continuous(m) => {
            let mut frame = time::interval(Duration::from_millis(m.frame_time));

            let start_time = SystemTime::now();
            let mut frame_number = m.start_frame_number;

            loop {
                let now = SystemTime::now().duration_since(start_time).unwrap();
                send(&producer, cli.clone(), &mut fbb, frame_number, now).await;

                frame_number += 1;
                frame.tick().await;
            }
        }
    }
}

async fn send(
    producer: &FutureProducer,
    cli: Cli,
    fbb: &mut FlatBufferBuilder<'_>,
    frame_number: u32,
    now: Duration,
) {
    let time: GpsTime = Utc::now().into();

    if let Some(topic) = &cli.event_topic {
        let start_time = SystemTime::now();
        fbb.reset();

        let metadata = FrameMetadataV1Args {
            frame_number,
            period_number: 0,
            protons_per_pulse: 0,
            running: true,
            timestamp: Some(&time),
            veto_flags: 0,
        };
        let metadata = FrameMetadataV1::create(fbb, &metadata);

        let message = DigitizerEventListMessageArgs {
            digitizer_id: cli.digitizer_id,
            metadata: Some(metadata),
            channel: Some(fbb.create_vector::<Channel>(&vec![1; cli.events_per_frame])),
            voltage: Some(fbb.create_vector::<Intensity>(&vec![2; cli.events_per_frame])),
            time: Some(fbb.create_vector::<Time>(&vec![
                u32::try_from(now.as_millis()).unwrap();
                cli.events_per_frame
            ])),
        };
        let message = DigitizerEventListMessage::create(fbb, &message);
        finish_digitizer_event_list_message_buffer(fbb, message);

        match producer
            .send(
                FutureRecord::to(topic)
                    .payload(fbb.finished_data())
                    .key(&"todo".to_string()),
                Timeout::After(Duration::from_millis(100)),
            )
            .await
        {
            Ok(r) => log::debug!("Delivery: {:?}", r),
            Err(e) => log::error!("Delivery failed: {:?}", e),
        };

        log::info!(
            "Event send took: {:?}",
            SystemTime::now().duration_since(start_time).unwrap()
        );
    }

    if let Some(topic) = &cli.trace_topic {
        let start_time = SystemTime::now();
        fbb.reset();

        let metadata = FrameMetadataV1Args {
            frame_number,
            period_number: 0,
            protons_per_pulse: 0,
            running: true,
            timestamp: Some(&time),
            veto_flags: 0,
        };
        let metadata = FrameMetadataV1::create(fbb, &metadata);

        let mut channel0_voltage = Vec::<Intensity>::new();
        channel0_voltage.resize(cli.measurements_per_frame, 404);
        channel0_voltage[0] = frame_number as Intensity;
        channel0_voltage[1] = cli.digitizer_id as Intensity;
        let channel0_voltage = Some(fbb.create_vector::<Intensity>(&channel0_voltage));
        let channel0 = ChannelTrace::create(
            fbb,
            &ChannelTraceArgs {
                channel: 0,
                voltage: channel0_voltage,
            },
        );

        let message = DigitizerAnalogTraceMessageArgs {
            digitizer_id: cli.digitizer_id,
            metadata: Some(metadata),
            sample_rate: 1_000_000_000,
            channels: Some(fbb.create_vector(&[channel0])),
        };
        let message = DigitizerAnalogTraceMessage::create(fbb, &message);
        finish_digitizer_analog_trace_message_buffer(fbb, message);

        match producer
            .send(
                FutureRecord::to(topic)
                    .payload(fbb.finished_data())
                    .key(&"todo".to_string()),
                Timeout::After(Duration::from_millis(100)),
            )
            .await
        {
            Ok(r) => log::debug!("Delivery: {:?}", r),
            Err(e) => log::error!("Delivery failed: {:?}", e),
        };

        log::info!(
            "Trace send took: {:?}",
            SystemTime::now().duration_since(start_time).unwrap()
        );
    }
}

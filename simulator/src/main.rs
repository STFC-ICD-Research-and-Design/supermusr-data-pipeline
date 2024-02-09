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

        let channel0_voltage = gen_dummy_trace_data(&cli, frame_number, 0);
        let channel0_voltage = fbb.create_vector::<Intensity>(&channel0_voltage);
        let channel0 = ChannelTrace::create(
            fbb,
            &ChannelTraceArgs {
                channel: 0,
                voltage: Some(channel0_voltage),
            },
        );

        let channel1_voltage = gen_dummy_trace_data(&cli, frame_number, 1);
        let channel1_voltage = fbb.create_vector::<Intensity>(&channel1_voltage);
        let channel1 = ChannelTrace::create(
            fbb,
            &ChannelTraceArgs {
                channel: 1,
                voltage: Some(channel1_voltage),
            },
        );

        let channel2_voltage = gen_dummy_trace_data(&cli, frame_number, 2);
        let channel2_voltage = fbb.create_vector::<Intensity>(&channel2_voltage);
        let channel2 = ChannelTrace::create(
            fbb,
            &ChannelTraceArgs {
                channel: 2,
                voltage: Some(channel2_voltage),
            },
        );

        let channel3_voltage = gen_dummy_trace_data(&cli, frame_number, 3);
        let channel3_voltage = fbb.create_vector::<Intensity>(&channel3_voltage);
        let channel3 = ChannelTrace::create(
            fbb,
            &ChannelTraceArgs {
                channel: 3,
                voltage: Some(channel3_voltage),
            },
        );

        let channel4_voltage = gen_dummy_trace_data(&cli, frame_number, 4);
        let channel4_voltage = fbb.create_vector::<Intensity>(&channel4_voltage);
        let channel4 = ChannelTrace::create(
            fbb,
            &ChannelTraceArgs {
                channel: 4,
                voltage: Some(channel4_voltage),
            },
        );

        let channel5_voltage = gen_dummy_trace_data(&cli, frame_number, 5);
        let channel5_voltage = fbb.create_vector::<Intensity>(&channel5_voltage);
        let channel5 = ChannelTrace::create(
            fbb,
            &ChannelTraceArgs {
                channel: 5,
                voltage: Some(channel5_voltage),
            },
        );

        let channel6_voltage = gen_dummy_trace_data(&cli, frame_number, 6);
        let channel6_voltage = fbb.create_vector::<Intensity>(&channel6_voltage);
        let channel6 = ChannelTrace::create(
            fbb,
            &ChannelTraceArgs {
                channel: 6,
                voltage: Some(channel6_voltage),
            },
        );

        let channel7_voltage = gen_dummy_trace_data(&cli, frame_number, 7);
        let channel7_voltage = fbb.create_vector::<Intensity>(&channel7_voltage);
        let channel7 = ChannelTrace::create(
            fbb,
            &ChannelTraceArgs {
                channel: 7,
                voltage: Some(channel7_voltage),
            },
        );

        let message = DigitizerAnalogTraceMessageArgs {
            digitizer_id: cli.digitizer_id,
            metadata: Some(metadata),
            sample_rate: 1_000_000_000,
            channels: Some(fbb.create_vector(&[
                channel0, channel1, channel2, channel3, channel4, channel5, channel6, channel7,
            ])),
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

fn gen_dummy_trace_data(cli: &Cli, frame_number: u32, channel_number: u32) -> Vec<Intensity> {
    let mut intensity = vec![404; cli.measurements_per_frame];
    intensity[0] = frame_number as Intensity;
    intensity[1] = cli.digitizer_id as Intensity;
    intensity[2] = channel_number as Intensity;
    intensity
}

use anyhow::Result;
use chrono::Utc;
use clap::Parser;
use flatbuffers::FlatBufferBuilder;
use rdkafka::{
    config::ClientConfig,
    producer::{FutureProducer, FutureRecord},
    util::Timeout,
};
use std::time::{Duration, SystemTime};
use streaming_types::{
    dat1_digitizer_analog_trace_v1_generated::{
        finish_digitizer_analog_trace_message_buffer, ChannelTrace, ChannelTraceArgs,
        DigitizerAnalogTraceMessage, DigitizerAnalogTraceMessageArgs,
    },
    dev1_digitizer_event_v1_generated::{
        finish_digitizer_event_list_message_buffer, DigitizerEventListMessage,
        DigitizerEventListMessageArgs,
    },
    frame_metadata_v1_generated::{FrameMetadataV1, FrameMetadataV1Args, GpsTime},
};
use tokio::time;

#[derive(Debug, Parser)]
#[clap(author, version, about)]
struct Cli {
    #[clap(long = "broker")]
    broker_address: String,

    #[clap(long)]
    username: String,

    #[clap(long)]
    password: String,

    /// Time in milliseconds between each frame
    #[clap(long, default_value = "20")]
    frame_time: u64,

    /// Topic to publish event packets to
    #[clap(long)]
    event_topic: Option<String>,

    /// Topic to publish analog trace packets to
    #[clap(long)]
    trace_topic: Option<String>,

    /// Digitizer identifier to use
    #[clap(long = "did", default_value = "0")]
    digitizer_id: u8,

    /// Number of first frame to be sent
    #[clap(long = "start-frame", default_value = "0")]
    start_frame_number: u32,

    /// Number of events to include in each frame
    #[clap(long = "events", default_value = "500")]
    events_per_frame: usize,

    /// Number of time bins to include in each frame
    #[clap(long = "time-bins", default_value = "500")]
    time_bins_per_frame: usize,
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let cli = Cli::parse();
    log::debug!("Args: {:?}", cli);

    let producer: FutureProducer = ClientConfig::new()
        .set("bootstrap.servers", &cli.broker_address)
        .set("security.protocol", "sasl_plaintext")
        .set("sasl.mechanisms", "SCRAM-SHA-256")
        .set("sasl.username", &cli.username)
        .set("sasl.password", &cli.password)
        .create()?;

    let mut frame = time::interval(Duration::from_millis(cli.frame_time));

    let mut fbb = FlatBufferBuilder::new();

    let start_time = SystemTime::now();
    let mut frame_number = cli.start_frame_number;

    loop {
        let now = SystemTime::now().duration_since(start_time)?;

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
            let metadata = FrameMetadataV1::create(&mut fbb, &metadata);

            let message = DigitizerEventListMessageArgs {
                digitizer_id: cli.digitizer_id,
                metadata: Some(metadata),
                channel: Some(fbb.create_vector::<u32>(&vec![1; cli.events_per_frame])),
                voltage: Some(fbb.create_vector::<u16>(&vec![2; cli.events_per_frame])),
                time: Some(fbb.create_vector::<u32>(&vec![
                    u32::try_from(now.as_millis())?;
                    cli.events_per_frame
                ])),
            };
            let message = DigitizerEventListMessage::create(&mut fbb, &message);
            finish_digitizer_event_list_message_buffer(&mut fbb, message);

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
                SystemTime::now().duration_since(start_time)?
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
            let metadata = FrameMetadataV1::create(&mut fbb, &metadata);

            let channel0_voltage: Vec<u16> =
                (0..cli.time_bins_per_frame).map(|i| i as u16).collect();
            let channel0_voltage = Some(fbb.create_vector::<u16>(&channel0_voltage));
            let channel0 = ChannelTrace::create(
                &mut fbb,
                &ChannelTraceArgs {
                    channel: 0,
                    voltage: channel0_voltage,
                },
            );

            let message = DigitizerAnalogTraceMessageArgs {
                digitizer_id: cli.digitizer_id,
                metadata: Some(metadata),
                sample_rate: 0,
                channels: Some(fbb.create_vector(&[channel0])),
            };
            let message = DigitizerAnalogTraceMessage::create(&mut fbb, &message);
            finish_digitizer_analog_trace_message_buffer(&mut fbb, message);

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
                SystemTime::now().duration_since(start_time)?
            );
        }

        frame_number += 1;

        frame.tick().await;
    }
}

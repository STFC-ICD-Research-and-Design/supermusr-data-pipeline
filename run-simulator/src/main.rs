use chrono::{DateTime, Utc};
use clap::{Parser, Subcommand};
use rdkafka::{
    producer::{FutureProducer, FutureRecord},
    util::Timeout,
};
use std::time::{Duration, SystemTime, Instant};
use supermusr_common::{Channel, Intensity, Time};
use supermusr_streaming_types::{
    ecs_6s4t_run_stop_generated::{RunStopArgs, RunStop, finish_run_stop_buffer}, ecs_df12_det_spec_map_generated::{SpectraDetectorMappingArgs, SpectraDetectorMapping, finish_spectra_detector_mapping_buffer}, ecs_pl72_run_start_generated::{finish_run_start_buffer, RunStart, RunStartArgs}, flatbuffers::FlatBufferBuilder
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

    /// Topic to publish command to
    #[clap(long)]
    topic: String,

    /// Unique name of the run
    #[clap(long)]
    run_name: u8,

    /// Timestamp of the message.
    #[clap(long, default_value = "Utc::now()")]
    time: DateTime<Utc>,

    #[command(subcommand)]
    mode: Mode,
}

#[derive(Clone, Subcommand)]
enum Mode {
    /// Run in single shot mode, output a single frame then exit
    RunStart(Status),

    /// Run in continuous mode, outputting one frame every `frame-time` milliseconds
    RunStop,
}

#[derive(Clone, Parser)]
struct Status {
    /// Number of frame to be sent
    #[clap(long)]
    instrument_name: String,

    /// If set a corresponding stop message is automatically sent
    #[clap(long)]
    stop_time: Option<DateTime<Utc>>,
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

    let pending_messages : Vec<(Instant, Vec<u8>)>;

    match cli.mode.clone() {
        Mode::RunStart(status) => {
            send(
                &producer,
                cli.clone(),
                &mut fbb,
                m.frame_number,
                Duration::default(),
            )
            .await;
        }
        Mode::RunStop => {
            RunStopArgs { stop_time: todo!(), run_name: todo!(), job_id: todo!(), service_id: todo!(), command_id: todo!() }
        }
    }
    while let Some((time,bytes)) = pending_messages.last() {
        if time.into() >= &Utc::now() {
            match producer
                .send(
                    FutureRecord::to(&cli.topic)
                        .payload(bytes)
                        .key(&"todo".to_string()),
                    Timeout::After(Duration::from_millis(100)),
                )
                .await
            {
                Ok(r) => log::debug!("Delivery: {:?}", r),
                Err(e) => log::error!("Delivery failed: {:?}", e),
            };

            log::info!("Run command send");
            pending_messages.pop();
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

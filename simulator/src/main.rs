mod integrated;
pub(crate) mod runs;

use chrono::Utc;
use clap::{Parser, Subcommand};
use integrated::run_configured_simulation;
use rdkafka::{
    producer::{FutureProducer, FutureRecord},
    util::Timeout,
};
use runs::{
    create_messages::{
        create_alarm_command, create_run_start_command, create_run_stop_command,
        create_runlog_command, create_sample_environment_command,
    },
    AlarmData, RunLogData, SampleEnvData, Start, Stop,
};
use std::{
    path::PathBuf,
    time::{Duration, SystemTime},
};
use supermusr_common::{
    init_tracer,
    tracer::{FutureRecordTracerExt, TracerEngine, TracerOptions},
    Channel, CommonKafkaOpts, Intensity, Time,
};
use supermusr_streaming_types::{
    dat2_digitizer_analog_trace_v2_generated::{
        finish_digitizer_analog_trace_message_buffer, ChannelTrace, ChannelTraceArgs,
        DigitizerAnalogTraceMessage, DigitizerAnalogTraceMessageArgs,
    },
    dev2_digitizer_event_v2_generated::{
        finish_digitizer_event_list_message_buffer, DigitizerEventListMessage,
        DigitizerEventListMessageArgs,
    },
    flatbuffers::FlatBufferBuilder,
    frame_metadata_v2_generated::{FrameMetadataV2, FrameMetadataV2Args, GpsTime},
};
use tokio::time;
use tracing::{debug, error, info, level_filters::LevelFilter, warn};

#[derive(Clone, Parser)]
#[clap(author, version, about)]
struct Cli {
    /// Kafka options common to all tools.
    #[clap(flatten)]
    common_kafka_options: CommonKafkaOpts,

    /// Topic to publish run start and run stop messages to
    #[clap(long)]
    control_topic: Option<String>,

    /// Topic to publish digitiser event packets to
    #[clap(long)]
    event_topic: Option<String>,

    /// Topic to publish frame assembled event packets to (only functional in Defined mode for TraceMessages with source-type = AggregatedFrame)
    #[clap(long)]
    frame_event_topic: Option<String>,

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

    /// If set, then OpenTelemetry data is sent to the URL specified, otherwise the standard tracing subscriber is used
    #[clap(long)]
    otel_endpoint: Option<String>,

    /// The reporting level to use for OpenTelemetry
    #[clap(long, default_value = "info")]
    otel_level: LevelFilter,

    #[command(subcommand)]
    mode: Mode,
}

#[derive(Clone, Subcommand)]
enum Mode {
    /// Run in single shot mode, output a single frame then exit
    Single(Single),

    /// Run in continuous mode, outputting one frame every `frame-time` milliseconds
    Continuous(Continuous),

    /// Run in json mode, behaviour is defined by the file given by --file
    Defined(Defined),

    /// Send a single RunStart command
    Start(Start),

    /// Send a single RunStop command
    Stop(Stop),

    /// Send a single RunStop command
    Log(RunLogData),

    /// Send a single SampleEnv command
    SampleEnv(SampleEnvData),

    /// Send a single Alarm command
    Alarm(AlarmData),
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

#[derive(Clone, Parser)]
struct Defined {
    /// Path to the json settings file
    file: PathBuf,

    /// Topic to publish run log data messages to
    #[clap(long)]
    runlog_topic: String,

    /// Topic to publish sample environment log messages to
    #[clap(long)]
    selog_topic: String,

    /// Topic to publish alarm messages to
    #[clap(long)]
    alarm_topic: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let tracer = init_tracer!(TracerOptions::new(
        cli.otel_endpoint.as_deref(),
        cli.otel_level
    ));

    let kafka_opts = &cli.common_kafka_options;

    let client_config = supermusr_common::generate_kafka_client_config(
        &kafka_opts.broker,
        &kafka_opts.username,
        &kafka_opts.password,
    );
    let producer = client_config.create().unwrap();

    match cli.mode.clone() {
        Mode::Single(single) => {
            run_single_simulation(&cli, &producer, single).await;
            Ok(())
        }
        Mode::Continuous(continuous) => {
            run_continuous_simulation(&cli, &producer, continuous).await;
            Ok(())
        }
        Mode::Defined(defined) => {
            Ok(run_configured_simulation(tracer.use_otel(), &cli, &producer, defined).await?)
        }
        Mode::Start(start) => {
            Ok(create_run_start_command(tracer.use_otel(), &start, &producer).await?)
        }
        Mode::Stop(stop) => Ok(create_run_stop_command(tracer.use_otel(), &stop, &producer).await?),
        Mode::Log(log) => Ok(create_runlog_command(tracer.use_otel(), &log, &producer).await?),
        Mode::SampleEnv(sample_env) => {
            Ok(
                create_sample_environment_command(tracer.use_otel(), &sample_env, &producer)
                    .await?,
            )
        }
        Mode::Alarm(alarm) => Ok(create_alarm_command(tracer.use_otel(), &alarm, &producer).await?),
    }
}

async fn run_single_simulation(cli: &Cli, producer: &FutureProducer, single: Single) {
    let mut fbb = FlatBufferBuilder::new();
    send(
        producer,
        cli.clone(),
        &mut fbb,
        single.frame_number,
        Duration::default(),
    )
    .await;
}

async fn run_continuous_simulation(cli: &Cli, producer: &FutureProducer, continuous: Continuous) {
    let mut fbb = FlatBufferBuilder::new();
    let mut frame = time::interval(Duration::from_millis(continuous.frame_time));

    let start_time = SystemTime::now();
    let mut frame_number = continuous.start_frame_number;

    loop {
        let now = SystemTime::now().duration_since(start_time).unwrap();
        send(producer, cli.clone(), &mut fbb, frame_number, now).await;

        frame_number += 1;
        frame.tick().await;
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

        let metadata = FrameMetadataV2Args {
            frame_number,
            period_number: 0,
            protons_per_pulse: 0,
            running: true,
            timestamp: Some(&time),
            veto_flags: 0,
        };
        let metadata = FrameMetadataV2::create(fbb, &metadata);

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

        let future_record = FutureRecord::to(topic)
            .payload(fbb.finished_data())
            .conditional_inject_current_span_into_headers(cli.otel_endpoint.is_some())
            .key("Simulated Event");

        let timeout = Timeout::After(Duration::from_millis(100));
        match producer.send(future_record, timeout).await {
            Ok(r) => debug!("Delivery: {:?}", r),
            Err(e) => error!("Delivery failed: {:?}", e),
        };

        info!(
            "Event send took: {:?}",
            SystemTime::now().duration_since(start_time).unwrap()
        );
    }

    if let Some(topic) = &cli.trace_topic {
        let start_time = SystemTime::now();
        fbb.reset();

        let metadata = FrameMetadataV2Args {
            frame_number,
            period_number: 0,
            protons_per_pulse: 0,
            running: true,
            timestamp: Some(&time),
            veto_flags: 0,
        };
        let metadata = FrameMetadataV2::create(fbb, &metadata);

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

        let future_record = FutureRecord::to(topic)
            .payload(fbb.finished_data())
            .conditional_inject_current_span_into_headers(cli.otel_endpoint.is_some())
            .key("Simulated Trace");

        let timeout = Timeout::After(Duration::from_millis(100));
        match producer.send(future_record, timeout).await {
            Ok(r) => debug!("Delivery: {:?}", r),
            Err(e) => error!("Delivery failed: {:?}", e.0),
        };

        info!(
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

mod nexus;
mod metrics;

use nexus::Nexus;
use anyhow::{anyhow, Result};
use chrono as _;
use ndarray as _;
use ndarray_stats as _;
use clap::Parser;
use kagiyama::{prometheus::metrics::info::Info, AlwaysReady, Watcher};
use rdkafka::{
    consumer::{stream_consumer::StreamConsumer, CommitMode, Consumer},
    message::Message,
};
use std::{net::SocketAddr, path::PathBuf};
use supermusr_streaming_types::{
    aev1_frame_assembled_event_v1_generated::{
        frame_assembled_event_list_message_buffer_has_identifier,
        root_as_frame_assembled_event_list_message,
    },
    dat1_digitizer_analog_trace_v1_generated::{
        digitizer_analog_trace_message_buffer_has_identifier,
        root_as_digitizer_analog_trace_message,
    },
};

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
    event_topic: Option<String>,

    #[clap(long)]
    trace_topic: Option<String>,

    #[clap(long)]
    histogram_topic: Option<String>,

    #[clap(long)]
    file: PathBuf,

    #[clap(long)]
    digitizer_count: Option<usize>,

    #[clap(long, default_value = "127.0.0.1:9090")]
    observability_address: SocketAddr,
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let args = Cli::parse();
    log::debug!("Args: {:?}", args);

    let mut watcher = Watcher::<AlwaysReady>::default();
    metrics::register(&mut watcher);
    {/*
        let output_files = Info::new(vec![
            (
                "event".to_string(),
                match args.event_file {
                    Some(ref f) => f.display().to_string(),
                    None => "none".into(),
                },
            ),
            (
                "trace".to_string(),
                match args.trace_file {
                    Some(ref f) => f.display().to_string(),
                    None => "none".into(),
                },
            ),
        ]);

        let mut registry = watcher.metrics_registry();
        registry.register("output_files", "Configured output filenames", output_files);
         */
    }
    watcher.start_server(args.observability_address).await;

    Nexus::create_file()?.write("output.nx")?;

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

    let topics_to_subscribe: Vec<&str> = vec![args.event_topic.as_deref(), args.trace_topic.as_deref(), args.histogram_topic.as_deref()]
        .into_iter()
        .flatten()
        .collect();
    if topics_to_subscribe.is_empty() {
        return Err(anyhow!(
            "Nothing to do (no message type requested to be saved)"
        ));
    }
    consumer.subscribe(&topics_to_subscribe)?;
    let mut file = Some(Nexus::create_file()?);
    /*let mut event_file = match args.event_file {
        Some(filename) => Some(EventFile::create(&filename)?),
        None => None,
    };

    let mut trace_file = match args.trace_file {
        Some(filename) => Some(TraceFile::create(
            &filename,
            args.digitizer_count
                .expect("digitizer count should be provided"),
        )?),
        None => None,
    };*/

    loop {
        match consumer.recv().await {
            Err(e) => log::warn!("Kafka error: {}", e),
            Ok(msg) => {
                log::debug!(
                    "key: '{:?}', topic: {}, partition: {}, offset: {}, timestamp: {:?}",
                    msg.key(),
                    msg.topic(),
                    msg.partition(),
                    msg.offset(),
                    msg.timestamp()
                );

                if let Some(payload) = msg.payload() {
                    if file.is_some() {
                        if args.trace_topic.as_deref().map(|topic| msg.topic() == topic).unwrap_or(false) {
                            match root_as_digitizer_analog_trace_message(payload) {
                                Ok(data) => {
                                    log::info!(
                                        "Trace packet: dig. ID: {}, metadata: {:?}",
                                        data.digitizer_id(),
                                        data.metadata()
                                    );
                                    metrics::MESSAGES_RECEIVED
                                        .get_or_create(&metrics::MessagesReceivedLabels::new(
                                            metrics::MessageKind::Trace,
                                        ))
                                        .inc();
                                    if let Err(e) = file.as_mut().unwrap().push_trace(&data) {
                                        log::warn!("Failed to save traces to file: {}", e);
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
                        } else if args.event_topic.as_deref().map(|topic| msg.topic() == topic).unwrap_or(false) {
                            match root_as_frame_assembled_event_list_message(payload) {
                                Ok(data) => {
                                    log::info!("Event packet: metadata: {:?}", data.metadata());
                                    metrics::MESSAGES_RECEIVED
                                        .get_or_create(&metrics::MessagesReceivedLabels::new(
                                            metrics::MessageKind::Event,
                                        ))
                                        .inc();
                                    if let Err(e) = file.as_mut().unwrap().push_event(&data) {
                                        log::warn!("Failed to save events to file: {}", e);
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
                        } else if args.histogram_topic.as_deref().map(|topic| msg.topic() == topic).unwrap_or(false) {
                            // todo
                        } else {
                            // todo
                        }
                    } else {
                        if args.trace_topic.as_deref().map(|topic| msg.topic() == topic).unwrap_or(false) {
                            // todo
                        } else {
                            log::warn!("Unexpected message type on topic \"{}\"", msg.topic());
                            metrics::MESSAGES_RECEIVED
                                .get_or_create(&metrics::MessagesReceivedLabels::new(
                                    metrics::MessageKind::Unknown,
                                ))
                                .inc();
                        }
                    }
                }

                consumer.commit_message(&msg, CommitMode::Async).unwrap();
            }
        };
    }
}

/*
#[derive(H5Type)]
#[repr(transparent)]
struct NXroot {
    file_name : String,
    file_time : String,
    initial_file_format : Option<String>,
    file_update_time : Option<String>,
    nexus_version : Option<String>,
    hdf_version : Option<String>,
    hdf5_version : Option<String>,
    xml_version : Option<String>,
    creator : Option<String>,
    entry : Vec<NXentry>,
}
impl NXroot {
    fn create(file : File) -> Result<()> {
        let root = file.create_group("NXroot")?;
        root.new_dataset::<u8>().shape(Extents::Simple());
        Ok(())
    }
}

#[derive(H5Type)]
struct NXentry {
    idf_version : i32,
    beamline : Option<String>,
    definition : String,
    definition_local : Option<String>,
    program_name : Option<String>,
    run_number : i32,
    title : String,
    notes : Option<String>,
    start_time : String,
    end_time : String,
    duration : Option<i32>,
    collection_time : Option<f32>,
    total_counts : Option<f32>,
    good_frames : Option<f32>,
    raw_frames : Option<f32>,
    proton_charge : Option<f32>,
    experiment_identifier : String,
    run_cycle : Option<String>,
    user_1 : NXuser,
    experiment_team : Vec<NXuser>,
    runlog : Option<NXrunlog>,
    selog : Option<NXselog>,
    periods : Option<NXperiod>,
    sample : Option<NXsample>,
    instrument : NXinstrument,
    data : Vec<NXdata>,
    characterization : Vec<NXcharacterization>,
    uif : Option<NXuif>,
}

#[derive(H5Type)]
struct NXuser{
    name : String,
}

#[derive(H5Type)]
#[repr(C)]
struct NXrunlog{
    name : Vec<u8>,
}
#[derive(H5Type)]
struct NXselog{
    name : String,
}
#[derive(H5Type)]
struct NXperiod{
    name : String,
}
#[derive(H5Type)]
struct NXsample{
    name : String,
}
#[derive(H5Type)]
struct NXinstrument{
    name : String,
}
#[derive(H5Type)]
struct NXdata{
    name : String,
}
#[derive(H5Type)]
struct NXcharacterization{
    name : String,
}
#[derive(H5Type)]
struct NXuif{
    name : String,
}*/
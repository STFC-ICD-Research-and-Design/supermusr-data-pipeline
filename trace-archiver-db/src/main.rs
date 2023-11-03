//! This crate uses the benchmarking tool for testing the performance of implementated time-series databases.
//!
//#![allow(dead_code, unused_variables, unused_imports)]
#![warn(missing_docs)]

//use anyhow::{anyhow,Result};
use clap::{Parser, Subcommand};

mod envfile;
use envfile::get_user_confirmation;

use anyhow::{Result, Error, anyhow};
use tdengine as engine;

#[cfg(feature = "benchmark")]
mod benchmarker;

#[cfg(feature = "benchmark")]
use benchmarker::{
    post_benchmark_message, ArgRanges, BenchMark, DataVector, Results, SteppedRange,
};
use engine::{tdengine::TDEngine, TimeSeriesEngine};

use rdkafka::{
    consumer::{stream_consumer::StreamConsumer, CommitMode, Consumer},
    producer::FutureProducer,
    message::{BorrowedMessage, Message}
};

use streaming_types::dat1_digitizer_analog_trace_v1_generated::{
    DigitizerAnalogTraceMessage,
    digitizer_analog_trace_message_buffer_has_identifier,
    root_as_digitizer_analog_trace_message
};

mod error;
mod full_test;
use crate::error::MessageError;

#[derive(Parser)]
#[clap(author, version, about)]
pub(crate) struct Cli {
    #[clap(long, short = 'b', env = "REDPANDA_URL")]
    kafka_broker_url: Option<String>,

    #[clap(long, short = 't', env = "REDPANDA_PORT")]
    kafka_broker_port: Option<u32>,

    #[clap(long, short = 'u', env = "REDPANDA_USER")]
    kafka_username: Option<String>,

    #[clap(long, short = 'p', env = "REDPANDA_PASSWORD")]
    kafka_password: Option<String>,

    #[clap(long, short = 'g', env = "REDPANDA_CONSUMER_GROUP", default_value = "trace-consumer")]
    kafka_consumer_group: String,

    #[clap(long, short = 'k', env = "REDPANDA_TOPIC_SUBSCRIBE")]
    kafka_trace_topic: Option<String>,

    #[clap(long, short = 'B', env = "TDENGINE_URL")]
    td_broker_url: Option<String>,

    #[clap(long, short = 'T', env = "TDENGINE_PORT")]
    td_broker_port: Option<u32>,

    #[clap(long, short = 'U', env = "TDENGINE_USER")]
    td_username: Option<String>,

    #[clap(long, short = 'P', env = "TDENGINE_PASSWORD")]
    td_password: Option<String>,

    #[clap(long, short = 'D', env = "TDENGINE_DATABASE")]
    td_database: Option<String>,

    #[clap(long, short = 'C', env = "TDENGINE_NUM_CHANNELS")]
    td_num_channels: Option<usize>,

    #[command(subcommand)]
    mode: Option<Mode>,
}

#[derive(Subcommand)]
enum Mode {
    #[clap(about = "Listen to messages on the kafka server.")]
    Listen(ListenParameters),
    #[cfg(feature = "benchmark")]
    #[clap(about = "Run the benchmarking system.")]
    BenchmarkLocal(BenchmarkParameters),
    #[cfg(feature = "benchmark")]
    #[clap(about = "Run the benchmarking system.")]
    BenchmarkKafka(BenchmarkParameters),
    #[clap(about = "Output the .env file, making use of any optional arguments specified.")]
    InitEnv,
    #[clap(about = "Delete Timeseries Database. You probably don't want to do this.")]
    DeleteTimeseriesDatabase,
}

#[derive(Parser)]
struct ListenParameters {}

#[cfg(feature = "benchmark")]
#[derive(Parser)]
struct BenchmarkParameters {
    #[clap(long, env = "BENCHMARK_NUM_SAMPLES_RANGE")]
    num_samples_range: Option<SteppedRange>,
    #[clap(long, env = "BENCHMARK_REPEATS")]
    num_repeats: Option<usize>,
    #[clap(long, env = "BENCHMARK_DELAY")]
    delay: Option<u64>,
    #[clap(long)]
    show_output: bool,
    #[clap(long)]
    save_output: bool,

    #[clap(long, default_value = "0")]
    message_delay_ms: u64,
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    dotenv::dotenv().ok();

    log::debug!("Parsing Cli");
    let cli = Cli::parse();

    //  If we are in InitEnv mode then we return after the following block
    if let Some(Mode::InitEnv) = &cli.mode {
        log::debug!("Entering InitEnv Mode");
        envfile::write_env(&cli).map_err(error::Error::DotEnvWrite)?;
        return Ok(());
    }

    //  All other modes require a TDEngine instance
    log::debug!("Createing TDEngine instance");
    let mut tdengine: TDEngine = TDEngine::from_optional(
        cli.td_broker_url,
        cli.td_broker_port,
        cli.td_username,
        cli.td_password,
        cli.td_database,
    )
    .await
    .map_err(error::Error::TDEngine)?;

    //  If we are in DeleteTimeseriesDatabase mode then we return after the following block
    if let Some(Mode::DeleteTimeseriesDatabase) = &cli.mode {
        if get_user_confirmation(
            "Are you sure you want to delete the timeseries database?",
            "Deleting timeseries database.",
            "Deletion cancelled.",
        )? {
            return tdengine.delete_database().await;
        }
    }

    //  All other modes require the TDEngine to be initialised
    tdengine.create_database().await?;
    let num_channels = cli
        .td_num_channels
        .ok_or(error::Error::EnvVar("TDENGINE_NUM_CHANNELS"))?;
    tdengine.init_with_channel_count(num_channels).await?;

    //  If we are in BenchmarkLocal mode then we return after the following block
    #[cfg(feature = "benchmark")]
    if let Some(Mode::BenchmarkLocal(bmk)) = cli.mode {
        log::debug!("Entering Benchmark Mode");
        let arg_ranges = ArgRanges::new(
            bmk.num_samples_range
                .ok_or(error::Error::EnvVar("BENCHMARK_NUM_SAMPLES_RANGE"))?,
        );
        let num_repeats: usize = bmk
            .num_repeats
            .ok_or(error::Error::EnvVar("BENCHMARK_REPEATS"))?;

        let results = benchmark_local(
            tdengine,
            arg_ranges,
            num_channels,
            num_repeats,
            bmk.show_output,
        )
        .await;
        results.calc_multilin_reg();
        if bmk.save_output {
            results.save_csv()?;
        }
        return Ok(());
    }

    //  All other modes require a kafka builder, a topic, and redpanda consumer
    log::debug!("Creating Kafka instance");
    
    let mut client_config =
        common::generate_kafka_client_config(
            &format!("{0}:{1}",cli.kafka_broker_url.unwrap(),cli.kafka_broker_port.unwrap()),
            &None, //cli.kafka_username,
            &None
    );


    let topic = cli
        .kafka_trace_topic
        .ok_or(error::Error::EnvVar("Redpanda Topic"))?; //unwrap_string_or_env_var(cli.kafka_trace_topic, "REDPANDA_TOPIC_SUBSCRIBE");
    
    let consumer: StreamConsumer = client_config
        .set("group.id", &cli.kafka_consumer_group)
        .set("enable.partition.eof", "false")
        .set("session.timeout.ms", "6000")
        .set("enable.auto.commit", "false")
        .create()?;
    consumer.subscribe(&[&topic]);

    //  The listen mode runs infinitely, however a return is included so as not to confuse the borrow checker
    if let Some(Mode::Listen(_)) = cli.mode {
        log::debug!("Entering Listening Mode");
        kafka_consumer(tdengine, consumer).await;
        return Ok(());
    }
    //  The default mode is the same as above, but is included separately in case use is made of the ListenParameters in the future
    if cli.mode.is_none() {
        log::debug!("Entering Listening Mode (as default)");
        kafka_consumer(tdengine, consumer).await;
        return Ok(());
    }

    #[cfg(feature = "benchmark")]
    if let Some(Mode::BenchmarkKafka(bmk)) = cli.mode {
        //  The final mode requires a redpanda producer as well as all the above
        let producer : FutureProducer = client_config.create()?;// redpanda_engine::new_producer(&redpanda_builder)?;

        log::debug!("Entering BenchmarkKafka Mode");
        let arg_ranges = ArgRanges::new(
            bmk.num_samples_range
                .ok_or(error::Error::EnvVar("BENCHMARK_NUM_SAMPLES_RANGE"))?,
        );
        let parameter_space_size = arg_ranges.get_parameter_space_size();
        let num_repeats: usize = bmk
            .num_repeats
            .ok_or(error::Error::EnvVar("BENCHMARK_REPEATS"))?;
        let delay: u64 = bmk.delay.ok_or(error::Error::EnvVar("BENCHMARK_DELAY"))?;
        log::debug!("parameter_space_size = {parameter_space_size}");

        let producer_thread = tokio::spawn(benchmark_kafka_producer_thread(
            arg_ranges,
            producer,
            topic,
            num_repeats,
            num_channels,
            delay,
        ));

        let results = benchmark_kafka(
            tdengine,
            num_repeats * parameter_space_size,
            consumer,
            bmk.show_output,
        )
        .await;
        log::debug!("producer_thread: joining main thread");
        tokio::join!(producer_thread).0?;
        results.calc_multilin_reg();
        if bmk.save_output {
            results.save_csv()?;
        }
    }
    Ok(())
}


pub fn extract_payload<'a, 'b: 'a>(
    message: &'b BorrowedMessage<'b>,
) -> Result<DigitizerAnalogTraceMessage<'a>> {
    let payload = message.payload().ok_or_else(|| {
        log::warn!("Message payload missing.");
        //MessageError::NoPayload(message.topic().to_string()).into()
        anyhow!("Error")
    })?;
    let dat_message = digitizer_analog_trace_message_buffer_has_identifier(payload)
        .then(|| root_as_digitizer_analog_trace_message(payload))
        .ok_or_else(|| {
            log::warn!("Message payload missing identifier.");
            anyhow!("Error")
            //MessageError::NoIdentifier(message.topic().to_owned()).into::<crate::error::Error>()
        })?;
    match dat_message {
        Ok(data) => {
            log::info!(
                "Trace packet: dig. ID: {}, metadata: {:?}",
                data.digitizer_id(),
                data.metadata()
            );
            Ok(data)
        }
        Err(e) => {
            log::warn!("Failed to parse message: {0}", e);
            //Err(MessageError::FailedToParse(message.topic().to_owned(), e).into::<Error>())
            Err(e.into())
        }
    }
}

async fn kafka_consumer(mut tdengine: TDEngine, consumer: StreamConsumer) {
    loop {
        match consumer.recv().await {
            Ok(message) => {
                match extract_payload(&message) {
                    Ok(message) => {
                        if let Err(e) = tdengine.process_message(&message).await {
                            log::warn!("Error processing message : {e}");
                        }
                        if let Err(e) = tdengine.post_message().await {
                            log::warn!("Error posting message to tdengine : {e}");
                        }
                    },
                    Err(e) => log::warn!("Error extracting payload from message: {e}"),
                }
                consumer.commit_message(&message, CommitMode::Async).unwrap();
            },
            Err(e) => log::warn!("Error recieving message from server: {e}"),
            
        }
    }
}

#[cfg(feature = "benchmark")]
async fn benchmark_local(
    mut tdengine: TDEngine,
    arg_ranges: ArgRanges,
    num_channels: usize,
    num_repeats: usize,
    show_output: bool,
) -> Results {
    let mut results = Results::new();
    for _ in 0..num_repeats {
        for s in arg_ranges.iter() {
            let bm = BenchMark::run_benchmark_from_parameters(num_channels, s, &mut tdengine).await;
            if show_output {
                bm.print_init();
                bm.print_results();
            }
            results.push(bm);
        }
    }
    results
}

#[cfg(feature = "benchmark")]
async fn benchmark_kafka_producer_thread(
    arg_ranges: ArgRanges,
    producer: FutureProducer,
    topic: String,
    num_repeats: usize,
    num_channels: usize,
    delay: u64,
) {
    log::debug!("producer_thread: Entering Thread");
    for i in 0..num_repeats {
        for s in arg_ranges.iter() {
            log::debug!("producer_thread: posting message instance {i} with parameters: ({s}) With delay: {delay}");
            post_benchmark_message(num_channels, s, &producer, &topic, delay).await;
        }
    }
}
#[cfg(feature = "benchmark")]
async fn benchmark_kafka(
    mut tdengine: TDEngine,
    num_messages: usize,
    consumer: StreamConsumer,
    show_output: bool,
) -> Results {
    log::debug!("Running Benchmarking Loop");
    let mut results = Results::new();
    for i in 0..num_messages {
        log::debug!("Instance {i}");
        match consumer.recv().await {
            Ok(message) => match extract_payload(&message) {
                Ok(message) => {
                    log::debug!("Received Message");
                    let bm = BenchMark::run_benchmark_from_message(&message, &mut tdengine).await;
                    if show_output {
                        bm.print_init();
                        bm.print_results();
                    }
                    results.push(bm);
                }
                Err(e) => log::warn!("Error extracting payload from message: {e}"),
            },
            Err(e) => log::warn!("Error recieving message from server: {e}"),
        }
    }
    log::debug!("Running Benchmark Analysis");
    results
}

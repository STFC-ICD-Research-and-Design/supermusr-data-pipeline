//! This crate uses the benchmarking tool for testing the performance of implementated time-series databases.
//!
//#![allow(dead_code, unused_variables, unused_imports)]
#![warn(missing_docs)]

//use anyhow::{anyhow,Result};
use clap::Parser;

use log::{debug, info, warn};

//mod envfile;

mod tdengine;
use tdengine as engine;

use anyhow::Result;

/*#[cfg(feature = "benchmark")]
mod benchmarker;

#[cfg(feature = "benchmark")]
use benchmarker::{
    post_benchmark_message, ArgRanges, BenchMark, DataVector, Results, SteppedRange,
};*/
use engine::{tdengine::TDEngine, TimeSeriesEngine};

use rdkafka::{
    consumer::{stream_consumer::StreamConsumer, CommitMode, Consumer},
    message::Message,
};

use streaming_types::dat1_digitizer_analog_trace_v1_generated::{
    digitizer_analog_trace_message_buffer_has_identifier,
    root_as_digitizer_analog_trace_message
};

mod error;
//mod full_test;

#[derive(Parser)]
#[clap(author, version, about)]
pub(crate) struct Cli {
    #[clap(long, short = 'b', env = "KAFKA_BROKER")]
    kafka_broker: Option<String>,

    #[clap(long, short = 'u', env = "KAFKA_USER")]
    kafka_username: Option<String>,

    #[clap(long, short = 'p', env = "KAFKA_PASSWORD")]
    kafka_password: Option<String>,

    #[clap(long, short = 'g', env = "KAFKA_CONSUMER_GROUP", default_value = "trace-consumer")]
    kafka_consumer_group: String,

    #[clap(long, short = 'k', env = "KAFKA_TOPIC", default_value = "Traces")]
    kafka_topic: Option<String>,

    #[clap(long, short = 'B', env = "TDENGINE_BROKER")]
    td_broker: Option<String>,

    #[clap(long, short = 'U', env = "TDENGINE_USER")]
    td_username: Option<String>,

    #[clap(long, short = 'P', env = "TDENGINE_PASSWORD")]
    td_password: Option<String>,

    #[clap(long, short = 'D', env = "TDENGINE_DATABASE")]
    td_database: Option<String>,

    #[clap(long, short = 'C', env = "TDENGINE_NUM_CHANNELS")]
    td_num_channels: Option<usize>,

    #[clap(long, help = "If set, will record benchmarking data")]
    benchmark: bool,

    //#[command(subcommand)]
    //mode: Option<Mode>,
}
/*
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
}*/

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    dotenv::dotenv().ok();

    debug!("Parsing Cli");
    let cli = Cli::parse();
/*
    //  If we are in InitEnv mode then we return after the following block
    if cli.init_env {
        debug!("Entering InitEnv Mode");
        envfile::write_env(&cli).map_err(error::Error::DotEnvWrite)?;
        return Ok(());
    }
 */
    //  All other modes require a TDEngine instance
    debug!("Createing TDEngine instance");
    let mut tdengine: TDEngine = TDEngine::from_optional(
        cli.td_broker,
        cli.td_username,
        cli.td_password,
        cli.td_database,
    )
    .await
    .map_err(error::Error::TDEngine)?;
/*
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
 */
    //  All other modes require the TDEngine to be initialised
    tdengine.create_database().await?;
    let num_channels = cli
        .td_num_channels
        .ok_or(error::Error::EnvVar("TDENGINE_NUM_CHANNELS"))?;
    tdengine.init_with_channel_count(num_channels).await?;
/*
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
 */
    //  All other modes require a kafka builder, a topic, and redpanda consumer
    debug!("Creating Kafka instance");
    
    let mut client_config =
        common::generate_kafka_client_config(
            &cli.kafka_broker.unwrap(),
            &cli.kafka_username,
            &cli.kafka_password
    );

    let topic = cli
        .kafka_topic
        .ok_or(error::Error::EnvVar("Kafka Topic"))?; //unwrap_string_or_env_var(cli.kafka_trace_topic, "REDPANDA_TOPIC_SUBSCRIBE");
    
    let consumer: StreamConsumer = client_config
        .set("group.id", &cli.kafka_consumer_group)
        .set("enable.partition.eof", "false")
        .set("session.timeout.ms", "6000")
        .set("enable.auto.commit", "false")
        .create()?;
    consumer.subscribe(&[&topic])?;

    debug!("Begin Listening For Messages");
    loop {
        match consumer.recv().await {
            Ok(message) => {
                match message.payload() {
                    Some(payload) =>
                    {
                        if digitizer_analog_trace_message_buffer_has_identifier(payload) {
                            match root_as_digitizer_analog_trace_message(payload) {
                                Ok(message) => {
                                    info!(
                                        "Trace packet: dig. ID: {}, metadata: {:?}",
                                        message.digitizer_id(),
                                        message.metadata()
                                    );
                                    if let Err(e) = tdengine.process_message(&message).await {
                                        warn!("Error processing message : {e}");
                                    }
                                    if let Err(e) = tdengine.post_message().await {
                                        warn!("Error posting message to tdengine : {e}");
                                    }
                                },
                                Err(e) => warn!("Failed to parse message: {0}", e)
                            }
                        } else {
                            warn!("Message payload missing identifier.")
                        }
                    },
                    None => warn!("Error extracting payload from message.")
                };
                consumer.commit_message(&message, CommitMode::Async).unwrap();
            },
            Err(e) => warn!("Error recieving message from server: {e}")
        }
    }
}
/*
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
    }*/

/*
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
*/
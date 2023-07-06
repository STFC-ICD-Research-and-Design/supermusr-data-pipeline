//! This crate uses the benchmarking tool for testing the performance of implementated time-series databases.
//! 
#![allow(dead_code,unused_variables,unused_imports)]
#![warn(missing_docs)]

use std::{thread, time::Instant};

use anyhow::Result;

use dotenv;
use clap::{Parser, Subcommand};

mod envfile;
mod utils;
mod simulator;
mod engine;
mod benchmarker;
mod redpanda_engine;

use rayon::iter::plumbing::Producer;
use utils::{log_then_panic, log_then_panic_t, unwrap_num_or_env_var};
use engine::{tdengine::TDEngine, TimeSeriesEngine};
//use engine::influxdb::InfluxDBEngine;
use benchmarker::{SteppedRange, ArgRanges, Results, BenchMark, post_benchmark_message};
use redpanda_engine::{RedpandaEngine, extract_payload, Consumer};
use futures::executor::block_on;

use crate::utils::get_user_confirmation;

#[derive(Parser)]
#[clap(author, version, about)]
pub(crate) struct Cli {
    #[clap(long,short='b')]
    kafka_broker_url: Option<String>,
    
    #[clap(long,short='t')]
    kafka_broker_port: Option<u32>,

    #[clap(long,short='u')]
    kafka_username: Option<String>,

    #[clap(long,short = 'p')]
    kafka_password: Option<String>,

    #[clap(long, short='g')]
    kafka_consumer_group: Option<String>,

    #[clap(long, short = 'k')]
    kafka_trace_topic: Option<String>,


    #[clap(long,short='B')]
    td_broker_url: Option<String>,

    #[clap(long,short='T')]
    td_broker_port: Option<u32>,

    #[clap(long,short='U')]
    td_username: Option<String>,

    #[clap(long,short = 'P')]
    td_password: Option<String>,

    #[clap(long,short='D')]
    td_database: Option<String>,

    #[clap(long,short='C')]
    td_num_channels: Option<usize>,

    #[command(subcommand)]
    mode: Mode,
}

#[derive(Subcommand)]
enum Mode {
    #[clap(about = "Listen to messages on the kafka server.")]
    Normal(NormalParameters),
    #[clap(about = "Run the benchmarking system.")]
    Benchmark(BenchmarkParameters),
    #[clap(about = "Run the benchmarking system.")]
    BenchmarkKafka(BenchmarkParameters),
    #[clap(about = "Output the .env file, making use of any optional arguments specified.")]
    InitEnv,
    #[clap(about = "Delete Timeseries Database. You probably don't want to do this.")]
    DeleteTimeseriesDatabase,
}

#[derive(Parser)]
struct NormalParameters {
}

#[derive(Parser)]
struct BenchmarkParameters {
    #[clap(long)]
    num_messages_range: Option<SteppedRange>,
    #[clap(long)]
    num_channels_range: Option<SteppedRange>,
    #[clap(long)]
    num_samples_range: Option<SteppedRange>,
    #[clap(long)]
    num_repeats: Option<usize>,
    #[clap(long)]
    delay: Option<u64>,

    #[clap(long, default_value = "0")]
    message_delay_ms: u64,
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    
    log::debug!("Parsing Cli");
    let cli = Cli::parse();

    match &cli.mode {
        Mode::InitEnv => {
            log::debug!("Entering InitEnv Mode");
            return Ok(envfile::write_env(&cli))
        },
        _ => (),
    }

    log::debug!("Createing TDEngine instance");
    let mut tdengine: TDEngine = TDEngine::from_optional(
        &cli.td_broker_url,
        &cli.td_broker_port,
        &cli.td_username,
        &cli.td_password,
        &cli.td_database).await;
        
    if let Mode::DeleteTimeseriesDatabase = &cli.mode {
        if get_user_confirmation("Are you sure you want to delete the timeseries database?", "Deleting timeseries database.", "Deletion cancelled.") {
            return tdengine.delete_database().await;
        }
    }

    tdengine.create_database().await?;
    let num_channels = unwrap_num_or_env_var(&cli.td_num_channels,"TDENGINE_NUM_CHANNELS");
    tdengine.init_with_channel_count(num_channels).await?;

    if let Mode::Benchmark(bmk) = &cli.mode {
        log::debug!("Entering Benchmark Mode");
        let arg_ranges = create_arg_ranges(&bmk);
        let num_repeats : usize = unwrap_num_or_env_var(&bmk.num_repeats, "BENCHMARK_REPEATS");

        let results = benchmark_local(&mut tdengine, arg_ranges, num_repeats).await;
        results.calc_multilin_reg();
        results.save_csv(8)?;
        return Ok(())
    }

    
    log::debug!("Creating RedpandaEngine instance");
    let redpanda_engine = RedpandaEngine::from_optional(
        &cli.kafka_broker_url,
        &cli.kafka_broker_port,
        &cli.kafka_username,
        &cli.kafka_password,
        &cli.kafka_consumer_group,
        &cli.kafka_trace_topic
    );
    
    match &cli.mode {
        Mode::Normal(_) => {
            log::debug!("Entering Normal Mode");
            kafka_consumer(&mut tdengine, &redpanda_engine, &cli).await?
        },
        Mode::BenchmarkKafka(bmk) => {
            log::debug!("Entering BenchmarkKafka Mode");
            let arg_ranges = create_arg_ranges(&bmk);
            let parameter_space_size = arg_ranges.get_parameter_space_size();
            let num_repeats : usize = unwrap_num_or_env_var(&bmk.num_repeats, "BENCHMARK_REPEATS");
            let delay : u64 = unwrap_num_or_env_var(&bmk.delay, "BENCHMARK_DELAY");
            log::debug!("parameter_space_size = {parameter_space_size}");

            let producer_thread = tokio::spawn(benchmark_kafka_producer_thread(arg_ranges,redpanda_engine::Producer::new(&redpanda_engine), num_repeats, delay));
            
            let results = benchmark_kafka(&mut tdengine, num_repeats*parameter_space_size, &redpanda_engine).await;
            log::debug!("producer_thread: joining main thread");
            tokio::join!(producer_thread).0?;
            results.calc_multilin_reg();
            results.save_csv(8)?;
        }
        _ => (),
    }
    Ok(())
}



async fn kafka_consumer(tdengine : &mut TDEngine, redpanda : &RedpandaEngine, cli : &Cli) -> Result<()> {
    let consumer = Consumer::new(redpanda);
    loop {
        match consumer.recv().await {
            Ok(message) =>
            match redpanda_engine::extract_payload(&message) {
                Ok(message) => {
                    if let Err(e) = tdengine.process_message(&message).await{ log::warn!("Error processing message : {e}"); }
                    if let Err(e) = tdengine.post_message().await           { log::warn!("Error posting message to tdengine : {e}"); }
                },
                Err(e) => log::warn!("Error extracting payload from message: {e}"),
            },
            Err(e) => log::warn!("Error recieving message from server: {e}"),
        }
    }
}



fn create_arg_ranges(bmk : &BenchmarkParameters) -> ArgRanges {
    ArgRanges {
        num_channels_range: unwrap_num_or_env_var(&bmk.num_channels_range, "BENCHMARK_NUM_CHANNELS_RANGE"),
        num_samples_range: unwrap_num_or_env_var(&bmk.num_samples_range, "BENCHMARK_NUM_SAMPLES_RANGE")
    }
}




async fn benchmark_local(tdengine : &mut TDEngine, arg_ranges : ArgRanges, num_repeats : usize) -> Results {
    let mut results = Results::default();
    for _ in 0..num_repeats {
        for (c,s) in arg_ranges.iter() {
            results.push(BenchMark::run_benchmark_from_parameters(c,s,tdengine).await);
        }
    }
    results
}






async fn benchmark_kafka_producer_thread(arg_ranges : ArgRanges, producer : redpanda_engine::Producer, num_repeats : usize, delay : u64) {
    log::debug!("producer_thread: Entering Thread");
    for i in 0..num_repeats {
        for (c, s) in arg_ranges.iter() {
            log::debug!("producer_thread: posting message instance {i} with parameters: ({c},{s}) With delay: {delay}");
            post_benchmark_message(c,s, &producer,delay).await;
        }
    }
}

async fn benchmark_kafka(tdengine : &mut TDEngine, num_messages : usize, redpanda : &RedpandaEngine) -> Results {
    log::debug!("Running Benchmarking Loop");
    let consumer = Consumer::new(redpanda);
    let mut benchmarker = Results::default();
    for i in 0..num_messages {
        log::debug!("Instance {i}");
        match consumer.recv().await {
            Ok(message) =>
            match redpanda_engine::extract_payload(&message) {
                Ok(message) => {
                    log::debug!("Received Message");
                    benchmarker.push(BenchMark::run_benchmark_from_message(&message, tdengine).await);
                },
                Err(e) => log::warn!("Error extracting payload from message: {e}"),
            },
            Err(e) => log::warn!("Error recieving message from server: {e}"),
        }
    }
    log::debug!("Running Benchmark Analysis");
    benchmarker
}
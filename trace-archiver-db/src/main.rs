//! This crate uses the benchmarking tool for testing the performance of implementated time-series databases.
//! 
#![allow(dead_code,unused_variables,unused_imports)]
#![warn(missing_docs)]

use anyhow::Result;
//use streaming_types::dat1_digitizer_analog_trace_v1_generated::{DigitizerAnalogTraceMessage, root_as_digitizer_analog_trace_message};
//use flatbuffers::FlatBufferBuilder;

mod simulator;
mod engine;
mod benchmarker;
mod consumer;

use engine::{tdengine::TDEngine,influxdb::InfluxDBEngine, TimeSeriesEngine};
use benchmarker::{SteppedRange,Analyser,EngineAnalyser, adhoc_benchmark};
use consumer::{RedpandaEngine, extract_payload};
use dotenv;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    if dotenv::dotenv().ok().is_none() { panic!("Failed to load dotenv. Is the .env file in the working directory?"); }

    let mut tdengine: TDEngine = TDEngine::new().await;
    tdengine.reset_database().await?;
    tdengine.init_with_channel_count(dotenv::var("TDENGINE_NUM_CHANNELS").unwrap_or_else(|e|panic!("TDEngine Channel Count not found in .env: {}",e)).parse().unwrap()).await?;

    /*let redpanda_engine = RedpandaEngine::new();
    loop {
        let message = redpanda_engine.recv().await.unwrap();
        let message = consumer::extract_payload(&message).unwrap();
        let (args, time_record) = adhoc_benchmark(message, &mut tdengine).await;
        println!("{0}, {1}, {2:?}, {3:?}", args.num_samples, args.num_channels, time_record.total_time, time_record.posting_time);
    }*/

    //let mut influxdb: InfluxDBEngine = InfluxDBEngine::new().await;
    //influxdb.reset_database().await?;
    
    let mut benchmarker = Analyser::new(SteppedRange(1..=4,1),SteppedRange(8..=8,2),SteppedRange(10001..=10001,50));
    benchmarker.push_timeseries_engine(&mut tdengine);
    benchmarker.run_benchmarks().await;
    /*
    benchmarker.push_timeseries_engine(&mut influxdb);
    */
 
    Ok(())
}
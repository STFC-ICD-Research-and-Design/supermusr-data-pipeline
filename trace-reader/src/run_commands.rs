use std::time::Duration;
use anyhow::Result;

use chrono::Utc;
use rdkafka::{producer::{FutureProducer, FutureRecord}, util::Timeout};
use supermusr_streaming_types::{ecs_6s4t_run_stop_generated::{finish_run_stop_buffer, RunStop, RunStopArgs}, ecs_pl72_run_start_generated::{finish_run_start_buffer, RunStart, RunStartArgs}, flatbuffers::FlatBufferBuilder};



pub(crate) async fn send_run_start_command(producer : &FutureProducer) -> Result<()> {
    let mut fbb = FlatBufferBuilder::new();
    let run_start = RunStartArgs {
        start_time: Utc::now().timestamp_millis() as u64,
        run_name: Some(fbb.create_string("Test")),
        instrument_name: Some(fbb.create_string("McGuffin")),
        ..Default::default()
    };
    let message = RunStart::create(&mut fbb, &run_start);
    finish_run_start_buffer(&mut fbb, message);
    let timeout = Timeout::After(Duration::from_millis(6000));
    producer.send(FutureRecord::to("controls").payload(fbb.finished_data()).key(""),timeout).await.map_err(|(e,_)|e)?;
    Ok(())
}


pub(crate) async fn send_run_stop_command(producer : &FutureProducer) -> Result<()> {
    let mut fbb = FlatBufferBuilder::new();
    let run_stop = RunStopArgs {
        stop_time: Utc::now().timestamp_millis() as u64,
        run_name: Some(fbb.create_string("Test")),
        ..Default::default()
    };
    let message = RunStop::create(&mut fbb, &run_stop);
    finish_run_stop_buffer(&mut fbb, message);
    let timeout = Timeout::After(Duration::from_millis(6000));
    producer.send(FutureRecord::to("controls").payload(fbb.finished_data()).key(""),timeout).await.map_err(|(e,_)|e)?;
    Ok(())
}
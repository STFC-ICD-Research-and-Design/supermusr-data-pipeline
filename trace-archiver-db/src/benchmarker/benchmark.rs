use super::args::Args;

use std::time::{Instant, Duration};

use common::Time;
use flatbuffers::FlatBufferBuilder;
use streaming_types::dat1_digitizer_analog_trace_v1_generated::{root_as_digitizer_analog_trace_message, DigitizerAnalogTraceMessage};

use crate::{simulator::{self, MalformType, Malform}, engine::TimeSeriesEngine, redpanda_engine::Producer};

#[derive(Default)]
pub struct TimeRecords {
    pub total_time : Duration,
    pub posting_time : Duration,
}

#[derive(Default)]  
pub(crate) struct BenchMark {
    pub(super) args: Args,
    pub(super) time: TimeRecords,
}

fn create_benchmark_message<'a>(num_channels: usize, num_samples: usize) -> FlatBufferBuilder<'a> {
    let mut fbb: FlatBufferBuilder = FlatBufferBuilder::new();
    simulator::create_partly_random_message_with_now(&mut fbb,
        0..=12,
        0..=24,
        num_samples,
        num_channels,
        &Malform::default(),
    ).unwrap();
    fbb
}


pub(crate) async fn post_benchmark_message(c: usize, s: usize, producer : &Producer, delay : u64) {
    let m = create_benchmark_message(c,s);
    let timer = Instant::now();
    producer.post(m.finished_data()).await.unwrap();
    let elapsed = timer.elapsed().as_millis() as u64;
    if delay > elapsed {
        std::thread::sleep(Duration::from_millis(delay - elapsed));
    }
}

impl BenchMark {
    pub(crate) fn new(c: usize, s: usize) -> BenchMark { BenchMark{ args: Args::new(c,s),..Default::default() } }
    pub(crate) async fn run_benchmark_from_parameters(c: usize, s: usize, engine: &mut dyn TimeSeriesEngine) -> Self {
        let fbb = create_benchmark_message(c,s);
        let message: DigitizerAnalogTraceMessage = root_as_digitizer_analog_trace_message(fbb.finished_data()).unwrap();

        BenchMark { args: Args::new(c,s), time: Self::run_benchmark(&message, engine).await }
    }
    pub(crate) async fn run_benchmark_from_message(message: &DigitizerAnalogTraceMessage<'_>, engine: &mut dyn TimeSeriesEngine) -> Self {
        let args = Args::new(
            message.channels().unwrap_or_default().len(),
            message.channels().unwrap_or_default().iter().map(|c|c.voltage().unwrap_or_default().len()).max().unwrap_or_default()
        );
        BenchMark { args, time: Self::run_benchmark(message, engine).await }
    }
    async fn run_benchmark(message: &DigitizerAnalogTraceMessage<'_>, engine: &mut dyn TimeSeriesEngine) -> TimeRecords {
        //  begin timer
        let timer = Instant::now();
        engine.process_message(message).await.unwrap();
        let posting_timer = Instant::now();
        engine.post_message().await.unwrap();
        //  end timer
        TimeRecords { total_time: timer.elapsed(), posting_time: posting_timer.elapsed() }
    }

    pub(super) fn print_init(&self) {
        print!("{:72}", self.args.output_init() );
    }
    pub(super) fn print_results(&self) {
        print!("{:32}",format!("Total time {} ms,", self.time.total_time.as_nanos() as f64 / 1_000_000.0));
        print!("{:32}",format!("posting time {} ms,", self.time.posting_time.as_nanos() as f64 / 1_000_000.0));
        println!();
    }
}

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

fn create_benchmark_message(args : &Args) -> FlatBufferBuilder {
    let mut fbb: FlatBufferBuilder = FlatBufferBuilder::new();
    simulator::create_partly_random_message_with_now(&mut fbb,
        0..=12,
        0..=24,
        args.num_samples,
        args.num_channels,
        &Malform::default(),
    ).unwrap();
    fbb
}

impl BenchMark {
    pub(crate) fn new(m: usize, c: usize, s: usize) -> BenchMark { BenchMark{ args: Args::new(m,c,s),..Default::default() } }
    pub(crate) async fn run_benchmark(mut self, engine: &mut dyn TimeSeriesEngine) -> Self {
        let fbb = create_benchmark_message(&self.args);
        let msg: DigitizerAnalogTraceMessage = root_as_digitizer_analog_trace_message(fbb.finished_data()).unwrap();

        //  begin timer
        self.time = TimeRecords::default();
        let timer = Instant::now();
        engine.process_message(&msg).await.unwrap();
        {
            let posting_timer = Instant::now();
            engine.post_message().await.unwrap();
            self.time.posting_time = posting_timer.elapsed();
        }
        self.time.total_time = timer.elapsed();
        //  end timer
        self
    }
    pub(super) fn new_with_results(m: usize, c: usize, s: usize, tr : TimeRecords) -> BenchMark { BenchMark{ args: Args::new(m,c,s), time:tr } }

    pub(crate) async fn post_benchmark_message(&mut self, producer : &Producer, delay : u64) {
        for m in (0..self.args.num_messages).into_iter()
        .map(|_| create_benchmark_message(&self.args)) {
            let timer = Instant::now();
            producer.post(m.finished_data()).await.unwrap();
            let elapsed = timer.elapsed().as_millis() as u64;
            if delay > elapsed {
                std::thread::sleep(Duration::from_millis(delay - elapsed));
            }
        }
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

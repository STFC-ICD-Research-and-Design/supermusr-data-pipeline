use super::args::Args;

use std::time::{Instant, Duration};

use flatbuffers::FlatBufferBuilder;
use streaming_types::dat1_digitizer_analog_trace_v1_generated::{root_as_digitizer_analog_trace_message, DigitizerAnalogTraceMessage};

use crate::{simulator, engine::TimeSeriesEngine};

#[derive(Default)]  
pub struct TimeRecords {
    pub total_time : Duration,
    pub posting_time : Duration,
}

#[derive(Default)]  
pub(super) struct BenchMark {
    pub(super) args: Args,
    pub(super) time: TimeRecords,
}

fn create_benchmark_message(args : &Args) -> FlatBufferBuilder {
    let mut fbb: FlatBufferBuilder = FlatBufferBuilder::new();
    simulator::create_partly_random_message_with_now(&mut fbb,
        0..=12,
        0..=24,
        args.num_samples,
        args.num_channels
    ).unwrap();
    fbb
}

impl BenchMark {
    pub(super) fn new(m: usize, c: usize, s: usize) -> BenchMark { BenchMark{ args: Args::new(m,c,s),..Default::default() } }

    pub(super) async fn run_benchmark(&mut self, engine: &mut dyn TimeSeriesEngine) {
        let fbbs : Vec<FlatBufferBuilder> = (0..self.args.num_messages)
            .into_iter()
            .map(|_| create_benchmark_message(&self.args))
            .collect();

        //  begin timer
        self.time = TimeRecords::default();
        for fbb in fbbs {
            let timer = Instant::now();
            let msg: DigitizerAnalogTraceMessage = root_as_digitizer_analog_trace_message(fbb.finished_data()).unwrap();
            engine.process_message(&msg).await.unwrap();
            {
                let posting_timer = Instant::now();
                engine.post_message().await.unwrap();
                self.time.posting_time = self.time.posting_time.checked_add(posting_timer.elapsed()).unwrap();
            }
            self.time.total_time = self.time.total_time.checked_add(timer.elapsed()).unwrap();
        }
        //  end timer
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

use std::time::{Duration, Instant};
use std::{env, fs::File, io::Write, iter::StepBy, ops::RangeInclusive, str::FromStr};

use common::Time;
use flatbuffers::FlatBufferBuilder;
use rand::rngs::ThreadRng;
use redpanda::RedpandaProducer;
use streaming_types::dat1_digitizer_analog_trace_v1_generated::{
    root_as_digitizer_analog_trace_message, DigitizerAnalogTraceMessage,
};

use crate::engine::TimeSeriesEngine;
use crate::utils::{log_then_panic, log_then_panic_t, unwrap_num_or_env_var};
use trace_simulator::{self, Malform, MalformType};

use super::linreg::{create_data, create_model, print_summary_statistics};
use crate::redpanda_engine;

//use itertools::{iproduct, ConsTuples, Product};

///  A range object that includes an inclusive range object and a step size.
#[derive(Clone)]
pub struct SteppedRange(pub RangeInclusive<usize>, pub usize);

impl SteppedRange {
    pub fn from_string(src: String) -> Result<Self, anyhow::Error> {
        let params: Vec<usize> = src
            .split(':')
            .map(|s| {
                s.parse()
                    .unwrap_or_else(|e| log_then_panic_t(format!("{src}: {e}")))
            })
            .collect();
        if params.len() != 3 {
            log_then_panic(format!(
                "SteppedRange: Wrong number of parameters in {src}: {params:?}"
            ))
        }
        Ok(SteppedRange(params[0]..=params[1], params[2]))
    }
    pub fn iter(&self) -> StepBy<RangeInclusive<usize>> {
        self.0.clone().into_iter().step_by(self.1)
    }
}

impl FromStr for SteppedRange {
    type Err = anyhow::Error;

    fn from_str(src: &str) -> Result<Self, Self::Err> {
        let params: Vec<usize> = src
            .split(':')
            .map(|s| {
                s.parse()
                    .unwrap_or_else(|e| log_then_panic_t(format!("{src}: {e}")))
            })
            .collect();
        if params.len() != 3 {
            log_then_panic(format!(
                "SteppedRange: Wrong number of parameters in {src}: {params:?}"
            ))
        }
        Ok(SteppedRange(params[0]..=params[1], params[2]))
    }
}

/// Args contains all the parameteres used in a benchmark.
/// Currently this is limited to the number of samples.
#[derive(Default, PartialEq)]
pub struct Args {
    pub num_samples: usize,
}
impl Args {
    pub(super) fn new(s: usize) -> Args {
        Args { num_samples: s }
    }
    pub(super) fn output_init(&self) -> String {
        format!("Running benchmark for {0} datapoints.", self.num_samples)
    }
}

/// ArgRanges defines the extent over which Args can range in benchmarks, as well as the interval between each value of the parameter space.
#[derive(Clone)]
pub(crate) struct ArgRanges {
    pub(crate) num_samples_range: SteppedRange,
}

type ParameterSpace = StepBy<RangeInclusive<usize>>;

impl ArgRanges {
    pub(crate) fn from_option_or_env(
        num_samples_range: Option<SteppedRange>,
        env_var: &str,
    ) -> Self {
        ArgRanges {
            num_samples_range: unwrap_num_or_env_var(num_samples_range, env_var),
        }
    }
    /// Abstracts over the space of parameters
    /// #Returns
    /// An iterator which ranges over all values in the parameter space
    pub(crate) fn iter(&self) -> ParameterSpace {
        self.num_samples_range.iter()
    }

    /// #Returns
    /// The number of elements in the parameter space
    pub(crate) fn get_parameter_space_size(&self) -> usize {
        self.iter().collect::<Vec<_>>().len()
    }
}

#[derive(Default)]
pub struct TimeRecords {
    pub total_time: Duration,
    pub posting_time: Duration,
}

#[derive(Default)]
pub(crate) struct BenchMark {
    pub(super) args: Args,
    pub(super) time: TimeRecords,
}

/// Creates a partially random message with the specified number of channels and number of samples
/// #Arguments
/// num_channels - the number of channels in the message
/// num_samples - the number of samples to generate in the message
/// #Returns
/// A FlatBufferBuilder instance containing the generated message
fn create_benchmark_message<'a>(num_channels: usize, num_samples: usize) -> FlatBufferBuilder<'a> {
    let mut fbb: FlatBufferBuilder = FlatBufferBuilder::new();
    trace_simulator::create_partly_random_message_with_now(
        &mut fbb,
        0..=12,
        0..=24,
        num_samples,
        num_channels,
        &Malform::default(),
    )
    .unwrap();
    fbb
}

/// Creates a partially random message with the specified number of channels and number of samples,
/// posts the byte stream to the given Producer instance, and then pauses for the given delay time.
/// #Arguments
/// num_channels - the number of channels in the message
/// num_samples - the number of samples to generate in the message
/// producer - the producer to which the stream should be sent
/// delay - the number of milliseconds to pause after sending the stream
pub(crate) async fn post_benchmark_message(
    c: usize,
    s: usize,
    producer: &RedpandaProducer,
    topic: &str,
    delay: u64,
) {
    let m = create_benchmark_message(c, s);
    let timer = Instant::now();
    redpanda_engine::producer_post(producer, topic, m.finished_data())
        .await
        .unwrap();
    let elapsed = timer.elapsed().as_millis() as u64;
    if delay > elapsed {
        std::thread::sleep(Duration::from_millis(delay - elapsed));
    }
}

impl BenchMark {
    /// Creates a partially random message with the specified number of channels and number of samples,
    /// and benchmarks how long it takes to insert into the given TimeSeriesEngine
    /// #Arguments
    /// num_channels - the number of channels in the message
    /// num_samples - the number of samples to generate in the message
    /// engine - a timeseries engine
    /// #Returns
    /// A Benchmark instance containing the times
    pub(crate) async fn run_benchmark_from_parameters(
        c: usize,
        s: usize,
        engine: &mut dyn TimeSeriesEngine,
    ) -> Self {
        let fbb = create_benchmark_message(c, s);
        let message: DigitizerAnalogTraceMessage =
            root_as_digitizer_analog_trace_message(fbb.finished_data()).unwrap();

        BenchMark {
            args: Args::new(s),
            time: Self::run_benchmark(&message, engine).await,
        }
    }

    /// Accepts a DigitizerAnalogTraceMessage and benchmarks how long it takes to insert
    /// into the given TimeSeriesEngine.
    /// #Arguments
    /// message - a reference to a DigitizerAnalogTraceMessage instance
    /// engine - a timeseries engine
    /// #Returns
    /// A Benchmark instance containing the times
    pub(crate) async fn run_benchmark_from_message(
        message: &DigitizerAnalogTraceMessage<'_>,
        engine: &mut dyn TimeSeriesEngine,
    ) -> Self {
        let args = Args::new(
            message
                .channels()
                .unwrap_or_default()
                .iter()
                .map(|c| c.voltage().unwrap_or_default().len())
                .max()
                .unwrap_or_default(),
        );
        BenchMark {
            args,
            time: Self::run_benchmark(message, engine).await,
        }
    }
    async fn run_benchmark(
        message: &DigitizerAnalogTraceMessage<'_>,
        engine: &mut dyn TimeSeriesEngine,
    ) -> TimeRecords {
        //  begin timer
        let timer = Instant::now();
        engine.process_message(message).await.unwrap();
        let posting_timer = Instant::now();
        engine.post_message().await.unwrap();
        //  end timer
        TimeRecords {
            total_time: timer.elapsed(),
            posting_time: posting_timer.elapsed(),
        }
    }

    pub(crate) fn print_init(&self) {
        print!("{:72}", self.args.output_init());
    }
    pub(crate) fn print_results(&self) {
        print!(
            "{:32}",
            format!(
                "Total time {} ms,",
                self.time.total_time.as_nanos() as f64 / 1_000_000.0
            )
        );
        print!(
            "{:32}",
            format!(
                "posting time {} ms,",
                self.time.posting_time.as_nanos() as f64 / 1_000_000.0
            )
        );
        println!();
    }
}

/// The struct BenchMarkData loops through the parameter space of
/// Example
/// ```rust
/// let mut engine = TDEngine::new();
/// let mut benchmark_data = BenchMarkData::new(num_messages_range : SteppedRange(0..4,2), num_channels_range : SteppedRange(0..4,2), num_samples_range : SteppedRange(0..4,2))
/// benchmark_data.run_benchmark(engine : engine);
/// benchmark_data.calc_multilin_reg();
/// ```
pub(crate) type Results = Vec<BenchMark>;

pub(crate) trait DataVector {
    fn calc_multilin_reg(&self);
    fn save_csv(&self) -> Result<(), std::io::Error>;
}

impl DataVector for Results {
    /// Fits a multilinear regression model to the results stored by a call the run_benchmark,
    /// and prints the results.
    fn calc_multilin_reg(&self) {
        let t: Vec<f64> = self
            .iter()
            .map(|x| x.time.total_time.as_nanos() as f64)
            .collect();
        let pt: Vec<f64> = self
            .iter()
            .map(|x| x.time.posting_time.as_nanos() as f64)
            .collect();
        let s: Vec<f64> = self.iter().map(|x| x.args.num_samples as f64).collect();

        let data = create_data(vec![("time", t), ("post_time", pt), ("samples", s)]).unwrap();
        let model = create_model(&data, "time ~ samples").unwrap();
        print_summary_statistics(&model, "Total Time");
        let model = create_model(&data, "post_time ~ samples").unwrap();
        print_summary_statistics(&model, "Posting Time");
    }
    fn save_csv(&self) -> Result<(), std::io::Error> {
        let cd = env::current_dir()
            .unwrap_or_else(|e| log_then_panic_t(format!("Cannot obtain current directory : {e}")));
        let path = cd.join("data/data.csv");
        let mut file = File::create(path)
            .unwrap_or_else(|e| log_then_panic_t(format!("Cannot create .csv file : {e}")));
        writeln!(&mut file, "num_samples, total_time, posting_time")?;
        for res in self.iter() {
            writeln!(
                &mut file,
                "{}, {}, {}",
                res.args.num_samples,
                res.time.total_time.as_micros(),
                res.time.posting_time.as_micros()
            )?;
        }
        Ok(())
    }
}

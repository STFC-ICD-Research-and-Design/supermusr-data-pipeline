use core::num;
use std::time::Duration;
use itertools::iproduct;

use rayon::*;

use crate::engine::TimeSeriesEngine;

use super::{engine_analyser::EngineAnalyser, linreg::{create_data, create_model, print_summary_statistics},  args::{ArgRanges, SeriesArgs}, SteppedRange};


/// The struct BenchmarkAnalyser contains a vector of timeseries engines and associated 
/// Example
/// ```rust
/// let mut engine = TDEngine::new();
/// let mut benchmarker = BenchmarkAnalyser::new(num_messages_range : SteppedRange(0..4,2), num_channels_range : SteppedRange(0..4,2), num_datapoints_range : SteppedRange(0..4,2))
/// benchmarker.push_time_series(&engine);
/// benchmarker.run_benchmarks();
/// benchmark_data.calc_multilin_reg();
/// ```


pub struct Analyser<'a> {
    arg_ranges : ArgRanges,
    benchmarks : Vec<(&'a mut dyn TimeSeriesEngine,EngineAnalyser)>,
}

impl<'a> Analyser<'a> {
    ///  Create a new instance of Bank with the given parameters indicting the parameter space.
    /// Each point in the parameter space defines a different test.
    /// #Arguments
    /// *num_messages_range is a SteppedRange indicating the minimum, maximum, and increment of the number of messages to simulate.
    /// *num_channels_range is a SteppedRange indicating the minimum, maximum, and increment of the number of channels per message.
    /// *num_datapoints_range is a SteppedRange indicating the minimum, maximum, and increment of the number of measurements per channel.
    pub fn new(num_messages_range: SteppedRange, num_channels_range: SteppedRange, num_samples_range: SteppedRange) -> Analyser<'a> {
        Analyser{
            arg_ranges : ArgRanges { num_messages_range,  num_channels_range, num_samples_range },
            benchmarks : Vec::<(&'a mut dyn TimeSeriesEngine,EngineAnalyser)>::default(),
        }
    }
    pub fn push_timeseries_engine<'b>(&'b mut self, engine : &'a mut dyn TimeSeriesEngine) {
        self.benchmarks.push((engine,EngineAnalyser::new_with_arg_ranges(self.arg_ranges.clone())));
    }
    pub async fn run_benchmarks(self) {
        for (engine, mut benchmark_data) in self.benchmarks {
            benchmark_data.run_benchmarks(engine).await;
        }
    }
}
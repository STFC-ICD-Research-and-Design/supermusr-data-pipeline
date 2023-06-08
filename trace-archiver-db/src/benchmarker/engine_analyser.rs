use std::{time::{Duration, Instant}, ops::RangeInclusive,iter::StepBy};

use streaming_types::dat1_digitizer_analog_trace_v1_generated::DigitizerAnalogTraceMessage;

use crate::engine::TimeSeriesEngine;

use super::{benchmark::{BenchMark, TimeRecords}, linreg::{create_data, create_model, print_summary_statistics},  args::{SteppedRange, SeriesArgs, ArgRanges, Args}};

/// The struct BenchMarkData loops through the parameter space of 
/// Example
/// ```rust
/// let mut engine = TDEngine::new();
/// let mut benchmark_data = BenchMarkData::new(num_messages_range : SteppedRange(0..4,2), num_channels_range : SteppedRange(0..4,2), num_samples_range : SteppedRange(0..4,2))
/// benchmark_data.run_benchmark(engine : engine);
/// benchmark_data.calc_multilin_reg();
/// ```
pub struct EngineAnalyser {
    arg_ranges: ArgRanges,
    results : Vec<BenchMark>,
    series: Vec<Series>,
}

impl EngineAnalyser {
    ///  Create a new instance of BenchMarkData with the given parameters indicting the parameter space.
    /// Each point in the parameter space defines a different test.
    /// #Arguments
    /// *num_messages_range is a SteppedRange indicating the minimum, maximum, and increment of the number of messages to simulate.
    /// *num_channels_range is a SteppedRange indicating the minimum, maximum, and increment of the number of channels per message.
    /// *num_datapoints_range is a SteppedRange indicating the minimum, maximum, and increment of the number of measurements per channel.
    pub fn new(num_messages_range: SteppedRange, num_channels_range: SteppedRange, num_samples_range: SteppedRange) -> EngineAnalyser {
        EngineAnalyser{
            arg_ranges : ArgRanges { num_messages_range,  num_channels_range, num_samples_range },
            results : Vec::<BenchMark>::default(),
            series : Vec::<Series>::default(),
        }
    }
    
    pub(super) fn new_with_arg_ranges(arg_ranges : ArgRanges) -> EngineAnalyser {
        EngineAnalyser{
            arg_ranges : arg_ranges,
            results : Vec::<BenchMark>::default(),
            series : Vec::<Series>::default(),
        }
    }
    /// Loops through the parameter space and runs the benchmark for each point,
    /// storing the results to be analysed by calc_multilin_reg.
    /// Also deletes all previous results and series.
    /// #Arguments
    /// *engine - An instance implementing the TimeSeriesEngine trait.
    pub async fn run_benchmarks(&mut self, engine: &mut dyn TimeSeriesEngine) {
        self.results.clear();
        self.series.clear();

        let parameter_space : Vec<_> = self.arg_ranges.get_parameter_space().collect();
        println!("Running benchmark with parameter space of size {}", parameter_space.len());

        for (m,c,d) in parameter_space {
            let mut bm: BenchMark = BenchMark::new(m,c,d);

            bm.print_init();
            bm.run_benchmark(engine).await;
            bm.print_results();

            self.results.push(bm);
        }
        println!();
    }

    /// Fits a multilinear regression model to the results stored by a call the run_benchmark,
    /// and prints the results.
    pub fn calc_multilin_reg(&self) {
        let t: Vec<f64> = self.results.iter().map(|x|x.time.total_time.as_nanos() as f64).collect();
        let pt: Vec<f64> = self.results.iter().map(|x|x.time.posting_time.as_nanos() as f64).collect();
        let m: Vec<f64> = self.results.iter().map(|x|x.args.num_messages as f64).collect();
        let c: Vec<f64> = self.results.iter().map(|x|x.args.num_channels as f64).collect();
        let d: Vec<f64> = self.results.iter().map(|x|x.args.num_samples as f64).collect();

        let data = create_data(vec![("time",t), ("post_time",pt), ("messages",m), ("channels",c), ("data",d)]).unwrap();
        let model = create_model(&data, "time ~ messages + channels + data").unwrap();
        print_summary_statistics(&model, "Total Time");
        let model = create_model(&data, "post_time ~ messages + channels + data").unwrap();
        print_summary_statistics(&model, "Posting Time");
    }

    /// Create a series from results stored by a call to run_benchmark.
    /// #Arguments
    /// *name - is a string slice containing the name of the series.
    /// *series_type - determines the type of timing data to collect.
    /// *args is a BenchMarkSeriesArgs struct defining the fixed parameters of the series.
    /// *Parameters given by Some(_) are fixed, whereas a None value indicates the independent variable.
    /// Example
    /// This creates a series consisting of all results with num_messages = 10, num_channels = 8
    /// and num_datapoints between 0 and 100 inclusive.
    /// ```rust
    /// let args = BenchMarkSeriesArgs::NumSamplesVariable{num_messages:10,num_channels:8,num_samples:0..=100};
    /// benchmarker.create_series("Total Time (10 messages, 8 channels)", SeriesType::TotalTime, args);
    /// ```
    pub fn create_series(&mut self, name : &str, series_type : SeriesType, args : SeriesArgs) {
        let data = self.results
            .iter()
            .filter(|x|x.args.is_matched(&args))
            .map(|bm|(bm.args.extract_param(&args).unwrap(),
                    match series_type {
                        SeriesType::TotalTime => bm.time.total_time,
                        SeriesType::PostingTime => bm.time.posting_time
                    }.as_nanos()))
            .collect();
        self.series.push(Series{
            name: name.to_string(),
            args,
            points: data,
        });
    }

    ///  Saves all created series to a csv file.
    pub fn save_series(&self) {
    }
}



pub async fn adhoc_benchmark(msg: DigitizerAnalogTraceMessage<'_>, engine: &mut impl TimeSeriesEngine) -> (Args,TimeRecords) {
    //  begin timer
    let mut time = TimeRecords::default();
    let timer = Instant::now();
    engine.process_message(&msg).await.unwrap();
    {
        let posting_timer = Instant::now();
        engine.post_message().await.unwrap();
        time.posting_time = time.posting_time.checked_add(posting_timer.elapsed()).unwrap();
    }
    time.total_time = time.total_time.checked_add(timer.elapsed()).unwrap();
    
    (Args {
        num_messages: 1,
        num_channels: msg.channels().unwrap().len(),
        num_samples: msg.channels().unwrap().iter().next().unwrap().voltage().unwrap().iter().len(),
    },time)
}


pub enum SeriesType { TotalTime, PostingTime, }

struct Series {
    name : String,
    args : SeriesArgs,
    points : Vec<(usize, u128)>,
}

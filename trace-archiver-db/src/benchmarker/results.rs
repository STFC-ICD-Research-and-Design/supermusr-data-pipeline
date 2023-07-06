use std::{time::{Duration, Instant}, ops::RangeInclusive,iter::StepBy, fs::File, io::Write, env};

use common::Channel;
use streaming_types::dat1_digitizer_analog_trace_v1_generated::{DigitizerAnalogTraceMessage, root_as_digitizer_analog_trace_message};

use crate::{engine::TimeSeriesEngine, redpanda_engine::Producer, utils::log_then_panic_t};

use super::{benchmark::{BenchMark, TimeRecords}, linreg::{create_data, create_model, print_summary_statistics},  args::{SteppedRange, SeriesArgs, ArgRanges, Args}};

/// The struct BenchMarkData loops through the parameter space of 
/// Example
/// ```rust
/// let mut engine = TDEngine::new();
/// let mut benchmark_data = BenchMarkData::new(num_messages_range : SteppedRange(0..4,2), num_channels_range : SteppedRange(0..4,2), num_samples_range : SteppedRange(0..4,2))
/// benchmark_data.run_benchmark(engine : engine);
/// benchmark_data.calc_multilin_reg();
/// ```
#[derive(Default)]
pub struct Results(Vec<BenchMark>);

impl Results {
    pub(crate) fn iter(&self) -> core::slice::Iter<BenchMark> { self.0.iter() }
    pub(crate) fn clear(&mut self) { self.0.clear() }
    pub(crate) fn push(&mut self, bm : BenchMark) { self.0.push(bm) }
    /// Fits a multilinear regression model to the results stored by a call the run_benchmark,
    /// and prints the results.
    pub fn calc_multilin_reg(&self) {
        let t: Vec<f64> = self.0.iter().map(|x|x.time.total_time.as_nanos() as f64).collect();
        let pt: Vec<f64> = self.0.iter().map(|x|x.time.posting_time.as_nanos() as f64).collect();
        let c: Vec<f64> = self.0.iter().map(|x|x.args.num_channels as f64).collect();
        let d: Vec<f64> = self.0.iter().map(|x|x.args.num_samples as f64).collect();

        let data = create_data(vec![("time",t), ("post_time",pt), ("channels",c), ("data",d)]).unwrap();
        let model = create_model(&data, "time ~ channels + data").unwrap();
        print_summary_statistics(&model, "Total Time");
        let model = create_model(&data, "post_time ~ channels + data").unwrap();
        print_summary_statistics(&model, "Posting Time");
    }
    pub fn save_csv(&self, num_channels : usize) -> Result<(),std::io::Error> {
        let cd = env::current_dir().unwrap_or_else(|e|log_then_panic_t(format!("Cannot obtain current directory : {e}")));
        let path = cd.join("data.csv");
        let mut file = File::create(path).unwrap_or_else(|e|log_then_panic_t(format!("Cannot create .env file : {e}")));
        writeln!(&mut file, "num_samples, total_time, posting_time")?;
        for res in self.0.iter().filter(|bm| bm.args.num_channels == num_channels) {
            writeln!(&mut file, "{}, {}, {}", res.args.num_samples, res.time.total_time.as_micros(), res.time.posting_time.as_micros())?;
        }
        Ok(())
    }
}


pub enum SeriesType { TotalTime, PostingTime, }

struct Series {
    name : String,
    args : SeriesArgs,
    points : Vec<(usize, u128)>,
}

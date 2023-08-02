/// The benchmarker module allows you to benchmark timeseries databases which
/// implement the Engine trait.
mod linreg;
use linreg::{create_data, create_model, print_summary_statistics};

mod benchmark;
pub(crate) use benchmark::{
    post_benchmark_message, ArgRanges, BenchMark, DataVector, Results, SteppedRange,
};

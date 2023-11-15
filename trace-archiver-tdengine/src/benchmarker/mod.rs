/// The benchmarker module allows you to benchmark timeseries databases which
/// implement the Engine trait.
mod linreg;

mod benchmark;
pub(crate) use benchmark::{
    post_benchmark_message, ArgRanges, BenchMark, DataVector, Results, SteppedRange,
};

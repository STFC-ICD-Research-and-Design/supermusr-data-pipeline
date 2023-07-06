/// The benchmarker module allows you to benchmark timeseries databases which
/// implement the Engine trait.

mod linreg;
use linreg::{create_model, print_summary_statistics, create_data};

mod benchmark;
pub(crate) use benchmark::{ArgRanges, SteppedRange, BenchMark, post_benchmark_message, Results, DataVector};

/// The benchmarker module allows you to benchmark timeseries databases which
/// implement the Engine trait.

mod linreg;
use linreg::{create_model, print_summary_statistics, create_data};

mod args;
pub(crate) use args::{ArgRanges,SteppedRange};

mod benchmark;
pub(crate) use benchmark::{BenchMark, post_benchmark_message};

mod results;
pub use results::Results;

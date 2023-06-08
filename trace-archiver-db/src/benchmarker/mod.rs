/// The benchmarker module allows you to benchmark timeseries databases which
/// implement the Engine trait.

mod linreg;
use linreg::{create_model, print_summary_statistics, create_data};

mod args;
use args::{Args,SeriesArgs};
pub use args::SteppedRange;

mod benchmark;
use benchmark::BenchMark;

mod engine_analyser;
pub use engine_analyser::EngineAnalyser;
pub use engine_analyser::adhoc_benchmark;

mod analyser;
pub use analyser::Analyser;

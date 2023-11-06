pub mod composite;
pub mod exponential_smoothing_window;
pub mod finite_differences;
pub mod gate;
pub mod iter;
pub mod noise_smoothing_window;
pub mod smoothing_window;
pub mod trivial;

pub use iter::{WindowFilter, WindowIter};

use crate::tracedata::Temporal;

pub trait Window: Clone {
    type TimeType: Temporal;
    type InputType: Copy;
    type OutputType;

    fn push(&mut self, value: Self::InputType) -> bool;
    fn stats(&self) -> Option<Self::OutputType>;
    fn apply_time_shift(&self, time: Self::TimeType) -> Self::TimeType;
}

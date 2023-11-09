//pub(crate) mod composite;
pub(crate) mod finite_differences;
pub(crate) mod iter;

pub(crate) mod smoothing_window;

pub(crate) use finite_differences::FiniteDifferences;
pub(crate) use iter::WindowFilter;
pub(crate) use smoothing_window::SmoothingWindow;

use super::{Real, RealArray, Temporal, TracePoint};

pub(crate) trait Window: Clone {
    type TimeType: Temporal;
    type InputType: Copy;
    type OutputType;

    fn push(&mut self, value: Self::InputType) -> bool;
    fn stats(&self) -> Option<Self::OutputType>;
    fn apply_time_shift(&self, time: Self::TimeType) -> Self::TimeType;
}

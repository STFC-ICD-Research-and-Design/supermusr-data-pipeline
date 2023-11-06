use std::fmt::{Debug, Display};

use crate::Real;

use super::{EventValuedDetector, FeedbackDetector};

mod biexponential;
mod pulse_detector_inner;

pub use biexponential::Biexponential;
pub use pulse_detector_inner::Gaussian;
pub use pulse_detector_inner::PulseDetector;
pub use pulse_detector_inner::PulseEvent;

pub trait PulseModel: Default + Debug + Display + Clone {
    fn get_value_at(&self, time: Real) -> Real;
    fn get_derivative_at(&self, time: Real) -> Real;
    fn get_second_derivative_at(&self, time: Real) -> Real;

    fn get_effective_interval(&self, bound: Real) -> (Real, Real);

    fn from_data(peak_time: Real, peak_value: Real, area_under_curve: Real) -> Self;
    fn from_data2(_data: Vec<Real>, _start: Real, _peak: Real) -> Self {
        Self::default()
    }
    fn from_basic(mean: Real, amplitude: Real) -> Self;
}

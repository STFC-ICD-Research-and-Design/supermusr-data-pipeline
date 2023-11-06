use crate::Real;

use crate::window::Window;

#[derive(Default, Clone)]
pub struct ExponentialSmoothingWindow {
    smoothing_factor: Real,
    value: Real,
}
impl ExponentialSmoothingWindow {
    pub fn new(smoothing_factor: Real) -> Self {
        ExponentialSmoothingWindow {
            smoothing_factor,
            ..Default::default()
        }
    }
}
impl Window for ExponentialSmoothingWindow {
    type TimeType = Real;
    type InputType = Real;
    type OutputType = Real;

    fn push(&mut self, value: Real) -> bool {
        self.value = value * self.smoothing_factor + self.value * (1. - self.smoothing_factor);
        true
    }
    fn stats(&self) -> Option<Real> {
        Some(self.value)
    }
    fn apply_time_shift(&self, time: Real) -> Real {
        time
    } //time - (self.size - 1.)/2.0 }
}

#[cfg(test)]
mod tests {
    /*
    use crate::processing;

    use common::Intensity;
    use itertools::Itertools;
    use super::super::iter::WindowFilter;
    use super::*;
    use assert_approx_eq::assert_approx_eq;
    */
}

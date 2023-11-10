use super::{
    Real,
    Window
};

#[derive(Default, Clone)]
pub struct Baseline {
    baseline: Real,
    value : Real,
    smoothing_factor: Real,
    warm_up: usize,
    time: usize,
}
impl Baseline {
    pub fn new(warm_up: usize, smoothing_factor: Real) -> Self {
        Baseline {
            warm_up,
            smoothing_factor,
            ..Default::default()
        }
    }
}
impl Window for Baseline {
    type TimeType = Real;
    type InputType = Real;
    type OutputType = Real;

    fn push(&mut self, value: Real) -> bool {
        self.value = value - self.baseline;
        if self.time < self.warm_up {
            self.baseline = value * self.smoothing_factor + self.baseline * (1. - self.smoothing_factor);
            self.time += 1;
            true
        } else {
            false
        }
    }
    fn output(&self) -> Option<Real> {
        (self.time < self.warm_up).then_some(self.value)
    }
    fn apply_time_shift(&self, time: Real) -> Real {
        time - (self.warm_up as Real)
    }
}

#[cfg(test)]
mod tests {
    use super::super::WindowFilter;
    use super::*;


    #[test]
    fn sample_data() {
        let input: Vec<Real> = vec![1.0, 6.0, 2.0, 1.0, 3.0, 1.0, 0.0];
        let output: Vec<Real> = input
            .into_iter()
            .enumerate()
            .map(|(i, v)| (i as Real, v as Real))
            .window(Baseline::new(3,1.0))
            .map(|(_, x)| x)
            .collect();

        assert_eq!(output[0], 0.);
        assert_eq!(output[1], 2.);
        assert_eq!(output[2], 0.);
        assert_eq!(output[3], -1.);
    }
}

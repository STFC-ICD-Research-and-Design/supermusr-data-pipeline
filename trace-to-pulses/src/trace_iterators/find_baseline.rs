/// This find the baseline of a trace stream by reading through the first warm_up values
/// and setting the baseline to the minimum of these. From hereon, all subsequent values
///
use crate::{tracedata::TraceData, Real};

use super::iter::TraceIter;
use super::iter::TraceIterType;

#[derive(Default, Clone)]
pub struct FindBaseline {
    warm_up: usize,
    baseline: Real,
    smoothing_factor: Real,
}

impl FindBaseline {
    pub fn new(warm_up: usize) -> Self {
        FindBaseline {
            warm_up,
            baseline: Real::default(),
            smoothing_factor: 0.1,
        }
    }
}

impl TraceIterType for FindBaseline {}

impl<I> Iterator for TraceIter<FindBaseline, I>
where
    I: Iterator,
    I::Item: TraceData<ValueType = Real>,
{
    type Item = (<I::Item as TraceData>::TimeType, Real);

    fn next(&mut self) -> Option<Self::Item> {
        while self.child.warm_up > 0 {
            match self.source.next() {
                Some(trace) => {
                    self.child.baseline = trace.take_value() * self.child.smoothing_factor
                        + self.child.baseline * (1. - self.child.smoothing_factor);
                    //self.child.baseline = Real::min(self.child.baseline, trace.take_value())
                }
                None => return None,
            }
            self.child.warm_up -= 1;
            //if self.warm_up == 0 { log::info!("{0}",self.baseline); }
        }
        self.source
            .next()
            .map(|trace| (trace.get_time(), trace.take_value() - self.child.baseline))
    }
}

pub trait FindBaselineFilter<I>
where
    I: Iterator,
    I::Item: TraceData,
{
    fn find_baseline(self, warm_up: usize) -> TraceIter<FindBaseline, I>;
}

impl<I> FindBaselineFilter<I> for I
where
    I: Iterator,
    I::Item: TraceData,
{
    fn find_baseline(self, warm_up: usize) -> TraceIter<FindBaseline, I> {
        TraceIter::new(FindBaseline::new(warm_up), self)
    }
}

#[cfg(test)]
mod tests {
    use super::{FindBaselineFilter, Real};
    use common::Intensity;

    #[test]
    fn sample_data() {
        let input: Vec<Intensity> = vec![1, 6, 2, 1, 3, 1, 0];
        let output: Vec<Real> = input
            .into_iter()
            .enumerate()
            .map(|(i, v)| (i as Real, v as Real))
            .find_baseline(3)
            .map(|(_, x)| x)
            .collect();

        assert_eq!(output[0], 0.);
        assert_eq!(output[1], 2.);
        assert_eq!(output[2], 0.);
        assert_eq!(output[3], -1.);
    }
}

use std::collections::VecDeque;

use crate::{
    Real,
    RealArray,
    tracedata::TraceData,
};
use num::integer::binomial;

use super::iter::{TraceIter, TraceIterType};

#[derive(Default, Clone)]
pub struct FiniteDifferences<const N: usize>
{
    coefficients: Vec<Vec<Real>>,
    values: VecDeque<Real>,
}

impl<const N: usize> TraceIterType for FiniteDifferences<N> {}

impl<const N: usize> FiniteDifferences<N> {
    pub fn new() -> Self {
        FiniteDifferences {
            values: VecDeque::<Real>::with_capacity(N),
            coefficients: (0..N)
                .map(|n| {
                    (0..=n)
                        .map(|k| (if k & 1 == 1 { -1. } else { 1. }) * (binomial(n, k) as Real))
                        .collect()
                })
                .collect(),
        }
    }
    fn nth_difference(&self, n: usize) -> Real {
        (0..=n)
            .map(|k| self.coefficients[n][k] * self.values[k])
            .sum()
    }
    fn next_no_index(&mut self, value: Real) -> RealArray<N> {
        self.values.push_front(value);
        let mut diffs = [Real::default(); N];
        for n in 0..N {
            diffs[n] = self.nth_difference(n);
        }
        self.values.pop_back();
        diffs
    }
}

impl<I, const N: usize> Iterator for TraceIter<FiniteDifferences<N>,I> where
    I: Iterator,
    I::Item : TraceData<TimeType = Real, ValueType = Real>,
{
    type Item = (Real, RealArray<N>);

    fn next(&mut self) -> Option<Self::Item> {
        let mut value = self.source.next();
        while self.child.values.len() + 1 < self.child.values.capacity() {
            match &value {
                Some(trace) => {
                    self.child.values.push_front(trace.get_value().clone());
                    value = self.source.next();
                }
                None => return None,
            }
        }
        let trace = value?;
        Some((trace.get_time(),self.child.next_no_index(trace.take_value())))
    }
}





pub trait FiniteDifferencesFilter<I, const N: usize> where
    I: Iterator,
    I::Item : TraceData<TimeType = Real, ValueType = Real>,
{
    fn finite_differences(self) -> TraceIter<FiniteDifferences<N>, I>;
}

impl<I, const N: usize> FiniteDifferencesFilter<I, N> for I where
    I: Iterator,
    I::Item : TraceData<TimeType = Real, ValueType = Real>,
{
    fn finite_differences(self) -> TraceIter<FiniteDifferences<N>, I> {
        TraceIter::new(FiniteDifferences::new(), self)
    }
}




#[cfg(test)]
mod tests {
    use super::{FiniteDifferencesFilter, Real, RealArray};
    use common::Intensity;

    #[test]
    fn sample_data() {
        let input: Vec<Intensity> = vec![0, 6, 2, 1, 3, 1, 0];
        let output: Vec<RealArray<3>> = input
            .into_iter()
            .enumerate()
            .map(|(i, v)| (i as Real, v as Real))
            .finite_differences()
            .map(|(_, x)| x)
            .collect();

        assert_eq!(output[0], [2., -4., -10.]);
        assert_eq!(output[1], [1., -1., 3.]);
        assert_eq!(output[2], [3., 2., 3.]);
        assert_eq!(output[3], [1., -2., -4.]);
        assert_eq!(output[4], [0., -1., 1.]);
    }
}

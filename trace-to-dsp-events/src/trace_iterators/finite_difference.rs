use std::collections::VecDeque;

use num::integer::binomial;
use crate::Real;
use super::RealArray;


#[derive(Clone)]
pub struct FiniteDifferencesIter<I, const N : usize> where I : Iterator<Item = (Real,Real)> {
    coefficients : Vec<Vec<Real>>,
    values : VecDeque<Real>,
    source : I,
}

impl<I, const N : usize> FiniteDifferencesIter<I,N> where I : Iterator<Item = (Real,Real)> {
    pub fn new(source : I) -> Self {
        FiniteDifferencesIter { source,
            values : VecDeque::<Real>::with_capacity(N),
            coefficients : (0..N).map(|n|
                    (0..=n).map(|k|
                        (if k & 1 == 1 { -1. } else { 1. })
                        *(binomial(n,k) as Real)).collect()
                ).collect()
        }
    }
}

impl<I, const N : usize> FiniteDifferencesIter<I,N> where I : Iterator<Item = (Real,Real)> {
    fn nth_difference(&self, n : usize) -> Real {
        (0..=n).map(|k|self.coefficients[n][k]*self.values[k]).sum()
    }
    fn next_no_index(&mut self, value : Option<Real>) -> Option<RealArray<N>> {
        match value {
            Some(v0) => {
                self.values.push_front(v0);
                let mut diffs = [Real::default(); N];
                for n in 0..N {
                    diffs[n] = self.nth_difference(n);
                }
                self.values.pop_back();
                Some(diffs)
            },
            None => None,
        }
    }
}

impl<I, const N : usize> Iterator for FiniteDifferencesIter<I,N> where I : Iterator<Item = (Real,Real)> {
    type Item = (Real,RealArray<N>);

    fn next(&mut self) -> Option<Self::Item> {
        let mut value = self.source.next();
        while self.values.len() + 1 < self.values.capacity() {
            match value {
                Some((_,v)) => {
                    self.values.push_front(v);
                    value = self.source.next();
                },
                None => return None,
            }
        }
        let index = value.map_or(Real::default(), |(i,_)|i);
        self.next_no_index(value.map(|(_,v)|v))
            .map(|d|(index,d))
    }
}

pub trait FiniteDifferencesFilter<I, const N : usize> where I: Iterator<Item = (Real,Real)>  {
    fn finite_differences(self) -> FiniteDifferencesIter<I,N>;
}

impl<I, const N : usize> FiniteDifferencesFilter<I,N> for I where I: Iterator<Item = (Real,Real)> {
    fn finite_differences(self) -> FiniteDifferencesIter<I,N> {
        FiniteDifferencesIter::new(self)
    }
}




#[cfg(test)]
mod tests {
    use super::{FiniteDifferencesFilter, RealArray, Real};
    use common::Intensity;

    #[test]
    fn sample_data() {
        let input : Vec<Intensity> = vec![0,6,2,1,3,1,0];
        let output: Vec<RealArray<3>> = input.into_iter()
            .enumerate()
            .map(|(i,v)|(i as Real, v as Real))
            .finite_differences()
            .map(|(_,x)|x).collect();

        assert_eq!(output[0], [2.,-4.,-10.]);
        assert_eq!(output[1], [1.,-1.,3.]);
        assert_eq!(output[2], [3.,2.,3.]);
        assert_eq!(output[3], [1.,-2.,-4.]);
        assert_eq!(output[4], [0.,-1.,1.]);
    }
}
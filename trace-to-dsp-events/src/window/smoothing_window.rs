use std::collections::VecDeque;

use common::Intensity;
use common::Time;
use crate::detectors::event::{Event,EventClass,TimeValue};
use crate::{Detector, Real, Integer, trace_iterators::RealArray};

use crate::window::Window;

pub mod extract {
    use super::*;
    pub fn enumerated_mean((i,Stats{value,mean,variance}) : (Real,Stats)) -> (Real,Real) {
        (i,mean)
    }
}


#[derive(Default)]
struct MeanVar {
    mean : Real,
    variance : Real,
}

#[derive(Default,Clone)]
pub struct Stats {
    pub value : Real,
    pub mean : Real,
    pub variance : Real,
}
#[derive(Default)]
pub struct SmoothingWindow {
    stats : Stats,
    value: Real,
    sum: Real,
    sum_of_squares : Real,
    size : Real,
    size_m1 : Real,
    window : VecDeque<MeanVar>,
}
impl SmoothingWindow {
    pub fn new(size : usize) -> Self {
        if size < 2 {
            panic!("Size must be >= 2");
        }
        SmoothingWindow { window: VecDeque::<MeanVar>::with_capacity(size),
            size: size as Real,
            size_m1 : size as Real - 1.,
            ..Default::default()
        }
    }
    pub fn is_full(&self) -> bool {
        self.window.len() == self.window.capacity()
    }
}
impl Window for SmoothingWindow {
    type InputType = Real;
    type OutputType = Stats;

    fn push(&mut self, value : Real) -> bool {
        self.value = value;
        if self.is_full() {let old = self.window.pop_front().unwrap_or_default();
            self.sum -= old.mean;
            self.sum_of_squares -= old.variance;
        }
        let new = MeanVar { mean : value, variance : (value - self.sum/self.size).powi(2) };
        self.sum += new.mean;
        self.sum_of_squares += new.variance;
        self.window.push_back(new);
        self.is_full()
    }
    fn stats(&self) -> Option<Stats> {
        if self.is_full() {
            Some(Stats{value: self.value, mean: self.sum/self.size, variance: self.sum_of_squares/self.size_m1} )
        } else {
            None
        }
    }
}





#[cfg(test)]
mod tests {
    use crate::processing;

    use super::*;
    use super::super::WindowFilter;

    #[test]
    #[should_panic]
    fn test_window_size_zero() {
        let data = [4,3,2,5,6,1,5,7,2,4];
        data.iter()
            .enumerate()
            .map(processing::make_enumerate_real)
            .window(SmoothingWindow::new(0));
    }
    
    #[test]
    #[should_panic]
    fn test_window_size_one() {
        let data = [4,3,2,5,6,1,5,7,2,4];
        data.iter()
            .enumerate()
            .map(processing::make_enumerate_real)
            .window(SmoothingWindow::new(1));
    }
    
    #[test]
    fn test_window_size_two() {
        let data = [4,3,2,5,6,1,5,7,2,4];
        assert!(
            data.iter()
                .enumerate()
                .map(processing::make_enumerate_real)
                .window(SmoothingWindow::new(2))
                .next()
                .is_some()
        );
    }

    #[test]
    fn test_no_data() {
        let data = [];
        assert!(
            data.iter()
                .enumerate()
                .map(processing::make_enumerate_real)
                .window(SmoothingWindow::new(3))
                .next()
                .is_none()
        );
    }
    #[test]
    fn test_insufficient_data() {
        let data = [4,3];
        assert!(
            data.iter()
                .enumerate()
                .map(processing::make_enumerate_real)
                .window(SmoothingWindow::new(3))
                .next()
                .is_none()
        );
    }
    #[test]
    fn test_minimal() {
        let data = [4,3];
        let (i,stats) = data.iter()
            .enumerate()
            .map(processing::make_enumerate_real)
            .window(SmoothingWindow::new(2))
            .next()
            .unwrap();
        assert_eq!(i,1.);
        assert_eq!(stats.value,3.);
        assert_eq!(stats.mean,7./2.);
        assert_eq!(stats.variance,((4. as Real - 0.).powi(2) + (3. as Real - 4./2.).powi(2))/1.);
    }
    #[test]
    fn test_three_data() {
        let data = [4,3,1];
        let (i,stats) = data.iter()
            .enumerate()
            .map(processing::make_enumerate_real)
            .window(SmoothingWindow::new(2))
            .skip(1)
            .next()
            .unwrap();
        assert_eq!(i,2.);
        assert_eq!(stats.value,1.);
        assert_eq!(stats.mean,2.);
        assert_eq!(stats.variance,((3. as Real - 7./2.).powi(2) + (1. as Real - 2.).powi(2))/1.);
    }

    #[test]
    fn test_three_data_three_window() {
        let data = [4,3,1];
        let (i,stats) = data.iter()
            .enumerate()
            .map(processing::make_enumerate_real)
            .window(SmoothingWindow::new(3))
            .next()
            .unwrap();
        assert_eq!(i,2.);
        assert_eq!(stats.value,1.);
        assert_eq!(stats.mean,8./3.);
        assert_eq!(stats.variance,((4. as Real - 0.).powi(2) + (3. as Real - 4./3.).powi(2) + (1. as Real - 7./3.).powi(2))/2.);
    }
}
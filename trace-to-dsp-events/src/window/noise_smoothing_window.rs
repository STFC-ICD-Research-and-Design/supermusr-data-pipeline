use std::collections::VecDeque;

use common::Intensity;
use common::Time;
use crate::detectors::event::{Event,EventClass,TimeValue};
use crate::{Detector, Real, Integer, trace_iterators::RealArray};

use crate::window::{Window, smoothing_window::SmoothingWindow};

use super::smoothing_window::Stats;


#[derive(Default)]
pub struct NoiseSmoothingWindow {
    threshold : Real,
    influence : Real,
    position : Real,
    window : SmoothingWindow,
}
impl NoiseSmoothingWindow {
    pub fn new(size : usize, threshold : Real, influence : Real) -> Self {
        NoiseSmoothingWindow { threshold, influence, window: SmoothingWindow::new(size),..Default::default()}
    }
}
impl Window for NoiseSmoothingWindow {
    type InputType = Real;
    type OutputType = Stats;

    fn push(&mut self, value : Real) -> bool {
        if let Some(Stats{value: _, mean, variance}) = self.window.stats() {
            let true_mean = mean + self.position;
            //println!("({0},{1}) {2}", value - mean,variance*self.threshold, value);
            if Real::powi(value - true_mean,2) > variance*Real::powi(self.threshold,2) {
                self.position = value;
                //let sign = if value - mean > 0. { 1. } else { -1. };
                true
                //self.window.push(value)
            } else {
                self.window.push(value - self.position)
            }
        } else {
            self.window.push(value)
        }
        
    }
    fn stats(&self) -> Option<Self::OutputType> {
        let stats = self.window.stats()?;
        Some(Stats{value: stats.value, mean: stats.mean + self.position, variance: stats.variance})
    }
}
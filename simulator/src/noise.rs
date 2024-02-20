use supermusr_common::{Intensity, Time};

use crate::json::{self, NoiseAttributes, NoiseSource};


pub(crate) struct Noise<'a> {
    source: &'a NoiseSource,
    prev: f64,
}

impl<'a> Noise<'a> {
    pub(crate) fn new(source: &'a NoiseSource) -> Self {
        Self {
            source,
            prev: f64::default()
        }
    }

    pub(crate) fn noisify(&mut self, value: f64, time: Time, frame_index : usize) -> f64 {
        self.prev = self.source.smooth(self.source.sample(time,frame_index), self.prev, frame_index);
        value + self.prev
    }
}

use crate::trace_iterators::TraceData;
use crate::{
    Detector,
    events::event::Event,
};
use std::fmt::Display;
use std::fmt::Debug;



//#[derive(Default)]
pub struct TracePartition<I, D> where
    I : Iterator,
    I::Item : TraceData<TimeType = D::TimeType, ValueType = D::ValueType>,
    D: Detector,
{
    pub event : Event<D::DataType>,
    pub iter : I,
    pub length : usize,
}

impl<I, D> TracePartition<I, D> where
    I : Iterator + Clone + Debug,
    I::Item : TraceData<TimeType = D::TimeType, ValueType = D::ValueType>,
    D: Detector,
    D::TimeType : Default + Debug + PartialEq,
    D::ValueType : Default + Debug,
    D::DataType : Clone,
{
    pub fn get_event(&self) -> &Event<D::DataType> {
        &self.event
    }
    pub fn iter(&self) -> SubPartitionIter<I> {
        SubPartitionIter { iter: self.iter.clone(), length: self.length }
    }
}

impl<I,D> Clone for TracePartition<I,D> where
    I : Iterator + Clone,
    I::Item : TraceData<TimeType = D::TimeType, ValueType = D::ValueType>,
    D: Detector,
    D::TimeType :  Clone,
    D::ValueType : Clone,
    D::DataType : Clone,
{
    fn clone(&self) -> Self {
        Self { event: self.event.clone(), iter: self.iter.clone(), length: self.length.clone() }
    }
}


impl<I,D> Debug for TracePartition<I,D> where
    I : Iterator + Debug,
    I::Item : TraceData<TimeType = D::TimeType, ValueType = D::ValueType>,
    D: Detector,
    D::TimeType : Debug,
    D::ValueType : Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TracePartition").field("event", &self.event).field("iter", &self.iter).field("length", &self.length).finish()
    }
}

impl<I, D> Display for TracePartition<I, D> where
    I : Iterator + Clone + Debug,
    I::Item : TraceData<TimeType = D::TimeType, ValueType = D::ValueType>,
    D: Detector,
    D::TimeType : Default + Clone + Debug,
    D::ValueType : Default + Clone + Debug,
    D::DataType : Clone,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("event:{0}, from {1:?} to {2}",self.event, self.iter, self.length))
    }
}







#[derive(Default)]
pub struct SubPartitionIter<I> where I : Iterator {
    iter : I,
    length : usize,
}

impl<I> Iterator for SubPartitionIter<I> where I : Iterator {
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        if self.length > 0 {
            self.length -= 1;
            self.iter.next()
        } else {
            None
        }
    }
}







#[derive(Clone)]
pub struct PartitionIter<I, D> where
    I : Iterator,
    I::Item : TraceData<TimeType = D::TimeType, ValueType = D::ValueType>,
    D: Detector,
    D::TimeType : Default + Clone + Debug,
    D::ValueType : Default + Clone + Debug,
    D::DataType : Clone,
{
    detector: D,
    source: I,
}

impl<I, D> PartitionIter<I, D> where
    I : Iterator,
    I::Item : TraceData<TimeType = D::TimeType, ValueType = D::ValueType>,
    D: Detector,
    D::TimeType : Default + Clone + Debug,
    D::ValueType : Default + Clone + Debug,
    D::DataType : Clone,
{
    pub fn new(source: I, detector: D) -> Self {
        PartitionIter { source, detector }
    }
    #[cfg(test)]
    pub fn get_detector(&self) -> &D {
        &self.detector
    }
}

impl<I, D> Iterator for PartitionIter<I,D> where
    I : Iterator + Clone + Debug,
    I::Item : TraceData<TimeType = D::TimeType, ValueType = D::ValueType>,
    D: Detector,
    D::TimeType : Default + Clone + Debug,
    D::ValueType : Default + Clone + Debug,
    D::DataType : Clone,
{
    type Item = TracePartition<I,D>;

    fn next(&mut self) -> Option<Self::Item> {
        let iter = self.source.clone();
        let mut length : usize = 0;
        loop {
            length += 1;
            let val = self.source.next()?;
            match self.detector.signal(val.get_time(),val.clone_value()) {
                Some(event) => return Some(TracePartition { event, iter, length }),
                None => (),
            };
        }
    }
}
pub trait PartitionFilter<I,D> where
    I : Iterator,
    I::Item : TraceData<TimeType = D::TimeType, ValueType = D::ValueType>,
    D: Detector,
    D::TimeType : Default + Clone + Debug,
    D::ValueType : Default + Clone + Debug,
    D::DataType : Clone,
{
    fn partition_on_detections(self, detector: D) -> PartitionIter<I,D>;
}

impl<I, D> PartitionFilter<I,D> for I where
    I : Iterator,
    I::Item : TraceData<TimeType = D::TimeType, ValueType = D::ValueType>,
    D: Detector,
    D: Detector,
    D::TimeType : Default + Clone + Debug + 'static,
    D::ValueType : Default + Clone + Debug + 'static,
    D::DataType : Clone,
{
    fn partition_on_detections(self, detector: D) -> PartitionIter<I,D> {
        PartitionIter::new(self, detector)
    }
}



#[cfg(test)]
mod tests {
    use crate::{
        Real,
        peak_detector::PeakDetector
    };

    use super::*;
    use common::Intensity;

    #[test]
    fn sample_data() {
        let input: Vec<Intensity> = vec![0, 6, 2, 1, 3, 1, 0];
        let _output: Vec<_> = input
            .iter()
            .enumerate()
            .map(|(i, v)| (i as Real, *v as Real))
            .partition_on_detections(PeakDetector::default())
            //.map(|(_, x)| x)
            .collect();
    }
}
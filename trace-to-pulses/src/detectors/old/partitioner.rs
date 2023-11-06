use std::collections::VecDeque;
use std::default;
use std::fmt::Display;
use std::mem::take;

use crate::events::{
    Event,
    EventData,
    EventWithData,
    SimpleEvent,
    TimeValue,
    multiple_events::MultipleEvents
};
use crate::window::{Window, smoothing_window,smoothing_window::SmoothingWindow};
use crate::{Detector, Real, RealArray};
use common::Intensity;
use fitting::approx::assert_abs_diff_eq;
use fitting::Gaussian;
use fitting::gaussian::GaussianError;
use fitting::ndarray::{array, Array, Array1};
use num::complex::ComplexFloat;
use std::fmt::Debug;

use log;

use super::change_detector;
use super::composite::CompositeDetector;

#[derive(Default, Debug, Clone)]
pub struct PartitionData<V, D> where
    V: Default + Clone + Debug + 'static,
    D: EventData
{
    pub data : D,
    pub prev : &'static [(Real,V)],
}

impl<V,D> PartitionData<V, D> where
    V: Default + Clone + Debug + 'static,
    D: EventData
{
    pub fn get_data(&self) -> &D {
        &self.data
    }
}

impl<V, D> EventData for PartitionData<V,D> where
    V: Default + Clone + Debug + 'static,
    D: EventData
{}

impl<V, D> Display for PartitionData<V,D> where
    V: Default + Clone + Debug,
    D: EventData
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{0}: {1:?}",self.data, self.prev))
    }
}


pub trait DataWithPartition : EventData where
{
    type ValueType : Default + Clone + Debug;

    fn prev_iter(&self) -> std::slice::Iter<'_,(Real,Self::ValueType)>;
}

impl<V,D> DataWithPartition for PartitionData<V,D> where
    V: Default + Clone + Debug + 'static,
    D: EventData
{
    type ValueType = V;

    fn prev_iter(&self) -> std::slice::Iter<'_,(Real,V)> {
        self.prev.iter()
    }
}

type PartitionEvent<V,D> = SimpleEvent<PartitionData<V,D>>;



pub struct Partitioner<Det,Evt, V,D> where
    Det : Detector<ValueType = V, TimeType = Real, EventType = Evt>,
    Evt : EventWithData<DataType = D>,
    V: Default + Clone + Debug + 'static,
    D: EventData
{
    min_size : usize,
    prev : Option<&'static [(Real,V)]>,
    detector: Det,
}

impl<Det,Evt,V,D> Partitioner<Det,Evt, V, D> where
    Det : Detector<ValueType = V, TimeType = Real, EventType = Evt>,
    Evt : EventWithData<DataType = D>,
    V: Default + Clone + Debug,
    D: EventData
{
    pub fn new(min_size: usize, detector: Det) -> Partitioner<Det,Evt,V,D> {
        Partitioner { min_size, detector, prev: None }
    }
}

impl<Det,Evt,V,D> Clone for Partitioner<Det,Evt,V, D> where
    Det : Detector<ValueType = V, TimeType = Real, EventType = Evt> + Clone,
    Evt : EventWithData<DataType = D>,
    V: Default + Clone + Debug + 'static,
    D: EventData
{
    fn clone(&self) -> Self {
        Self { min_size: self.min_size.clone(), prev: self.prev.clone(), detector: self.detector.clone() }
    }
}

impl<Det,Evt,V,D> Detector for Partitioner<Det,Evt,V, D> where
    Det : Detector<ValueType = V, TimeType = Real, EventType = Evt>,
    Evt : EventWithData<DataType = D>,
    V: Default + Clone + Debug + 'static,
    D: EventData
{
    type TimeType = Real;
    type ValueType = V;
    type EventType = PartitionEvent<V,D>;

    fn signal(&mut self, time: Real, value: Self::ValueType) -> Option<Self::EventType> {

        self.prev.push((time,value.clone()));
        if self.prev.len() > self.min_size {
            if let Some(event) = self.detector.signal(time, value) {
                Some(PartitionEvent::<V,D> {
                    time,
                    data: PartitionData::<V,D> {
                        data: event.take_data(),
                        prev: take(&mut self.prev),
                    },
                })
            } else {
                None
            }
         } else {
            None
        }
    }
}
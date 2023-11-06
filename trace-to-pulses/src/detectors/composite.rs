use std::fmt::Display;

use crate::events::{
    event::Event,
};
use crate::tracedata::EventData;
use crate::{Detector, Real, RealArray};

type ConstituantType<D> = Box<dyn Detector<TimeType = Real, ValueType = Real, DataType = D>>;
//#[derive(Clone)]
pub struct CompositeDetector<const N: usize, D: EventData> {
    detectors: [ConstituantType<D>; N],
}

impl<const N: usize, D: EventData> CompositeDetector<N, D> {
    pub fn new(detectors: [ConstituantType<D>; N]) -> CompositeDetector<N, D> {
        CompositeDetector::<N, D> { detectors }
    }
}

#[derive(Clone, Debug)]
pub struct CompositeData<D: EventData, const N : usize> {
    value: RealArray<N>,
    data: Vec<(usize,D)>,
}
impl<D: EventData, const N : usize> Default for CompositeData<D,N> {
    fn default() -> Self {
        CompositeData{
            value : [Real::default(); N],
            data: Vec::<(usize,D)>::default(),
        }
    }
}
pub type CompositeEvent<D,const N : usize> = Event<Real,CompositeData<D,N>>;

impl<D: EventData, const N : usize> CompositeData<D,N> {
    pub fn get_value(&self) -> &RealArray<N> {
        &self.value
    }
    pub fn get_data(&self) -> &Vec<(usize, D)> {
        &self.data
    }
}

impl<D: EventData, const N : usize> Display for CompositeData<D,N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{0:?}:{1:?}", self.value, self.data))
    }
}

impl<D: EventData, const N : usize> EventData for CompositeData<D,N> {}









impl<const N: usize, D: EventData> Detector for CompositeDetector<N, D> {
    type TimeType = Real;
    type ValueType = RealArray<N>;
    type DataType = CompositeData<D,N>;

    fn signal(&mut self, time: Real, value: RealArray<N>) -> Option<CompositeEvent<D,N>> {
        let data: Vec<(usize, D)> = self
            .detectors
            .iter_mut()
            .enumerate()
            .map(|(i, detector)| {
                let event = detector.signal(time, value[i])?;
                Some((i,event.take_data()))
            })
            .flatten()
            .collect();
        if data.is_empty() {
            None
        } else {
            Some(CompositeEvent::new(time,CompositeData::<D,N> { value, data }))
        }
    }
}








#[derive(Clone, Debug)]
pub struct CompositeTopOnlyData<D: EventData, const N : usize> {
    value: RealArray<N>,
    index: usize,
    data: D,
}
impl<D: EventData, const N : usize> Default for CompositeTopOnlyData<D,N> {
    fn default() -> Self {
        CompositeTopOnlyData{
            value : [Real::default(); N],
            index : usize::default(),
            data: D::default(),
        }
    }
}
pub type CompositeTopOnlyEvent<D,const N : usize> = Event<Real,CompositeTopOnlyData<D,N>>;

impl<D: EventData, const N : usize> CompositeTopOnlyData<D,N> {
    pub fn get_value(&self) -> &RealArray<N> { &self.value }
    pub fn get_index(&self) -> usize { self.index }
    pub fn get_data(&self) -> &D { &self.data }
}

impl<D: EventData, const N : usize> Display for CompositeTopOnlyData<D,N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{0:?}:{1:?}", self.value, self.data))
    }
}

impl<D: EventData, const N : usize> EventData for CompositeTopOnlyData<D,N> {}



pub struct CompositeTopOnlyDetector<const N: usize, D: EventData> {
    detectors: [ConstituantType<D>; N],
}

impl<const N: usize, D: EventData> CompositeTopOnlyDetector<N, D> {
    pub fn new(detectors: [ConstituantType<D>; N]) -> CompositeTopOnlyDetector<N, D> {
        CompositeTopOnlyDetector::<N, D> { detectors }
    }
}

impl<const N: usize, D: EventData> Detector for CompositeTopOnlyDetector<N, D> {
    type TimeType = Real;
    type ValueType = RealArray<N>;
    type DataType = CompositeTopOnlyData<D,N>;

    fn signal(&mut self, time: Real, value: RealArray<N>) -> Option<CompositeTopOnlyEvent<D,N>> {
        let data: Vec<(usize, D)> = self
            .detectors
            .iter_mut()
            .enumerate()
            .map(|(i, detector)| {
                let event = detector.signal(time, value[i])?;
                Some((i,event.take_data()))
            })
            .flatten()
            .collect();
        if data.is_empty() {
            None
        } else {
            Some(CompositeTopOnlyEvent::new(time,CompositeTopOnlyData::<D,N> { value, index: data[0].0, data: data[0].1.clone() }))
        }
    }
}

use std::array::{from_ref, from_fn};
use std::fmt::Display;

use crate::events::{EventData,TimeValue, SimpleEvent, MultipleEvents, Event, EventWithData};
use crate::window::Window;
use crate::window::smoothing_window::{Stats, SNRSign};
use crate::{Detector, Real, SmoothingWindow, RealArray};



type ConstituantType<E> = Box::<dyn Detector<TimeType = Real, ValueType = Real, EventType = E>>;
//#[derive(Default)]
pub struct CompositeDetector<const N : usize, E : EventWithData> {
    detectors: [ConstituantType<E>;N],
}
    
impl<const N : usize, E : EventWithData> CompositeDetector<N,E> {
    pub fn new(detectors: [ConstituantType<E>;N]) -> CompositeDetector<N,E> {
        CompositeDetector::<N,E> {
            detectors,
        }
    }
}

#[derive(Default, Clone, Debug)]
pub struct CompositeData<D : EventData> {
    index : usize,
    data : D
}

impl<D : EventData> CompositeData<D> {
    pub fn get_index(&self) -> usize { self.index }
    pub fn get_data(&self) -> &D { &self.data }
}

impl<D : EventData> Display for CompositeData<D> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{0}:{1}",self.index,self.data))
    }
}

impl<D : EventData> EventData for CompositeData<D> {
    fn has_influence_at(&self, index: Real) -> bool {
        true
    }

    fn get_intensity_at(&self, index: Real) -> Real {
        Real::default()
    }
}

impl<const N : usize, E : EventWithData> Detector for CompositeDetector<N,E> {
    type TimeType = Real;
    type ValueType = RealArray<N>;
    type EventType = MultipleEvents<SimpleEvent<CompositeData<E::DataType>>>;

    fn signal(&mut self, time : Real, value: RealArray<N>) -> Option<Self::EventType> {
        let events : Vec<SimpleEvent<CompositeData<E::DataType>>> = self.detectors
            .iter_mut()
            .enumerate()
            .map(|(i,detector)| {
                let event = detector.signal(time,value[i])?;
                Some(SimpleEvent::new(
                    event.get_time(),
                    CompositeData{index: i, data: event.take_data()})
                )
            })
            .flatten()
            .collect();
        if events.is_empty() {
            None
        } else {
            Some(MultipleEvents::new(events, time))
        }
    }
}
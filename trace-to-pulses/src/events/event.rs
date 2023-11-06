use crate::trace_iterators::feedback::FeedbackParameter;
use crate::tracedata::{EventData, Temporal, TraceValue};
use std::fmt::Debug;
use std::fmt::Display;

#[derive(Default, Debug, Clone)]
pub struct Event<T, D>
where
    T: Temporal,
    D: EventData,
{
    pub time: T,
    pub data: D,
}
impl<T, D> Event<T, D>
where
    D: EventData,
    T: Temporal,
    D: EventData,
{
    pub fn new(time: T, data: D) -> Self {
        Self { time, data }
    }
    pub fn get_time(&self) -> T {
        self.time
    }
    pub fn get_data(&self) -> &D {
        &self.data
    }
    pub fn get_data_mut(&mut self) -> &mut D {
        &mut self.data
    }
    pub fn take_data(self) -> D {
        self.data
    }
}

impl<T, D> PartialEq for Event<T, D>
where
    T: Temporal,
    D: EventData,
{
    fn eq(&self, other: &Self) -> bool {
        self.time == other.time
    }
}

impl<T, D> Display for Event<T, D>
where
    T: Temporal,
    D: EventData,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{0},{1};", self.time, self.data))
    }
}
/*

impl<D> TraceData for Event<D> where
    D: EventData,
{
    type TimeType = Real;
    type ValueType = Real;
    type DataType = D;

    fn get_time(&self) -> Self::TimeType { self.get_time() }
    fn get_value(&self) -> &Self::ValueType { self.get_data() }
    fn take_value(self) -> Self::ValueType { self.take_data() }
    fn get_data(&self) -> Option<&Self::DataType> { Some(self.get_value()) }
}
 */

#[derive(Default, Debug, Clone)]
pub struct EventWithFeedback<T, D, V>
where
    T: Temporal,
    D: EventData,
    V: TraceValue,
{
    pub event: Event<T, D>,
    pub parameter: FeedbackParameter<V>,
}
impl<T, D, V> EventWithFeedback<T, D, V>
where
    T: Temporal,
    D: EventData,
    V: TraceValue,
{
    pub fn new(time: T, data: D) -> Self {
        Self {
            event: Event::<T, D>::new(time, data),
            ..Default::default()
        }
    }
    pub fn get_time(&self) -> T {
        self.event.get_time()
    }
    pub fn get_data(&self) -> &D {
        self.event.get_data()
    }
    pub fn get_data_mut(&mut self) -> &mut D {
        self.event.get_data_mut()
    }
    pub fn take_data(self) -> D {
        self.event.take_data()
    }
}

impl<T, D, V> PartialEq for EventWithFeedback<T, D, V>
where
    T: Temporal,
    D: EventData,
    V: TraceValue,
{
    fn eq(&self, other: &Self) -> bool {
        self.event == other.event
    }
}

impl<T, D, V> Display for EventWithFeedback<T, D, V>
where
    T: Temporal,
    D: EventData,
    V: TraceValue,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.event, f)
    }
}

/*
impl<D,V> TraceData for EventWithFeedback<D,V> where
    D: EventData,
    V : TraceValue,
{
    type TimeType = Real;
    type ValueType = D;
    type DataType = Empty;

    fn get_time(&self) -> Self::TimeType { self.get_time() }
    fn get_value(&self) -> &Self::ValueType { self.get_data() }
    fn take_value(self) -> Self::ValueType { self.take_data() }
}
*/

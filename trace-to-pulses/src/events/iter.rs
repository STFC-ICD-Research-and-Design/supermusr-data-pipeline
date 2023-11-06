use std::iter::Take;

use crate::detectors::{Assembler, Detector, EventValuedDetector, FeedbackDetector};
use crate::pulse::Pulse;
use crate::trace_iterators::feedback::FeedbackParameter;

use super::event::Event;
use crate::tracedata::{EventData, TraceData, TraceValue};

pub trait EventIterType: Default + Clone {}
#[derive(Default, Clone)]
pub struct Standard;
impl EventIterType for Standard {}
#[derive(Default, Clone)]
pub struct WithFeedback<V>(FeedbackParameter<V>)
where
    V: TraceValue;
impl<V> EventIterType for WithFeedback<V> where V: TraceValue {}
#[derive(Default, Clone)]
pub struct WithTrace;
impl EventIterType for WithTrace {}
#[derive(Default, Clone)]
pub struct WithTracePartition;
impl EventIterType for WithTracePartition {}
#[derive(Default, Clone)]
pub struct WithTraceAndFeedback<V>(FeedbackParameter<V>)
where
    V: TraceValue;
impl<V> EventIterType for WithTraceAndFeedback<V> where V: TraceValue {}

pub struct EventIter<Type, I, D>
where
    Type: EventIterType,
    I: Iterator,
    I::Item: TraceData<TimeType = D::TimeType, ValueType = D::ValueType>,
    D: Detector,
{
    source: I,
    detector: D,
    child: Type,
}

impl<Type, I, D> Clone for EventIter<Type, I, D>
where
    Type: EventIterType,
    I: Iterator + Clone,
    I::Item: TraceData<TimeType = D::TimeType, ValueType = D::ValueType>,
    D: Detector,
{
    fn clone(&self) -> Self {
        Self {
            source: self.source.clone(),
            detector: self.detector.clone(),
            child: self.child.clone(),
        }
    }
}

impl<I, D> Iterator for EventIter<Standard, I, D>
where
    I: Iterator,
    I::Item: TraceData<TimeType = D::TimeType, ValueType = D::ValueType>,
    D: Detector,
{
    type Item = Event<D::TimeType, D::DataType>;
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let trace = self.source.next()?;
            if let Some(event) = self.detector.signal(trace.get_time(), trace.clone_value()) {
                return Some(event);
            }
        }
    }
}

impl<I, D> Iterator for EventIter<WithFeedback<<I::Item as TraceData>::ValueType>, I, D>
where
    I: Iterator + Clone,
    I::Item: TraceData<TimeType = D::TimeType, ValueType = D::ValueType>,
    D: FeedbackDetector,
{
    type Item = Event<D::TimeType, D::DataType>;
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let trace = self.source.next()?;
            self.detector
                .modify_parameter(trace.get_time(), &self.child.0);
            let event = self.detector.signal(trace.get_time(), trace.clone_value());
            if event.is_some() {
                return event;
            }
            if self.detector.is_active() {
                let mut temp_source = self.source.clone();
                let trace = temp_source.next()?;
                while self.detector.is_active() {
                    if let Some(event) = self.detector.signal(trace.get_time(), trace.clone_value())
                    {
                        return Some(event);
                    }
                }
            }
        }
    }
}

impl<I, D> Iterator for EventIter<WithTrace, I, D>
where
    I: Iterator,
    I::Item: TraceData<TimeType = D::TimeType, ValueType = D::ValueType> + std::fmt::Debug,
    D: Detector,
{
    type Item = (D::TimeType, D::ValueType, Option<D::DataType>);
    fn next(&mut self) -> Option<Self::Item> {
        let trace = self.source.next()?;
        let event = self.detector.signal(trace.get_time(), trace.clone_value());
        Some((
            trace.get_time(),
            trace.take_value(),
            event.map(|e| e.take_data()),
        ))
    }
}

impl<I, D> Iterator for EventIter<WithTraceAndFeedback<<I::Item as TraceData>::ValueType>, I, D>
where
    I: Iterator,
    I::Item: TraceData<TimeType = D::TimeType, ValueType = D::ValueType, DataType = D::DataValueType>
        + std::fmt::Debug,
    D: EventValuedDetector + FeedbackDetector,
{
    type Item = Event<D::TimeType, D::DataType>;
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let trace = self.source.next()?;
            let time = trace.get_time();
            let value = trace.clone_value();
            self.detector.signal(time, value);
            self.detector.modify_parameter(time, &self.child.0);
            if let Some(data) = trace.take_data() {
                if let Some(event) = self.detector.on_event(data.make_event(time)) {
                    return Some(event);
                }
            }
        }
        //Some((trace.get_time(), trace.take_value(), event.map(|e|e.take_data())))
    }
}

impl<'a, I, D> Iterator for EventIter<WithTracePartition, I, D>
where
    I: Iterator + Clone,
    I::Item: TraceData<TimeType = D::TimeType, ValueType = D::ValueType> + std::fmt::Debug + 'a,
    D: Detector,
{
    type Item = (Option<Event<D::TimeType, D::DataType>>, Take<I>);

    fn next(&mut self) -> Option<Self::Item> {
        let mut n = usize::default();
        let start = self.source.clone();
        for trace in &mut self.source {
            n += 1;
            let time = trace.get_time();
            let value = trace.clone_value();
            if let Some(event) = self.detector.signal(time, value) {
                return Some((Some(event), start.take(n)));
            }
        }
        Some((None, start.take(n)))
        //Some((trace.get_time(), trace.take_value(), event.map(|e|e.take_data())))
    }
}

pub trait EventFilter<I, D>
where
    I: Iterator,
    I::Item: TraceData<TimeType = D::TimeType, ValueType = D::ValueType>,
    D: Detector,
{
    fn events(self, detector: D) -> EventIter<Standard, I, D>;
    fn trace_with_events(self, detector: D) -> EventIter<WithTrace, I, D>;
    fn trace_partition_by_events(self, detector: D) -> EventIter<WithTracePartition, I, D>;
}

impl<I, D> EventFilter<I, D> for I
where
    I: Iterator,
    I::Item: TraceData<TimeType = D::TimeType, ValueType = D::ValueType>,
    D: Detector,
{
    fn events(self, detector: D) -> EventIter<Standard, I, D> {
        EventIter {
            source: self,
            detector,
            child: Standard,
        }
    }

    fn trace_with_events(self, detector: D) -> EventIter<WithTrace, I, D> {
        EventIter {
            source: self,
            detector,
            child: WithTrace,
        }
    }

    fn trace_partition_by_events(self, detector: D) -> EventIter<WithTracePartition, I, D> {
        EventIter {
            source: self,
            detector,
            child: WithTracePartition,
        }
    }
}

pub trait EventsWithFeedbackFilter<I, D>
where
    I: Iterator,
    I::Item: TraceData<TimeType = D::TimeType, ValueType = D::ValueType>,
    D: Detector,
{
    fn events_with_feedback(
        self,
        parameter: FeedbackParameter<<I::Item as TraceData>::ValueType>,
        detector: D,
    ) -> EventIter<WithFeedback<<I::Item as TraceData>::ValueType>, I, D>;
    fn events_from_events_with_feedback(
        self,
        parameter: FeedbackParameter<<I::Item as TraceData>::ValueType>,
        detector: D,
    ) -> EventIter<WithTraceAndFeedback<<I::Item as TraceData>::ValueType>, I, D>;
}

impl<I, D> EventsWithFeedbackFilter<I, D> for I
where
    I: Iterator,
    I::Item: TraceData<TimeType = D::TimeType, ValueType = D::ValueType>,
    D: FeedbackDetector,
{
    fn events_with_feedback(
        self,
        parameter: FeedbackParameter<<I::Item as TraceData>::ValueType>,
        detector: D,
    ) -> EventIter<WithFeedback<<I::Item as TraceData>::ValueType>, I, D> {
        EventIter {
            source: self,
            detector,
            child: WithFeedback(parameter),
        }
    }

    fn events_from_events_with_feedback(
        self,
        parameter: FeedbackParameter<<I::Item as TraceData>::ValueType>,
        detector: D,
    ) -> EventIter<WithTraceAndFeedback<<I::Item as TraceData>::ValueType>, I, D> {
        EventIter {
            source: self,
            detector,
            child: WithTraceAndFeedback(parameter),
        }
    }
}

#[derive(Clone)]
pub struct AssemblerIter<I, A>
where
    A: Assembler,
    I: Iterator<
            Item = Event<
                <A::DetectorType as Detector>::TimeType,
                <A::DetectorType as Detector>::DataType,
            >,
        > + Clone,
{
    source: I,
    assembler: A,
}
impl<I, A> Iterator for AssemblerIter<I, A>
where
    A: Assembler,
    I: Iterator<
            Item = Event<
                <A::DetectorType as Detector>::TimeType,
                <A::DetectorType as Detector>::DataType,
            >,
        > + Clone,
{
    type Item = Pulse;

    fn next(&mut self) -> Option<Pulse> {
        for event in &mut self.source {
            let pulse = self.assembler.assemble_pulses(event);
            if pulse.is_some() {
                return pulse;
            }
        }
        None
    }
}

pub trait AssembleFilter<I, A>
where
    A: Assembler,
    I: Iterator<
            Item = Event<
                <A::DetectorType as Detector>::TimeType,
                <A::DetectorType as Detector>::DataType,
            >,
        > + Clone,
{
    fn assemble(self, assembler: A) -> AssemblerIter<I, A>;
}

impl<I, A> AssembleFilter<I, A> for I
where
    A: Assembler,
    I: Iterator<
            Item = Event<
                <A::DetectorType as Detector>::TimeType,
                <A::DetectorType as Detector>::DataType,
            >,
        > + Clone,
{
    fn assemble(self, assembler: A) -> AssemblerIter<I, A> {
        AssemblerIter {
            source: self,
            assembler,
        }
    }
}

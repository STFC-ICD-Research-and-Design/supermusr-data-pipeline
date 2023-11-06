use crate::{
    detectors::pulse_detector::{PulseEvent, PulseModel},
    tracedata::TraceData,
    Real,
};

fn sum_event_energy_at<Model: PulseModel>(events: &[PulseEvent<Model>], time: Real) -> Real {
    let sum = events
        .iter()
        .map(
            |event| event.get_data().get_effective_value_at(time), /*if event.get_data()
                                                                       .get_standard_deviation()
                                                                       .unwrap_or_default() >= 0.
                                                                   {
                                                                   } else {
                                                                       0.
                                                                   }*/
        )
        .sum::<Real>();
    sum
}

#[derive(Clone)]
pub struct SimulationIter<'a, I, Model>
where
    I: Iterator,
    I::Item: TraceData<TimeType = Real, ValueType = Real>,
    Model: PulseModel,
{
    source: I,
    events: &'a [PulseEvent<Model>],
}

impl<'a, I, Model> Iterator for SimulationIter<'a, I, Model>
where
    I: Iterator,
    I::Item: TraceData<TimeType = Real, ValueType = Real>,
    Model: PulseModel,
{
    type Item = (Real, Real);

    fn next(&mut self) -> Option<Self::Item> {
        let trace = self.source.next()?;
        Some((
            trace.get_time(),
            sum_event_energy_at(self.events, trace.get_time()),
        ))
    }
}

#[derive(Clone)]
pub struct EvaluationIter<'a, I, Model>
where
    I: Iterator,
    I::Item: TraceData<TimeType = Real, ValueType = Real>,
    Model: PulseModel,
{
    source: I,
    events: &'a [PulseEvent<Model>],
}
impl<'a, I, Model> Iterator for EvaluationIter<'a, I, Model>
where
    I: Iterator,
    I::Item: TraceData<TimeType = Real, ValueType = Real>,
    Model: PulseModel,
{
    type Item = (Real, Real, Real);

    fn next(&mut self) -> Option<Self::Item> {
        let trace = self.source.next()?;
        Some((
            trace.get_time(),
            trace.clone_value(),
            (trace.get_value() - sum_event_energy_at(self.events, trace.get_time())).abs(),
        ))
    }
}

pub trait ToTrace<'a, I, Model>
where
    I: Iterator<Item = (Real, Real)>,
    Model: PulseModel,
{
    fn to_trace(self, events: &'a [PulseEvent<Model>]) -> SimulationIter<'a, I, Model>;
    fn evaluate_events(self, events: &'a [PulseEvent<Model>]) -> EvaluationIter<'a, I, Model>;
}

impl<'a, I, Model> ToTrace<'a, I, Model> for I
where
    I: Iterator<Item = (Real, Real)> + Clone,
    Model: PulseModel,
{
    fn to_trace(self, events: &'a [PulseEvent<Model>]) -> SimulationIter<'a, I, Model> {
        SimulationIter {
            source: self,
            events,
        }
    }
    fn evaluate_events(self, events: &'a [PulseEvent<Model>]) -> EvaluationIter<'a, I, Model> {
        EvaluationIter {
            source: self,
            events,
        }
    }
}

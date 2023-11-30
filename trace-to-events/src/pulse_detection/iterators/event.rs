use super::{Assembler, Detector, Pulse, TracePoint};

#[derive(Clone)]
pub(crate) struct EventIter<I, D>
where
    I: Iterator<Item = D::TracePointType>,
    D: Detector,
{
    source: I,
    detector: D,
}

impl<I, D> Iterator for EventIter<I, D>
where
    I: Iterator<Item = D::TracePointType>,
    D: Detector,
{
    type Item = D::EventPointType;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let trace = self.source.next()?;
            if let Some(event) = self.detector.signal(trace.get_time(), trace.clone_value()) {
                return Some(event);
            }
        }
    }
}

pub(crate) trait EventFilter<I, D>
where
    I: Iterator,
    I: Iterator<Item = D::TracePointType>,
    D: Detector,
{
    fn events(self, detector: D) -> EventIter<I, D>;
}

impl<I, D> EventFilter<I, D> for I
where
    I: Iterator,
    I: Iterator<Item = D::TracePointType>,
    D: Detector,
{
    fn events(self, detector: D) -> EventIter<I, D> {
        EventIter {
            source: self,
            detector,
        }
    }
}

#[derive(Clone)]
pub(crate) struct AssemblerIter<I, A>
where
    A: Assembler,
    I: Iterator<Item = <A::DetectorType as Detector>::EventPointType> + Clone,
{
    source: I,
    assembler: A,
}
impl<I, A> Iterator for AssemblerIter<I, A>
where
    A: Assembler,
    I: Iterator<Item = <A::DetectorType as Detector>::EventPointType> + Clone,
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

pub(crate) trait AssembleFilter<I, A>
where
    A: Assembler,
    I: Iterator<Item = <A::DetectorType as Detector>::EventPointType> + Clone,
{
    fn assemble(self, assembler: A) -> AssemblerIter<I, A>;
}

impl<I, A> AssembleFilter<I, A> for I
where
    A: Assembler,
    I: Iterator<Item = <A::DetectorType as Detector>::EventPointType> + Clone,
{
    fn assemble(self, assembler: A) -> AssemblerIter<I, A> {
        AssemblerIter {
            source: self,
            assembler,
        }
    }
}

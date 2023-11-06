use crate::tracedata::TraceData;

pub trait TraceIterType: Clone {}

#[derive(Clone)]
pub struct TraceIter<Type, I>
where
    Type: TraceIterType,
    I: Iterator,
    I::Item: TraceData,
{
    pub(super) child: Type,
    pub(super) source: I,
}

impl<Type, I> TraceIter<Type, I>
where
    Type: TraceIterType,
    I: Iterator,
    I::Item: TraceData,
{
    pub fn new(child: Type, source: I) -> TraceIter<Type, I> {
        TraceIter { child, source }
    }
}

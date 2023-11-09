use super::super::datatype::TracePoint;

pub trait TraceIterType: Clone {}

#[derive(Clone)]
pub(crate) struct TraceIter<Type, I>
where
    Type: TraceIterType,
    I: Iterator,
    I::Item: TracePoint,
{
    pub(super) child: Type,
    pub(super) source: I,
}

impl<Type, I> TraceIter<Type, I>
where
    Type: TraceIterType,
    I: Iterator,
    I::Item: TracePoint,
{
    pub fn new(child: Type, source: I) -> TraceIter<Type, I> {
        TraceIter { child, source }
    }
}

/// Feedback allows a filter, which use a FeedbackDetector, to affect change in the stream
/// at an earlier point in the algorithm.
/// #Example
///```rust
/// let events = trace.iter()
///     .enumerate()
///     .map(make_real_enumerate)
///     .start_feedback(|x,y|x - y)     // This converts the iterator into a FeedbackIter
///     .window(SmoothedWindow(4))      // This window works as it would without the previous line
///     .events_with_feedback(          // This allows PulseDetector to change the stream
///         PulseDetector(              // at the point where ```rust start_feedback``` occurs by setting
///             ChangeDetector(0.5),1   // the value of y in the function given to ```rust start_feedback```.
///     ))
///```
use std::{
    cell::Cell,
    fmt::{self, Display, Formatter},
    rc::Rc,
};

use crate::tracedata::{TraceData, TraceValue};

use super::iter::{TraceIter, TraceIterType};

/// This is a wrapper for the pointer to the feedback parameter.
/// Instances of this can be cloned and passed around and modified.
#[derive(Default, Debug)]
pub struct FeedbackParameter<V>(pub Rc<Cell<V::FeedbackType>>)
where
    V: TraceValue;

/// #Methods
/// - new(initial : V): Creates a new Cell and Rc in which to store the feedback parameter.
/// - set(value : V): Dereferences the Rc and changes the value of the cell.
impl<V> FeedbackParameter<V>
where
    V: TraceValue,
{
    pub fn new() -> Self {
        Self(Rc::new(Cell::new(V::FeedbackType::default())))
    }
    pub fn set(&self, value: V::FeedbackType) {
        (*self.0).set(value);
    }
}

/// Clone creates a new instance and clones the Rc contained within it.
/// This creates a new pointer to the Cell containing the parameter.
/// Note this does not clone the parameter itself.
impl<V> Clone for FeedbackParameter<V>
where
    V: TraceValue,
{
    fn clone(&self) -> Self {
        Self(Rc::clone(&self.0))
    }
}

impl<V> Display for FeedbackParameter<V>
where
    V: TraceValue,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!("{self:?}"))
    }
}

pub struct FeedbackFunction<V>
where
    V: TraceValue,
{
    _parameter: FeedbackParameter<V>,
    _modifier: fn(&V::ContentType, &V::FeedbackType) -> V,
}

impl<V> FeedbackFunction<V>
where
    V: TraceValue,
{
    fn _new(modifier: fn(&V::ContentType, &V::FeedbackType) -> V) -> Self {
        Self {
            _parameter: FeedbackParameter::new(),
            _modifier: modifier,
        }
    }
    /// Clone creates a new instance and clones the Rc contained within it.
    /// This creates a new pointer to the Cell containing the parameter.
    /// Note this does not clone the parameter itself.
    fn _clone_parameter(&self) -> FeedbackParameter<V> {
        self._parameter.clone()
    }
}

#[derive(Clone)]
pub struct Feedback<V>
where
    V: TraceValue,
{
    parameter: FeedbackParameter<V>,
    modifier: fn(&V::ContentType, &V::FeedbackType) -> V,
}

impl<V> Feedback<V>
where
    V: TraceValue,
{
    fn new(
        parameter: FeedbackParameter<V>,
        modifier: fn(&V::ContentType, &V::FeedbackType) -> V,
    ) -> Self {
        Self {
            parameter,
            modifier,
        }
    }
}

impl<V> TraceIterType for Feedback<V> where V: TraceValue {}

impl<I> Iterator for TraceIter<Feedback<<I::Item as TraceData>::ValueType>, I>
where
    I: Iterator,
    I::Item: TraceData,
{
    type Item = (
        <I::Item as TraceData>::TimeType,
        <I::Item as TraceData>::ValueType,
    );

    fn next(&mut self) -> Option<Self::Item> {
        let val = self.source.next()?;
        let time = val.get_time();
        let value =
            (self.child.modifier)(val.get_value().get_value(), &self.child.parameter.0.get());

        // LOG
        //log::info!("Applied correction of {0:?}", self.parameter.0.get());
        //let r = Rc::strong_count(&self.parameter.0);
        // LOG
        //log::info!("Number of references: {0:?}",r);
        Some((time, value)) //
    }
}
/*
/// This is the simplest non-trivial implementation which includes flexible feedback
/// The first and second elements are the time and value respectively,
/// whilst the third contains an OptFeedParam instance, essentially a pointer to the
/// feedback parameter which can be modified.
/// The feedback parameter pointer is accessed by calling
/// ```rust
/// get_parameter()
/// ```
impl<X,Y> TraceData for (X,Y) where X : Temporal, Y: TraceValue {
    type TimeType = X;
    type ValueType = Y;
    type DataType = Empty;

    fn get_time(&self) -> Self::TimeType { self.0 }
    fn get_value(&self) -> &Self::ValueType { &self.1 }
    fn take_value(self) -> Self::ValueType { self.1 }
}

/// This is the simplest non-trivial implementation which includes flexible feedback
/// The first and second elements are the time and value respectively,
/// whilst the third contains an OptFeedParam instance, essentially a pointer to the
/// feedback parameter which can be modified.
/// The feedback parameter pointer is accessed by calling
/// ```rust
/// get_parameter()
/// ```
impl<X,Y,D> TraceData for (X,Y,Option<D>) where X : Temporal, Y: TraceValue, D : TraceEventData {
    type TimeType = X;
    type ValueType = Y;
    type DataType = D;

    fn get_time(&self) -> Self::TimeType { self.0 }
    fn get_value(&self) -> &Self::ValueType { &self.1 }
    fn take_value(self) -> Self::ValueType { self.1 }

    fn get_data(&self) -> Option<&Self::DataType> { self.3.as_ref() }
}
 */

type FeedbackValueType<I> = <<I as Iterator>::Item as TraceData>::ValueType;

type FeedbackFunctionTypeValue<I> = <FeedbackValueType<I> as TraceValue>::ContentType;
type FeedbackFunctionTypeFeedback<I> = <FeedbackValueType<I> as TraceValue>::FeedbackType;
type FeedbackFunctionType<I> =
    fn(&FeedbackFunctionTypeValue<I>, &FeedbackFunctionTypeFeedback<I>) -> FeedbackValueType<I>;

/// This trait is implemented for any iterator that contains TraceData.
/// #Methods
/// - start_feedback(modifier): from hereon, all trace values have the modifier function
/// applied to it, where modifier has the signature
///```rust
/// modifier: fn(&ValueType, &ParameterType)->ValueType
/// ```
/// Note ValueType and ParameterType refer to the associated types of the TraceData trait referred to above.
pub trait FeedbackFilter<I>
where
    I: Iterator,
    I::Item: TraceData,
{
    fn start_feedback(
        self,
        parameter: &FeedbackParameter<FeedbackValueType<I>>,
        modifier: FeedbackFunctionType<I>,
    ) -> TraceIter<Feedback<FeedbackValueType<I>>, I>;
}

impl<I> FeedbackFilter<I> for I
where
    I: Iterator,
    I::Item: TraceData,
{
    fn start_feedback(
        self,
        parameter: &FeedbackParameter<FeedbackValueType<I>>,
        modifier: FeedbackFunctionType<I>,
    ) -> TraceIter<Feedback<FeedbackValueType<I>>, I> {
        TraceIter::new(Feedback::new(parameter.clone(), modifier), self)
    }
}

/// This trait can be implemented for any iterator whose items are of the form:
/// ```rust
/// (X,Y,OptFeedParam<Z>)
/// ```,
/// that is anywhere iterator after start_feedback has been called,
/// and before events_with_feedback has been called.
/// #Methods
/// - end_feedback(): removes the feedback parameter from the data stream.
/// This is useful if you want to implement more than one non-intersecting feedback parameter

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Real;
    use common::Intensity;

    #[test]
    fn sample_data_zero() {
        let feedback_parameter = FeedbackParameter::new();
        let input: Vec<Intensity> = vec![1, 6, 2, 1, 3, 1, 0];
        let output: Vec<Real> = input
            .into_iter()
            .enumerate()
            .map(|(i, v)| (i as Real, v as Real))
            .start_feedback(&feedback_parameter, |x, &y: &Real| x + y)
            .map(|(_, x)| x)
            .collect();

        assert_eq!(output[0], 1.);
        assert_eq!(output[1], 6.);
        assert_eq!(output[2], 2.);
        assert_eq!(output[3], 1.);
        assert_eq!(output[4], 3.);
        assert_eq!(output[5], 1.);
        assert_eq!(output[6], 0.);
    }

    #[test]
    fn sample_data_update() {
        let feedback_parameter = FeedbackParameter::new();
        let input: Vec<Intensity> = vec![1, 6, 2, 1, 3, 1, 0];
        let output: Vec<Real> = input
            .into_iter()
            .enumerate()
            .map(|(i, v)| (i as Real, v as Real))
            .start_feedback(&feedback_parameter, |x, &y: &Real| x + y)
            .map(|(_, x)| {
                feedback_parameter.set(2.);
                x
            })
            .collect::<Vec<_>>();

        assert_eq!(output[0], 1.);
        assert_eq!(output[1], 8.);
        assert_eq!(output[2], 4.);
        assert_eq!(output[3], 3.);
        assert_eq!(output[4], 5.);
        assert_eq!(output[5], 3.);
        assert_eq!(output[6], 2.);
    }
}

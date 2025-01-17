use super::{eventdata::Empty, EventData, Temporal, TraceValue};

/// An abstraction of the types that are processed by the various filters
/// To implement TracePoint a type must contain time data, a value,
/// and a parameter (which is used for applying feedback).
pub(crate) trait TracePoint: Clone {
    /// The type which represents the time of the data point.
    /// This should be trivially copyable (usually a scalar).
    type Time: Temporal;

    /// The type which contains the value of the data point.
    type Value: TraceValue;

    type Data: EventData;

    /// Returns the time of the data point.
    fn get_time(&self) -> Self::Time;

    /// Returns an immutable reference to the value of the data point.
    fn get_value(&self) -> &Self::Value;

    /// Take ownership of a clone of the value without destructing the data point.
    fn clone_value(&self) -> Self::Value {
        self.get_value().clone()
    }
}

/// This is the most basic non-trivial TraceData type.
/// The first element is the TimeType and the second the ValueType.
/// The ParameterType is the same as the ValueType, but as there is no
/// implementation of ```rust get_parameter()```, the type does not support
/// feedback.
impl<X, Y> TracePoint for (X, Y)
where
    X: Temporal,
    Y: TraceValue,
{
    type Time = X;
    type Value = Y;
    type Data = Empty;

    fn get_time(&self) -> Self::Time {
        self.0
    }

    fn get_value(&self) -> &Self::Value {
        &self.1
    }

    fn clone_value(&self) -> Self::Value {
        self.get_value().clone()
    }
}

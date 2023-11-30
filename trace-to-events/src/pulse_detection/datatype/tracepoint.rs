use super::{eventdata::Empty, EventData, Temporal, TraceValue};

/// An abstraction of the types that are processed by the various filters
/// To implement TracePoint a type must contain time data, a value,
/// and a parameter (which is used for applying feedback).
/// *Associated Types
/// - TimeType: the type which represents the time of the data point.
/// This should be trivially copyable (usually a scalar).
/// - ValueType: the type which contains the value of the data point.
/// * Methods
/// - get_time(): returns the time of the data point.
/// - get_value(): returns an immutable reference to the value of the data point.
/// - take_value(): destructs the data point and gives the caller ownership of the value.
/// - clone_value(): allows the user to take ownership of a clone of the value without
/// destructing the data point.
pub(crate) trait TracePoint: Clone {
    type TimeType: Temporal;
    type ValueType: TraceValue;
    type DataType: EventData;

    fn get_time(&self) -> Self::TimeType;
    fn get_value(&self) -> &Self::ValueType;
    fn take_value(self) -> Self::ValueType;

    fn clone_value(&self) -> Self::ValueType {
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
    type TimeType = X;
    type ValueType = Y;
    type DataType = Empty;

    fn get_time(&self) -> Self::TimeType {
        self.0
    }

    fn get_value(&self) -> &Self::ValueType {
        &self.1
    }

    fn take_value(self) -> Self::ValueType {
        self.1
    }

    fn clone_value(&self) -> Self::ValueType {
        self.get_value().clone()
    }
}

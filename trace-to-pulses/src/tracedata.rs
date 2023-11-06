use std::fmt::{Debug, Display, Formatter, Result};

use common::Intensity;

use crate::{events::event::Event, Real, TraceArray, TracePair};

pub trait Temporal: Default + Copy + Debug + Display + PartialEq {}
impl Temporal for Intensity {}
impl Temporal for Real {}

pub trait TraceValue: Default + Clone + Debug + Display {
    type ContentType: Default + Clone + Debug + Display;
    type FeedbackType: Default + Copy + Debug + Display;

    fn get_value(&self) -> &Self::ContentType;
    fn take_value(self) -> Self::ContentType;
}
impl TraceValue for Real {
    type ContentType = Real;
    type FeedbackType = Real;

    fn get_value(&self) -> &Self::ContentType {
        self
    }
    fn take_value(self) -> Self::ContentType {
        self
    }
}
impl<const N: usize, T: TraceValue + Copy> TraceValue for TraceArray<N, T> {
    type ContentType = TraceArray<N, T>;
    type FeedbackType = TraceArray<N, T>;

    fn get_value(&self) -> &Self::ContentType {
        self
    }
    fn take_value(self) -> Self::ContentType {
        self
    }
}
impl<T1: TraceValue + Copy, T2: TraceValue + Copy> TraceValue for TracePair<T1, T2> {
    type ContentType = TracePair<T1, T2>;
    type FeedbackType = TracePair<T1::FeedbackType, T2::FeedbackType>;

    fn get_value(&self) -> &Self::ContentType {
        self
    }
    fn take_value(self) -> Self::ContentType {
        self
    }
}

pub trait EventData: Default + Clone + Debug + Display {
    fn make_event<T>(self, time: T) -> Event<T, Self>
    where
        T: Temporal,
    {
        Event::<T, Self> { time, data: self }
    }
}

#[derive(Default, Clone, Copy, Debug, PartialEq)]
pub struct Empty {}
impl Display for Empty {
    fn fmt(&self, _f: &mut Formatter<'_>) -> Result {
        Ok(())
    }
}
impl EventData for Empty {}

//impl<T> Eventy for T where T : Scalar {}

/// An abstraction of the types that are processed by the various filters
/// To implement TraceData a type must contain time data, a value,
/// and a parameter (which is used for applying feedback).
/// Optionally, the type can contain a data type, into which event data can be encoded.
/// *Associated Types
/// - TimeType: the type which represents the time of the data point.
/// This should be trivially copyable (usually a scalar).
/// - ValueType: the type which contains the value of the data point.
/// - ParameterType: when the data point expects to implement feedback
/// from later filters, this type represents the data needed to apply it.
/// Often this is the same as the ValueType but doesn't need to be.
/// * Methods
/// - get_time(): returns the time of the data point.
/// - get_value(): returns an immutable reference to the value of the data point.
/// - take_value(): destructs the data point and gives the caller ownership of the value.
/// - clone_value(): allows the user to take ownership of a clone of the value without
/// destructing the data point.
/// - get_parameter(): returns an OptFeedParam instance which abstracts the feedback
/// parameter. If feedback is not intended for this type, this method has a default implementation
/// which should be used.
pub trait TraceData: Clone {
    type TimeType: Temporal;
    type ValueType: TraceValue;
    type DataType: EventData;

    fn get_time(&self) -> Self::TimeType;
    fn get_value(&self) -> &Self::ValueType;
    fn take_value(self) -> Self::ValueType;

    fn clone_value(&self) -> Self::ValueType {
        self.get_value().clone()
    }

    fn get_data(&self) -> Option<&Self::DataType> {
        None
    }
    fn take_data(self) -> Option<Self::DataType> {
        None
    }
}

/// This is the most basic non-trivial TraceData type.
/// The first element is the TimeType and the second the ValueType.
/// The ParameterType is the same as the ValueType, but as there is no
/// implementation of ```rust get_parameter()```, the type does not support
/// feedback.
impl<X, Y> TraceData for (X, Y)
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
}

/// This is the most basic non-trivial TraceData type.
/// The first element is the TimeType and the second the ValueType.
/// The ParameterType is the same as the ValueType, but as there is no
/// implementation of ```rust get_parameter()```, the type does not support
/// feedback.
impl<X, Y, D> TraceData for (X, Y, Option<D>)
where
    X: Temporal,
    Y: TraceValue,
    D: EventData,
{
    type TimeType = X;
    type ValueType = Y;
    type DataType = D;

    fn get_time(&self) -> Self::TimeType {
        self.0
    }
    fn get_value(&self) -> &Self::ValueType {
        &self.1
    }
    fn take_value(self) -> Self::ValueType {
        self.1
    }
    fn get_data(&self) -> Option<&Self::DataType> {
        self.2.as_ref()
    }
    fn take_data(self) -> Option<Self::DataType> {
        self.2
    }
}

#[derive(Default, Clone, Debug)]
pub struct Stats {
    pub value: Real,
    pub mean: Real,
    pub variance: Real,
}
impl From<Real> for Stats {
    fn from(value: Real) -> Self {
        Stats {
            value,
            mean: value,
            variance: 0.,
        }
    }
}
impl Display for Stats {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(
            f,
            "({}:?, {}:?, {}:?)",
            self.value, self.mean, self.variance
        )
    }
}

pub mod extract {
    use super::*;
    pub fn mean<D>(trace: D) -> Real
    where
        D: TraceData<ValueType = Stats>,
    {
        trace.get_value().mean
    }
    pub fn enumerated_mean<T, D>(trace: D) -> (T, Real)
    where
        D: TraceData<TimeType = T, ValueType = Stats>,
    {
        (trace.get_time(), trace.get_value().mean)
    }
    pub fn enumerated_variance<T, D>(trace: D) -> (T, Real)
    where
        D: TraceData<TimeType = T, ValueType = Stats>,
    {
        (trace.get_time(), trace.get_value().variance)
    }
    pub fn enumerated_standard_deviation<T, D>(trace: D) -> (T, Real)
    where
        D: TraceData<TimeType = T, ValueType = Stats>,
    {
        (trace.get_time(), trace.get_value().variance.sqrt())
    }
    pub fn enumerated_normalised_mean<T, D>(trace: D) -> (T, Real)
    where
        D: TraceData<TimeType = T, ValueType = Stats>,
    {
        if trace.get_value().variance == 0. {
            (trace.get_time(), trace.get_value().mean)
        } else {
            (
                trace.get_time(),
                trace.get_value().mean / trace.get_value().variance.sqrt(),
            )
        }
    }
    pub fn enumerated_normalised_value<T, D>(trace: D) -> (T, Real)
    where
        D: TraceData<TimeType = T, ValueType = Stats>,
    {
        if trace.get_value().variance == 0. {
            (trace.get_time(), trace.get_value().value)
        } else {
            (
                trace.get_time(),
                (trace.get_value().value - trace.get_value().mean)
                    / trace.get_value().variance.sqrt(),
            )
        }
    }
}
pub mod operation {
    use super::*;

    pub fn add_real(&value: &Real, param: &Real) -> Real {
        value + param
    }
    pub fn shift_stats(
        &Stats {
            value,
            mean,
            variance,
        }: &Stats,
        param: &Real,
    ) -> Stats {
        Stats {
            value: value + param,
            mean: mean + param,
            variance,
        }
    }
}

#[derive(Default, Clone, PartialEq)]
pub enum SNRSign {
    Pos,
    Neg,
    #[default]
    Zero,
}
impl Stats {
    pub fn signal_over_noise_sign(&self, threshold: Real) -> SNRSign {
        if (self.value - self.mean).powi(2) >= self.variance * threshold.powi(2) {
            if (self.value - self.mean).is_sign_positive() {
                SNRSign::Pos
            } else {
                SNRSign::Neg
            }
        } else {
            SNRSign::Zero
        }
    }
    pub fn get_normalized_value(&self) -> Real {
        (self.value - self.mean).powi(2) / self.variance.sqrt()
    }
    pub fn shift(&mut self, delta: Real) {
        self.value += delta;
        self.mean += delta;
    }
}

impl TraceValue for Stats {
    type ContentType = Stats;
    type FeedbackType = Real;

    fn get_value(&self) -> &Self::ContentType {
        self
    }
    fn take_value(self) -> Self::ContentType {
        self
    }
}

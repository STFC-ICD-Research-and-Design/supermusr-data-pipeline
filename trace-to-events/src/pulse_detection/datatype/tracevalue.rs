use std::{
    fmt::{Debug, Display, Formatter, Result},
    ops::{Index, IndexMut},
};

use super::Real;

/// An abstraction of the types that represent values processed by the various filters
/// This differs from the TraceData type in that TraceData
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
pub trait TraceValue: Default + Clone + Debug + Display {
    type ContentType: Default + Clone + Debug + Display;

    fn get_value(&self) -> &Self::ContentType;
    fn take_value(self) -> Self::ContentType;
}
impl TraceValue for Real {
    type ContentType = Real;

    fn get_value(&self) -> &Self::ContentType {
        self
    }
    fn take_value(self) -> Self::ContentType {
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TraceArray<const N: usize, T>(pub [T; N])
where
    T: TraceValue;

impl<const N: usize, T> TraceArray<N, T>
where
    T: TraceValue,
{
    pub fn new(value: [T; N]) -> Self {
        Self(value)
    }
}

impl<const N: usize, T> Default for TraceArray<N, T>
where
    T: TraceValue + Copy,
{
    fn default() -> Self {
        Self([T::default(); N])
    }
}

impl<const N: usize, T> Display for TraceArray<N, T>
where
    T: TraceValue,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let TraceArray(array) = self;
        for val in array.iter().take(N - 1) {
            write!(f, "{val},")?;
        }
        write!(f, "{0}", array[N - 1])
    }
}
impl<const N: usize, T> Index<usize> for TraceArray<N, T>
where
    T: TraceValue,
{
    type Output = T;

    fn index(&self, idx: usize) -> &T {
        &self.0[idx]
    }
}
impl<const N: usize, T> IndexMut<usize> for TraceArray<N, T>
where
    T: TraceValue,
{
    fn index_mut(&mut self, idx: usize) -> &mut T {
        &mut self.0[idx]
    }
}
impl<const N: usize, T: TraceValue + Copy> TraceValue for TraceArray<N, T> {
    type ContentType = TraceArray<N, T>;

    fn get_value(&self) -> &Self::ContentType {
        self
    }
    fn take_value(self) -> Self::ContentType {
        self
    }
}

pub type RealArray<const N: usize> = TraceArray<N, Real>;

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

impl TraceValue for Stats {
    type ContentType = Stats;

    fn get_value(&self) -> &Self::ContentType {
        self
    }
    fn take_value(self) -> Self::ContentType {
        self
    }
}

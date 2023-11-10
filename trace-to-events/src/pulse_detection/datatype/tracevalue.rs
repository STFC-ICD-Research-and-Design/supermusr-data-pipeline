use std::{
    fmt::{Debug, Display, Formatter, Result},
    ops::{Index, IndexMut},
};

use super::Real;

/// An abstraction of the types that represent values processed by the various filters
/// This differs from the TracePoint type in that TracePoint must represent a time value,
/// whereas TraceValue is time-agnostic.
/// To implement TraceValue a type must contain time data, a value,
/// and a parameter (which is used for applying feedback).
/// *Associated Types
/// - ValueType: the type which contains the value of the data point.
/// * Methods
/// - get_value(): returns an immutable reference to the value of the data point.
/// - take_value(): destructs the data point and gives the caller ownership of the value.
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

/// This type allows the use of static arrays of TraceValue types as TraceValues
/// that can be used in the pipeline.
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

/// In practice arrays of Real types are mostly used.
pub type RealArray<const N: usize> = TraceArray<N, Real>;


/// This type allows contains descriptive statistical data.
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

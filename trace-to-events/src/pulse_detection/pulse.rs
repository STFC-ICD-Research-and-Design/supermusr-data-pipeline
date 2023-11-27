use std::fmt::Display;

use super::Real;
use super::RealArray;

#[derive(Default, Clone, Debug, PartialEq)]
pub(crate) struct TimeValue<T>
where
    T: Default + Clone,
{
    pub(crate) time: Real,
    pub(crate) value: T,
}

impl<T> Display for TimeValue<T>
where
    T: Default + Clone + Copy + Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{0},{1}", self.time, self.value))
    }
}

#[derive(Default, Clone, Debug)]
pub(crate) struct TimeValueOptional<T>
where
    T: Default + Clone,
{
    pub(crate) time: Option<Real>,
    pub(crate) value: Option<T>,
}

impl<T> From<TimeValue<T>> for TimeValueOptional<T>
where
    T: Default + Clone + Copy + Display,
{
    fn from(source: TimeValue<T>) -> Self {
        TimeValueOptional {
            time: Some(source.time),
            value: Some(source.value),
        }
    }
}

impl<T> Display for TimeValueOptional<T>
where
    T: Default + Clone + Copy + Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "{0},{1}",
            self.time.unwrap_or_default(),
            self.value.unwrap_or_default()
        ))
    }
}

#[derive(Default)]
pub(crate) struct Pulse {
    pub(crate) start: TimeValueOptional<Real>,
    pub(crate) end: TimeValueOptional<Real>,
    pub(crate) peak: TimeValueOptional<Real>,
    pub(crate) steepest_rise: TimeValueOptional<RealArray<2>>,
    pub(crate) sharpest_fall: TimeValueOptional<RealArray<2>>,
}

impl Display for Pulse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "{0},{1},{2},{3},{4}",
            self.start, self.end, self.peak, self.steepest_rise, self.sharpest_fall
        ))
    }
}

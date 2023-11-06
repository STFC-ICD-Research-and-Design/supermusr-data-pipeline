use std::fmt::Display;

use crate::Real;
use crate::RealArray;

#[derive(Default, Clone, Debug)]
pub struct TimeValue<T>
where
    T: Default + Clone,
{
    pub time: Real,
    pub value: T,
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
pub struct TimeValueOptional<T>
where
    T: Default + Clone,
{
    pub time: Option<Real>,
    pub value: Option<T>,
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
pub struct Pulse {
    pub start: TimeValueOptional<Real>,
    pub end: TimeValueOptional<Real>,
    pub peak: TimeValueOptional<Real>,
    pub steepest_rise: TimeValueOptional<RealArray<2>>,
    pub sharpest_fall: TimeValueOptional<RealArray<2>>,
}

impl Display for Pulse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "{0},{1},{2},{3},{4}",
            self.start, self.end, self.peak, self.steepest_rise, self.sharpest_fall
        ))
    }
}

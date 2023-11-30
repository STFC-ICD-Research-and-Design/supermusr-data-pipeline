pub(crate) mod baseline;
pub(crate) mod finite_differences;
pub(crate) mod smoothing_window;

use super::{Real, RealArray, Stats, Temporal, TracePoint};
pub(crate) use baseline::Baseline;
pub(crate) use finite_differences::FiniteDifferences;
pub(crate) use smoothing_window::SmoothingWindow;

pub(crate) trait Window: Clone {
    type TimeType: Temporal;
    type InputType: Copy;
    type OutputType;

    fn push(&mut self, value: Self::InputType) -> bool;
    fn output(&self) -> Option<Self::OutputType>;
    fn apply_time_shift(&self, time: Self::TimeType) -> Self::TimeType;
}

#[derive(Clone)]
pub(crate) struct WindowIter<I, W>
where
    I: Iterator,
    I::Item: TracePoint,
    W: Window,
{
    window_function: W,
    source: I,
}

impl<I, W> WindowIter<I, W>
where
    I: Iterator,
    I::Item: TracePoint,
    W: Window,
{
    pub fn new(source: I, window_function: W) -> Self {
        WindowIter {
            source,
            window_function,
        }
    }

    #[cfg(test)]
    pub fn get_window(&self) -> &W {
        &self.window_function
    }
}

impl<I, W> Iterator for WindowIter<I, W>
where
    I: Iterator,
    I::Item: TracePoint,
    W: Window<
        TimeType = <I::Item as TracePoint>::TimeType,
        InputType = <I::Item as TracePoint>::ValueType,
    >,
{
    type Item = (W::TimeType, W::OutputType);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let val = self.source.next()?;
            if self.window_function.push(val.get_value().clone()) {
                return Some((
                    self.window_function.apply_time_shift(val.get_time()),
                    self.window_function.output()?,
                ));
            }
        }
    }
}
pub(crate) trait WindowFilter<I, W>
where
    I: Iterator,
    I::Item: TracePoint,
    W: Window,
{
    fn window(self, window: W) -> WindowIter<I, W>;
}

impl<I, W> WindowFilter<I, W> for I
where
    I: Iterator,
    I::Item: TracePoint,
    W: Window,
{
    fn window(self, window: W) -> WindowIter<I, W> {
        WindowIter::<I, W>::new(self, window)
    }
}

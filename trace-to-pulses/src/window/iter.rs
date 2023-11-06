use crate::tracedata::TraceData;

use super::Window;

#[derive(Clone)]
pub struct WindowIter<I, W>
where
    I: Iterator,
    I::Item: TraceData,
    W: Window,
{
    window_function: W,
    source: I,
}

impl<I, W> WindowIter<I, W>
where
    I: Iterator,
    I::Item: TraceData,
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
    I::Item: TraceData,
    W: Window<
        TimeType = <I::Item as TraceData>::TimeType,
        InputType = <I::Item as TraceData>::ValueType,
    >,
{
    type Item = (W::TimeType, W::OutputType);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let val = self.source.next()?;
            if self.window_function.push(val.get_value().clone()) {
                return Some((
                    self.window_function.apply_time_shift(val.get_time()),
                    self.window_function.stats()?,
                ));
            }
        }
    }
}
pub trait WindowFilter<I, W>
where
    I: Iterator,
    I::Item: TraceData,
    W: Window,
{
    fn window(self, window: W) -> WindowIter<I, W>;
}

impl<I, W> WindowFilter<I, W> for I
where
    I: Iterator,
    I::Item: TraceData,
    W: Window,
{
    fn window(self, window: W) -> WindowIter<I, W> {
        WindowIter::<I, W>::new(self, window)
    }
}

/*

#[derive(Clone)]
pub struct WindowEachIter<I, W, const N : usize, T> where
    I: Iterator,
    T: TraceValue,
    I::Item : TraceData<ValueType = TraceArray<N,T>>,
    W: Window,
{
    window_function: [W; N],
    source: I,
}

impl<I, W, const N : usize, T> WindowEachIter<I, W, N, T> where
    I: Iterator,
    T : TraceValue,
    I::Item : TraceData<ValueType = TraceArray<N,T>>,
    W: Window,
{
    pub fn new(source: I, window_function: [W; N]) -> Self {
        WindowEachIter { source, window_function }
    }
    #[cfg(test)]
    pub fn get_window(&self) -> &[W;N] {
        &self.window_function
    }
}

impl<I, W, const N : usize, T> Iterator for WindowEachIter<I,W,N,T> where
    I: Iterator,
    T : TraceValue,
    I::Item : TraceData<ValueType = TraceArray<N,T>>,
    W: Window<
        TimeType = <I::Item as TraceData>::TimeType,
        InputType = <I::Item as TraceData>::ValueType
    >,
{
    type Item = (W::TimeType, W::OutputType);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let val = self.source.next()?;
            if self.window_function.iter_mut().map(|window_function| window_function.push(val.get_value().clone())).any(|b|b) {
                return Some(self.window_function.iter().map(|window_function|Some((window_function.apply_time_shift(val.get_time()), window_function.stats()?))).try_into()?);
            }
        }
    }
}





pub trait WindowEachFilter<I, W, const N : usize, T> where
    I: Iterator,
    T : TraceValue,
    I::Item : TraceData<ValueType = TraceArray<N,T>>,
    W: Window,
{
    fn window_each(self, window: W) -> WindowEachIter<I, W, N, T>;
}

impl<I, W, const N : usize, T> WindowEachFilter<I,W,N,T> for I where
    I: Iterator,
    T : TraceValue,
    I::Item : TraceData<ValueType = TraceArray<N,T>>,
    W: Window,
{
    fn window_each(self, window: W) -> WindowEachIter<I,W,N,T> {
        WindowEachIter::<I,W,N,T>::new(self, window)
    }
} */

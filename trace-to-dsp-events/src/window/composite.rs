use crate::{Real, trace_iterators::RealArray};
use core::array::from_fn;

use crate::window::Window;

use super::TrivialWindow;
use super::smoothing_window::Stats;

type BoxedDynWindow = Box<dyn Window<InputType = Real, OutputType = Stats>>;

pub struct CompositeWindow<const N : usize> {
    windows : [BoxedDynWindow; N],
}
impl<const N : usize> CompositeWindow<N> {
    pub fn new(windows :[BoxedDynWindow;N]) -> Self {
        CompositeWindow { windows }
    }
    
    pub fn trivial() -> Self {
        CompositeWindow { windows: from_fn(|_|Box::new(TrivialWindow::default()) as BoxedDynWindow) }
    }
}
impl<const N : usize> Window for CompositeWindow<N> {
    type InputType = RealArray<N>;
    type OutputType = [Stats;N];

    fn push(&mut self, value : RealArray<N>) -> bool {
        let mut full = true;
        for i in 0..N {
            full = full && self.windows[i].push(value[i])
        }
        full
        
    }
    fn stats(&self) -> Option<Self::OutputType> {
        // This will do for now, but calling stats twice is inefficient (maybe)
        if self.windows.iter().any(|window|window.stats().is_none()) {
            None
        } else {
            Some(from_fn(|i|self.windows[i].stats().unwrap()))
        }
    }
}
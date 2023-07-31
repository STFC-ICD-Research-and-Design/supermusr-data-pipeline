use crate::Real;

pub mod smoothing_window;
pub mod noise_smoothing_window;
pub mod gate;
pub mod composite;


use smoothing_window::Stats;

pub trait Window {
    type InputType : Copy;
    type OutputType;
    
    fn push(&mut self, value : Self::InputType) -> bool;
    fn stats(&self) -> Option<Self::OutputType>;
}



#[derive(Clone)]
pub struct WindowIter<I,W> where I : Iterator<Item = (Real,W::InputType)>, W : Window {
    window : W,
    source : I,
}

impl<I,W> WindowIter<I,W> where I : Iterator<Item = (Real,W::InputType)>, W : Window {
    pub fn new(source : I, window : W) -> Self {
        WindowIter { source, window }
    }
    #[cfg(test)]
    pub fn get_window(&self) -> &W { &self.window }
}

impl<I,W> Iterator for WindowIter<I,W> where I : Iterator<Item = (Real,W::InputType)>, W : Window {
    type Item = (Real,W::OutputType);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let val = self.source.next()?;
            if self.window.push(val.1) {
                return Some((val.0,self.window.stats()?));
            }
        }
    }
}
pub trait WindowFilter<I,W> where W : Window, I : Iterator<Item = (Real,W::InputType)> {
    fn window(self, window : W) -> WindowIter<I,W>;
}

impl<I,W> WindowFilter<I,W> for I where W : Window, I : Iterator<Item = (Real,W::InputType)> {
    fn window(self, window : W) -> WindowIter<I,W> {
        WindowIter::<I,W>::new(self,window)
    }
}





#[derive(Default,Clone,Copy)]
pub struct TrivialWindow {
    value : Real,
}
impl Window for TrivialWindow {
    type InputType = Real;
    type OutputType = Stats;

    fn push(&mut self, value : Real) -> bool {
        self.value = value;
        true
    }
    fn stats(&self) -> Option<Self::OutputType> {
        Some(Stats{value: self.value, mean: self.value, variance: 0.})
    }
}
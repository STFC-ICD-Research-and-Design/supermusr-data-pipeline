use crate::{EventIter, Detector, events::{SimpleEvent, EventData}, peak_detector, Real};


pub trait ToTrace<I,T,V> where I : Iterator {
    fn to_trace(self, length : T) -> Vec<(T,V)>;
}

impl<I> ToTrace<I,Real,Real> for I where I: Iterator<Item = SimpleEvent<peak_detector::Data>> {
    fn to_trace(self, length : Real) -> Vec<(Real,Real)> {
        let events : Vec<SimpleEvent<peak_detector::Data>> = self.collect();
        let mut it = events.iter();
        (0..length as usize).map(|i| (i as Real, {
            it.next();
            0.
        }
        )).collect()
    }
}

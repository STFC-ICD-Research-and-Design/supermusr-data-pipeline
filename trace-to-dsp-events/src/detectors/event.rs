use std::fmt::Display;
use common::Intensity;
use common::Time;
use std::fmt::Debug;
use crate::{Integer,Real};

#[derive(Default,Debug,Clone,Copy)]
pub struct FuzzyReal {
    value: Real,
    uncertainty: Real,
}

impl FuzzyReal {
    pub fn from_real(value: Real) -> Self { Self {value, uncertainty: 0.} }
    pub fn new(value: Real, uncertainty : Real) -> Self { Self {value, uncertainty} }
    pub fn get_central(&self) -> Real { self.value }
    pub fn get_uncertainty(&self) -> Real { self.uncertainty }
}
impl Display for FuzzyReal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{0}~{1}",self.value,self.uncertainty,))
    }
}



#[derive(Default,Debug,Clone,Copy)]
pub struct TimeValue {
    pub time : FuzzyReal,
    pub value : FuzzyReal,
}
impl TimeValue {
    pub fn new(time: FuzzyReal, value : FuzzyReal) -> Self { Self {time, value} }
    pub fn from_exact( time : Real, value : Real ) -> Self { Self {
        time: FuzzyReal::from_real(time),
        value: FuzzyReal::from_real(value),
    }}
}
impl Display for TimeValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{0}:{1}",self.time,self.value,))
    }
}




pub trait EventClass : Default + Debug + Clone + Display {}
pub trait Event : Default + Debug + Clone + Display {
    fn has_influence_at(&self, index: Real) -> bool;
    fn get_intensity(&self, index: Real) -> Real;
}

#[derive(Default,Debug,Clone)]
pub struct SingleEvent<C> where C : EventClass {
    pub class : C,
    pub peak: TimeValue,
    pub bounds: Option<(TimeValue,TimeValue)>,
}
impl<C> Event for SingleEvent<C> where C : EventClass {
    fn has_influence_at(&self, index : Real) -> bool {
        if let Some((start,end)) = self.bounds {
            start.time.get_central() <= index && index <= start.time.get_central()
        } else {
            true
        }
    }
    fn get_intensity(&self, index: Real) -> Real {
        match self.bounds {
            Some((start,end)) => {
                self.peak.value.get_central()*Real::exp(-0.5*(self.peak.time.get_central() - index).powi(2)/(end.time.get_central() - start.time.get_central()).powi(2))
            },
            None => 0.,
        }
    }
}

impl<C> SingleEvent<C> where C : EventClass {
    pub fn new(class : C, peak : TimeValue, bounds : Option<(TimeValue,TimeValue)>) -> Self {
        SingleEvent {class, peak, bounds,}
    }
}

impl<C> Display for SingleEvent<C> where C : EventClass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.bounds {
            Some((start,end))   => f.write_fmt(format_args!("{0},{1},{2},{3};", self.class, self.peak, start, end)),
            None                => f.write_fmt(format_args!("{0},{1};",         self.class, self.peak)),
        }
    }
}

#[derive(Default,Debug,Clone)]
pub struct MultipleEvents<C> where C : EventClass {
    events : Vec<(usize,SingleEvent<C>)>,
}
impl<C> MultipleEvents<C> where C : EventClass {
    pub fn new(events: Vec<(usize,SingleEvent<C>)>) -> Self {
        Self { events }
    }
}

impl<C> Event for MultipleEvents<C> where C : EventClass {
    fn has_influence_at(&self, index : Real) -> bool {
        true
    }
    fn get_intensity(&self, index: Real) -> Real {
        0.
    }
}
impl<C> Display for MultipleEvents<C> where C : EventClass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (i,event) in &self.events {
            f.write_fmt(format_args!("{i},{event}"))?;
        }
        Ok(())
    }
}
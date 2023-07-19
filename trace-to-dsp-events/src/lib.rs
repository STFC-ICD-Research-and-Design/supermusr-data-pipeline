use common::Intensity;
use common::Time;

// Code from https://github.com/swizard0/smoothed_z_score/blob/master/README.md

pub enum Peak {Low, High}

type Real = f64;

pub struct EventsDetector {
    threshold: Real,
    influence: Real,
    window: Vec<Intensity>,
}

impl EventsDetector {
    pub fn new(lag: usize, threshold: Real, influence: Real) -> EventsDetector {
        EventsDetector {
            threshold,
            influence,
            window: Vec::with_capacity(lag),
        }
    }

    pub fn signal(&mut self, value: Intensity) -> Option<Peak> {
        if self.window.len() < self.window.capacity() {
            self.window.push(value);
            None
        } else if let (Some((mean, stddev)), Some(&window_last)) = (self.stats(), self.window.last()) {
            self.window.remove(0);
            let difference = if value < mean { mean - value } else { value - mean };
            if difference as Real > (self.threshold * stddev as Real) {
                let next_value =
                    ((value as Real * self.influence) + ((1. - self.influence) * window_last as Real)) as Intensity;
                self.window.push(next_value);
                Some(if value > mean { Peak::High } else { Peak::Low })
            } else {
                self.window.push(value);
                None
            }
        } else {
            None
        }
    }

    pub fn stats(&self) -> Option<(Intensity, Intensity)> {
        if self.window.is_empty() {
            None
        } else {
            let window_len = self.window.len() as f64;
            let mean = self.window.iter().fold(0, |a, v| a + v) as f64 / window_len;
            let sq_sum = self.window.iter().fold(0., |a, v| a + f64::powi(*v as f64 - mean,2));
            let stddev = (sq_sum / (window_len - 1.)).sqrt();
            Some((mean as Intensity, stddev as Intensity))
        }
    }
}

#[derive(Default,Debug)]
pub struct Event {
    pub time: Time,
    pub intensity: Intensity,
    pub width: Time,
}

pub struct EventIter<I> where I: Iterator<Item = (usize,Intensity)> {
    source : I,
    detector : EventsDetector,
}

impl<I> Iterator for EventIter<I> where I: Iterator<Item = (usize,Intensity)> {
    type Item = Event;
    fn next(&mut self) -> Option<Self::Item> {

        let mut base = Intensity::default();
        let mut base_at = Time::default();

        let mut peak = Intensity::default();

        while let Some(item) = self.source.next() {
            if self.detector.signal(item.1).is_some() {
                base_at = item.0 as Time;
                base = item.1;
                peak = item.1;
                break;
            }
        }

        while let Some(item) = self.source.next() {
            if let Some(signal) = self.detector.signal(item.1) {
                match signal {
                    Peak::High =>
                        if item.1 > peak {
                            peak = item.1;
                        },
                    Peak::Low =>
                        return Some(Event {
                            time: base_at,
                            intensity: peak - base,
                            width: item.0 as Time - base_at,
                        })
                }
            }
        }
        None
    }
}

pub trait EventFilter<I> where I: Iterator<Item = (usize,Intensity)>  {
    fn events(self, detector : EventsDetector) -> EventIter<I>;
}

impl<I> EventFilter<I> for I where I: Iterator<Item = (usize,Intensity)>  {
    fn events(self, detector: EventsDetector) -> EventIter<I>
    {
        EventIter { source: self, detector, }
    }
}

#[cfg(test)]
mod tests {
    use super::{EventsDetector, EventFilter, Intensity};

    #[test]
    fn sample_data() {
        let input = vec![
            1.0, 1.0, 1.1, 1.0, 0.9, 1.0, 1.0, 1.1, 1.0, 0.9, 1.0, 1.1, 1.0, 1.0, 0.9, 1.0, 1.0, 1.1, 1.0,
            1.0, 1.0, 1.0, 1.1, 0.9, 1.0, 1.1, 1.0, 1.0, 0.9, 1.0, 1.1, 1.0, 1.0, 1.1, 1.0, 0.8, 0.9, 1.0,
            1.2, 0.9, 1.0, 1.0, 1.1, 1.2, 1.0, 1.5, 1.0, 3.0, 2.0, 5.0, 3.0, 2.0, 1.0, 1.0, 1.0, 0.9, 1.0,
            1.0, 3.0, 2.6, 4.0, 3.0, 3.2, 2.0, 1.0, 1.0, 0.8, 4.0, 4.0, 2.0, 2.5, 1.0, 1.0, 1.0
        ];
        let output: Vec<_> = input.iter().map(|x|(x*1000.) as Intensity)
            .into_iter()
            .enumerate()
            .events(EventsDetector::new(10, 2.0, 0.6))
            .collect();
        for line in output {
            println!("{line:?}")
        }
    }
}
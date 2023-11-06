use std::fmt::Display;

use crate::events::Event;
use crate::tracedata::{EventData, Stats};
use crate::{Detector, Real};

#[derive(Default, Debug, Clone, PartialEq)]
pub enum ChangeClass {
    #[default]
    Flat,
    Rising,
    Falling,
}
impl Display for ChangeClass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "{0}",
            match self {
                Self::Rising => 1i32,
                Self::Flat => 0i32,
                Self::Falling => -1i32,
            }
        ))
    }
}

#[derive(Default, Debug, Clone)]
pub struct ChangeData {
    class: ChangeClass,
    from: Option<Real>,
    to: Option<Real>,
}
impl ChangeData {
    pub fn new(class: ChangeClass) -> Self {
        Self {
            class,
            ..Default::default()
        }
    }
    pub fn with_values(class: ChangeClass, from: Real, to: Real) -> Self {
        Self {
            class,
            from: Some(from),
            to: Some(to),
        }
    }
    pub fn get_class(&self) -> ChangeClass {
        self.class.clone()
    }
    pub fn get_value_from(&self) -> Option<Real> {
        self.from
    }
    pub fn get_value_to(&self) -> Option<Real> {
        self.to
    }
}
impl EventData for ChangeData {}
pub type ChangeEvent = Event<Real, ChangeData>;

impl Display for ChangeData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "{0},{1},{2}",
            self.from.unwrap_or_default(),
            self.class,
            self.to.unwrap_or_default()
        ))
    }
}

#[derive(Default, Clone)]
pub struct ChangeDetector {
    mode: ChangeClass,
    prev: Option<Real>,
    threshold: Real,
}
impl ChangeDetector {
    pub fn new(threshold: Real) -> Self {
        Self {
            threshold,
            ..Default::default()
        }
    }
}
impl Detector for ChangeDetector {
    type TimeType = Real;
    type ValueType = Stats;
    type DataType = ChangeData;

    fn signal(&mut self, time: Real, value: Stats) -> Option<ChangeEvent> {
        if let Some(prev_value) = self.prev {
            let new_mode = {
                if (value.mean - prev_value).abs() <= self.threshold {
                    ChangeClass::Flat
                } else if value.mean > prev_value {
                    ChangeClass::Rising
                } else {
                    ChangeClass::Falling
                }
            };

            let event_class = if new_mode == self.mode {
                None
            } else {
                Some(new_mode.clone())
            };
            self.mode = new_mode;
            self.prev = Some(value.mean);
            event_class.map(|e| {
                ChangeData::with_values(e.clone(), prev_value, value.mean).make_event(time)
            })
        } else {
            self.prev = Some(value.mean);
            None
        }
    }
}

/*
Modelling using skew gaussian
Histogramming
Adaptive binning
*/

#[cfg(test)]
mod tests {
    use itertools::Itertools;

    use super::*;
    use crate::processing;

    #[test]
    fn zero_data() {
        let data = [];
        let mut detector = ChangeDetector::new(1.);
        let results = data
            .iter()
            .enumerate()
            .map(processing::make_enumerate_real)
            .map(|(i, v)| detector.signal(i, v.into()))
            .collect_vec();

        assert!(results.is_empty());
    }

    #[test]
    fn test_with_data() {
        use ChangeClass::*;

        let data = [4.0, 3.0, 2.0, 5.0, 6.0, 1.0, 5.0, 7.0, 2.0, 4.0];
        let mut detector = ChangeDetector::new(1.5);
        let results = data
            .iter()
            .enumerate()
            .map(processing::make_enumerate_real)
            .map(|(i, v)| detector.signal(i, v.into()))
            .collect_vec();

        assert_eq!(results.len(), data.len());
        assert_eq!(results[0], None);
        assert_eq!(results[1], None);
        assert_eq!(results[2], None);
        assert_eq!(
            results[3],
            Some(ChangeData::with_values(Rising, 5., 2.).make_event(3.))
        );
        assert_eq!(
            results[4],
            Some(ChangeData::with_values(Flat, 6., 5.).make_event(4.))
        );
        assert_eq!(
            results[5],
            Some(ChangeData::with_values(Falling, 1., 6.).make_event(5.))
        );
        assert_eq!(
            results[6],
            Some(ChangeData::with_values(Rising, 5., 1.).make_event(6.))
        );
        assert_eq!(results[7], None);
        assert_eq!(
            results[8],
            Some(ChangeData::with_values(Falling, 2., 7.).make_event(8.))
        );
        assert_eq!(
            results[9],
            Some(ChangeData::with_values(Rising, 4., 2.).make_event(9.))
        );
    }
}

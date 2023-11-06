use std::fmt::Display;

use crate::events::Event;
use crate::tracedata::{EventData, Stats, TraceValue};
use crate::{Detector, Real, RealArray};

#[derive(Default, Debug, Clone, PartialEq)]
pub enum LocalExtremumClass {
    #[default]
    LocalMax,
    LocalMin,
}
impl Display for LocalExtremumClass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::LocalMax => "1",
            Self::LocalMin => "-1",
        })
    }
}

#[derive(Default, Debug, Clone)]
pub struct LocalExtremumData {
    class: LocalExtremumClass,
    tag: Option<usize>,
    value: Option<Real>,
}
impl LocalExtremumData {
    pub fn new(class: LocalExtremumClass) -> Self {
        LocalExtremumData {
            class,
            ..Default::default()
        }
    }
    pub fn add_value(self, value: Real) -> Self {
        LocalExtremumData {
            class: self.class,
            tag: self.tag,
            value: Some(value),
        }
    }
    pub fn add_tag(self, tag: usize) -> Self {
        LocalExtremumData {
            class: self.class,
            tag: Some(tag),
            value: self.value,
        }
    }
    pub fn get_class(&self) -> LocalExtremumClass {
        self.class.clone()
    }
    pub fn get_tag(&self) -> Option<usize> {
        self.tag
    }
    pub fn get_value(&self) -> Option<Real> {
        self.value
    }
}
impl EventData for LocalExtremumData {}

impl Display for LocalExtremumData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "{0},{1},{2}",
            self.class,
            self.tag.unwrap_or(0),
            self.value.unwrap_or(0.)
        ))
    }
}

type LocalExtremumEvent = Event<Real, LocalExtremumData>;

#[derive(Default, Clone)]
pub struct LocalExtremumDetector<D>
where
    D: TraceValue,
{
    threshold: D,
    prev: Option<(D, Option<D>)>,
}

impl<D> LocalExtremumDetector<D>
where
    D: TraceValue,
{
    pub fn new(threshold: D) -> Self {
        Self {
            threshold,
            ..Default::default()
        }
    }

    fn detect_event(
        time: Real,
        tag: usize,
        prev_prev_value: Real,
        prev_value: Real,
        value: Real,
        threshold: Real,
    ) -> Option<LocalExtremumEvent> {
        if prev_prev_value != value {
            if prev_prev_value + threshold <= prev_value && prev_value >= value + threshold {
                Some(LocalExtremumData::new(LocalExtremumClass::LocalMax))
            } else if prev_prev_value >= prev_value + threshold && prev_value + threshold <= value {
                Some(LocalExtremumData::new(LocalExtremumClass::LocalMin))
            } else {
                None
            }
        } else {
            None
        }
        .map(|data| data.add_value(prev_value).add_tag(tag).make_event(time))
    }
}

impl Detector for LocalExtremumDetector<Real> {
    type TimeType = Real;
    type ValueType = Real;
    type DataType = LocalExtremumData;

    fn signal(&mut self, time: Real, value: Real) -> Option<LocalExtremumEvent> {
        if let Some((prev_value, Some(prev_prev_value))) = self.prev {
            let return_value = Self::detect_event(
                time - 1.,
                usize::default(),
                prev_prev_value,
                prev_value,
                value,
                self.threshold,
            );
            self.prev = Some((value, Some(prev_value)));
            return_value
        } else if let Some((prev_value, None)) = self.prev {
            self.prev = Some((value, Some(prev_value)));
            None
        } else {
            self.prev = Some((value, None));
            None
        }
    }
}

impl Detector for LocalExtremumDetector<Stats> {
    type TimeType = Real;
    type ValueType = Stats;
    type DataType = LocalExtremumData;

    fn signal(&mut self, time: Real, value: Stats) -> Option<LocalExtremumEvent> {
        if let Some((prev_value, Some(prev_prev_value))) = &self.prev {
            let return_value = Self::detect_event(
                time - 1.,
                usize::default(),
                prev_prev_value.mean,
                prev_value.mean,
                value.mean,
                self.threshold.mean,
            );
            self.prev = Some((value, Some(prev_value.clone())));
            return_value
        } else if let Some((prev_value, None)) = &self.prev {
            self.prev = Some((value, Some(prev_value.clone())));
            None
        } else {
            self.prev = Some((value, None));
            None
        }
    }
}

impl<const N: usize> Detector for LocalExtremumDetector<RealArray<N>> {
    type TimeType = Real;
    type ValueType = RealArray<N>;
    type DataType = LocalExtremumData;

    fn signal(&mut self, time: Real, value: RealArray<N>) -> Option<LocalExtremumEvent> {
        if let Some((prev_value, Some(prev_prev_value))) = self.prev {
            let return_value = (0..N).fold(None, |return_value, i| {
                return_value.or_else(|| {
                    Self::detect_event(
                        time - 1.,
                        i,
                        prev_prev_value[i],
                        prev_value[i],
                        value[i],
                        self.threshold[i],
                    )
                })
            });
            self.prev = Some((value, Some(prev_value)));
            return_value
        } else if let Some((prev_value, None)) = self.prev {
            self.prev = Some((value, Some(prev_value)));
            None
        } else {
            self.prev = Some((value, None));
            None
        }
    }
}

#[derive(Default, Debug, Clone)]
pub struct PeakData {
    value: Option<Real>,
    time_since_start: Option<Real>,
    time_till_end: Option<Real>,
}
impl PeakData {
    pub fn get_value(&self) -> Option<Real> {
        self.value
    }
    pub fn get_time_since_start(&self) -> Option<Real> {
        self.time_since_start
    }
    pub fn get_time_till_end(&self) -> Option<Real> {
        self.time_till_end
    }
}
impl EventData for PeakData {}

impl Display for PeakData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "{0},{1},{2}",
            self.value.unwrap_or(0.),
            self.time_since_start.unwrap_or(0.),
            self.time_till_end.unwrap_or(0.)
        ))
    }
}

type PeakEvent = Event<Real, PeakData>;

pub fn local_extrema_to_peaks(
    (left, mid, right): (LocalExtremumEvent, LocalExtremumEvent, LocalExtremumEvent),
) -> Option<PeakEvent> {
    if mid.get_data().get_class() == LocalExtremumClass::LocalMax {
        Some(
            PeakData {
                value: mid.get_data().get_value(),
                time_since_start: Some(mid.get_time() - left.get_time()),
                time_till_end: Some(right.get_time() - mid.get_time()),
            }
            .make_event(mid.get_time()),
        )
    } else {
        None
    }
}

/*

#[derive(Default,Clone)]
pub struct PeakDetector {
    detector : LocalExtremeDetector,
    prev: VecDeque<LocalExtremeEvent>,
}

impl PeakDetector {
    pub fn new() -> Self { PeakDetector { prev: VecDeque::<LocalExtremeEvent>::with_capacity(3), ..Default::default() } }
}

impl Detector for PeakDetector {
    type TimeType = Real;
    type ValueType = Real;
    type DataType = PeakData;

    fn signal(&mut self, time: Real, value: Real) -> Option<PeakEvent> {
        let event = self.detector.signal(time,value);
        if let Some(event) = event {
            self.prev.push_front(event);
        }
        let return_value = if self.prev.len() > 2 {
            two_sided(self.prev.get(0), self.prev.get(1), self.prev.get(2))
        } else {
            None
        };
        self.prev.truncate(2);
        return_value
    }
}*/

#[cfg(test)]
mod tests {
    use itertools::Itertools;

    use super::*;
    use crate::processing;

    #[test]
    fn zero_data() {
        let data = [];
        let mut detector = LocalExtremumDetector::<Real>::new(1.0);
        let results = data
            .iter()
            .enumerate()
            .map(processing::make_enumerate_real)
            .map(|(i, v)| detector.signal(i, v.into()))
            .collect_vec();

        assert!(results.is_empty());
    }

    #[test]
    fn test_gate_zero_threshold() {
        let data = [4.0, 3.0, 2.0, 5.0, 6.0, 1.0, 5.0, 7.0, 2.0, 4.0];
        let mut detector = LocalExtremumDetector::<Real>::new(1.0);
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
            Some(
                LocalExtremumData::new(LocalExtremumClass::LocalMin)
                    .add_value(2.)
                    .make_event(2.)
            )
        );
        assert_eq!(results[4], None);
        assert_eq!(
            results[5],
            Some(
                LocalExtremumData::new(LocalExtremumClass::LocalMax)
                    .add_value(6.)
                    .make_event(4.)
            )
        );
        assert_eq!(
            results[6],
            Some(
                LocalExtremumData::new(LocalExtremumClass::LocalMin)
                    .add_value(1.)
                    .make_event(5.)
            )
        );
        assert_eq!(results[7], None);
        assert_eq!(
            results[8],
            Some(
                LocalExtremumData::new(LocalExtremumClass::LocalMax)
                    .add_value(7.)
                    .make_event(7.)
            )
        );
        assert_eq!(
            results[9],
            Some(
                LocalExtremumData::new(LocalExtremumClass::LocalMin)
                    .add_value(2.)
                    .make_event(8.)
            )
        );
    }
}

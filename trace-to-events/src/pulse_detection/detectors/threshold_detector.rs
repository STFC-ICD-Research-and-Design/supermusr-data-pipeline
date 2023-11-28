use super::{Assembler, Detector, EventData, EventPoint, Pulse, Real, TimeValueOptional};
use std::fmt::Display;
use std::marker::PhantomData;

#[derive(Default, Debug, Clone, PartialEq)]
pub(crate) struct Data {}

impl Display for Data {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Ok(())
    }
}

impl EventData for Data {}

#[derive(Default, Debug, Clone)]
pub(crate) struct ThresholdDuration {
    pub(crate) threshold: Real,
    pub(crate) duration: i32,
    pub(crate) cool_off: i32,
}

pub(crate) trait ThresholdClass: Default + Clone {
    fn test(value: Real, threshold: Real) -> bool;
}

#[derive(Default, Clone)]
pub(crate) struct UpperThreshold {}
impl ThresholdClass for UpperThreshold {
    fn test(value: Real, threshold: Real) -> bool {
        value > threshold
    }
}

#[derive(Default, Clone)]
pub(crate) struct LowerThreshold {}
impl ThresholdClass for LowerThreshold {
    fn test(value: Real, threshold: Real) -> bool {
        value < threshold
    }
}

#[derive(Default, Clone)]
pub(crate) struct ThresholdDetector<Class: ThresholdClass> {
    time: i32, // If this is non-negative, then the detector is armed
    trigger: ThresholdDuration,
    phantom: PhantomData<Class>,
}

impl<Class: ThresholdClass> ThresholdDetector<Class> {
    pub(crate) fn new(trigger: &ThresholdDuration) -> Self {
        Self {
            trigger: trigger.clone(),
            time: 0,
            ..Default::default()
        }
    }
}

pub(crate) type ThresholdEvent = (Real, Data);

impl<Class: ThresholdClass> Detector for ThresholdDetector<Class> {
    type TracePointType = (Real, Real);
    type EventPointType = (Real, Data);

    fn signal(&mut self, time: Real, value: Real) -> Option<ThresholdEvent> {
        if self.time < 0 {
            self.time += 1;
            None
        } else if Class::test(value, self.trigger.threshold) {
            self.time += 1;
            if self.time == self.trigger.duration {
                self.time = -self.trigger.cool_off;
                Some((time - (self.trigger.duration - 1) as Real / 2.0, Data {}))
            } else {
                None
            }
        } else {
            self.time = 0;
            None
        }
    }
}

#[derive(Default, Clone)]
pub(crate) struct ThresholdAssembler<Class: ThresholdClass> {
    phantom: PhantomData<Class>,
}

impl<Class: ThresholdClass> Assembler for ThresholdAssembler<Class> {
    type DetectorType = ThresholdDetector<Class>;

    fn assemble_pulses(
        &mut self,
        source: <Self::DetectorType as Detector>::EventPointType,
    ) -> Option<Pulse> {
        Some(Pulse {
            start: TimeValueOptional {
                time: Some(source.get_time()),
                ..Default::default()
            },
            ..Default::default()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pulse_detection::EventFilter;

    #[test]
    fn zero_data() {
        let data: [Real; 0] = [];
        let detector = ThresholdDetector::<UpperThreshold>::new(&ThresholdDuration {
            threshold: 2.0,
            cool_off: 0,
            duration: 2,
        });
        let mut iter = data
            .into_iter()
            .enumerate()
            .map(|(i, v)| (i as Real, v as Real))
            .events(detector);
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_positive_threshold() {
        let data = [4, 3, 2, 5, 6, 1, 5, 7, 2, 4];
        let detector = ThresholdDetector::<UpperThreshold>::new(&ThresholdDuration {
            threshold: 2.0,
            cool_off: 0,
            duration: 2,
        });
        let mut iter = data
            .into_iter()
            .enumerate()
            .map(|(i, v)| (i as Real, v as Real))
            .events(detector);
        assert_eq!(iter.next(), Some((0.5, Data {})));
        assert_eq!(iter.next(), Some((3.5, Data {})));
        assert_eq!(iter.next(), Some((6.5, Data {})));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_negative_threshold() {
        let data = [4, 3, 2, 5, 2, 1, 5, 7, 2, 2];
        let detector = ThresholdDetector::<LowerThreshold>::new(&ThresholdDuration {
            threshold: 2.5,
            cool_off: 0,
            duration: 2,
        });
        let mut iter = data
            .into_iter()
            .enumerate()
            .map(|(i, v)| (i as Real, v as Real))
            .events(detector);
        assert_eq!(iter.next(), Some((4.5, Data {})));
        assert_eq!(iter.next(), Some((8.5, Data {})));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_zero_duration() {
        let data = [4, 3, 2, 5, 2, 1, 5, 7, 2, 2];
        let detector = ThresholdDetector::<LowerThreshold>::new(&ThresholdDuration {
            threshold: 2.5,
            cool_off: 0,
            duration: 0,
        });
        let mut iter = data
            .into_iter()
            .enumerate()
            .map(|(i, v)| (i as Real, v as Real))
            .events(detector);
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_cool_off() {
        // Without cool-off the detector triggers at the following points:
        //          .  .  x  .  x  x  .  .  x  x
        // With a 1 sample cool-off the detector triggers at the following points
        //          .  .  x  .  x  .  .  .  x  .
        // With a 2 sample cool-off the detector triggers at the following points
        //          .  .  x  .  .  x  .  .  x  .
        let data = [4, 3, 2, 5, 2, 1, 5, 7, 2, 2];
        let detector2 = ThresholdDetector::<LowerThreshold>::new(&ThresholdDuration {
            threshold: 2.5,
            cool_off: 2,
            duration: 1,
        });
        let mut iter = data
            .iter()
            .copied()
            .enumerate()
            .map(|(i, v)| (i as Real, v as Real))
            .events(detector2);
        assert_eq!(iter.next(), Some((2.0, Data {})));
        assert_eq!(iter.next(), Some((5.0, Data {})));
        assert_eq!(iter.next(), Some((8.0, Data {})));
        assert_eq!(iter.next(), None);

        let detector1 = ThresholdDetector::<LowerThreshold>::new(&ThresholdDuration {
            threshold: 2.5,
            cool_off: 1,
            duration: 1,
        });

        let mut iter = data
            .into_iter()
            .enumerate()
            .map(|(i, v)| (i as Real, v as Real))
            .events(detector1);
        assert_eq!(iter.next(), Some((2.0, Data {})));
        assert_eq!(iter.next(), Some((4.0, Data {})));
        assert_eq!(iter.next(), Some((8.0, Data {})));
        assert_eq!(iter.next(), None);

        let detector0 = ThresholdDetector::<LowerThreshold>::new(&ThresholdDuration {
            threshold: 2.5,
            cool_off: 0,
            duration: 1,
        });

        let mut iter = data
            .into_iter()
            .enumerate()
            .map(|(i, v)| (i as Real, v as Real))
            .events(detector0);
        assert_eq!(iter.next(), Some((2.0, Data {})));
        assert_eq!(iter.next(), Some((4.0, Data {})));
        assert_eq!(iter.next(), Some((5.0, Data {})));
        assert_eq!(iter.next(), Some((8.0, Data {})));
        assert_eq!(iter.next(), Some((9.0, Data {})));
        assert_eq!(iter.next(), None);
    }
}

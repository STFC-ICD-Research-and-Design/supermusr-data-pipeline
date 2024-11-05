use super::{Assembler, Detector, EventData, Pulse, Real, TimeValueOptional};
use std::fmt::Display;

#[derive(Default, Debug, Clone, PartialEq)]
pub(crate) struct Data {
    pub(crate) pulse_height: Real,
}

impl Display for Data {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.pulse_height)
    }
}

impl EventData for Data {}

#[derive(Default, Debug, Clone)]
pub(crate) struct ThresholdDuration {
    pub(crate) threshold: Real,
    pub(crate) duration: i32,
    pub(crate) cool_off: i32,
}

#[derive(Default, Clone)]
pub(crate) struct ThresholdDetector {
    trigger: ThresholdDuration,
    time_of_last_return: Option<Real>,
    time_crossed: Option<Real>,
    temp_time: Option<Real>,
    max_pulse_height: Real,
}

impl ThresholdDetector {
    pub(crate) fn new(trigger: &ThresholdDuration) -> Self {
        Self {
            trigger: trigger.clone(),
            ..Default::default()
        }
    }
}

pub(crate) type ThresholdEvent = (Real, Data);

impl Detector for ThresholdDetector {
    type TracePointType = (Real, Real);
    type EventPointType = (Real, Data);

    fn signal(&mut self, time: Real, value: Real) -> Option<ThresholdEvent> {
        match self.time_crossed {
            Some(time_crossed) => {
                // If we are already over the threshold
                self.max_pulse_height = self.max_pulse_height.max(value);

                if time - time_crossed == self.trigger.duration as Real {
                    // If the current value is below the threshold
                    self.temp_time = Some(time_crossed);
                }

                if value <= self.trigger.threshold {
                    // If the current value is below the threshold
                    self.time_crossed = None;
                    if time - time_crossed >= self.trigger.duration as Real {
                        self.time_of_last_return = Some(time);

                        if let Some(time) = &self.temp_time {
                            let result = (
                                *time,
                                Data {
                                    pulse_height: self.max_pulse_height,
                                },
                            );
                            self.temp_time = None;
                            Some(result)
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            None => {
                //  If we are under the threshold
                if value > self.trigger.threshold {
                    // If the current value as over the threshold
                    // If we have a "time_of_last_return", then test if we have passed the cool-down time
                    match self.time_of_last_return {
                        Some(time_of_last_return) => {
                            if time - time_of_last_return >= self.trigger.cool_off as Real {
                                self.max_pulse_height = value;
                                self.time_crossed = Some(time);
                            }
                        }
                        None => {
                            self.max_pulse_height = value;
                            self.time_crossed = Some(time);
                        }
                    }
                }
                None
            }
        }
    }

    fn finish(&mut self) -> Option<Self::EventPointType> {
        let result = self.temp_time;
        self.temp_time = None;
        result.map(|time| {
            (
                time,
                Data {
                    pulse_height: self.max_pulse_height,
                },
            )
        })
    }
}

#[derive(Default, Clone)]
pub(crate) struct ThresholdAssembler {}

impl Assembler for ThresholdAssembler {
    type DetectorType = ThresholdDetector;

    fn assemble_pulses(
        &mut self,
        source: <Self::DetectorType as Detector>::EventPointType,
    ) -> Option<Pulse> {
        let (time, Data { pulse_height }) = source;
        Some(Pulse {
            start: TimeValueOptional {
                time: Some(time),
                value: Some(pulse_height),
            },
            ..Default::default()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pulse_detection::{EventFilter, Real};

    #[test]
    fn zero_data() {
        let data: [Real; 0] = [];
        let detector = ThresholdDetector::new(&ThresholdDuration {
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
        let detector = ThresholdDetector::new(&ThresholdDuration {
            threshold: 2.0,
            cool_off: 0,
            duration: 2,
        });
        let mut iter = data
            .into_iter()
            .enumerate()
            .map(|(i, v)| (i as Real, v as Real))
            .events(detector);
        assert_eq!(iter.next(), Some((0.0, Data { pulse_height: 4.0 })));
        assert_eq!(iter.next(), Some((3.0, Data { pulse_height: 6.0 })));
        assert_eq!(iter.next(), Some((6.0, Data { pulse_height: 7.0 })));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_negative_threshold() {
        let data = [4, 3, 2, 5, 2, 1, 5, 7, 2, 2, 2, 4];
        let detector = ThresholdDetector::new(&ThresholdDuration {
            threshold: -2.5,
            cool_off: 0,
            duration: 2,
        });
        let mut iter = data
            .into_iter()
            .enumerate()
            .map(|(i, v)| (i as Real, -v as Real))
            .events(detector);
        assert_eq!(iter.next(), Some((4.0, Data { pulse_height: -1.0 })));
        assert_eq!(iter.next(), Some((8.0, Data { pulse_height: -2.0 })));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_zero_duration() {
        let data = [4, 3, 2, 5, 2, 1, 5, 7, 2, 2];
        let detector = ThresholdDetector::new(&ThresholdDuration {
            threshold: -2.5,
            cool_off: 0,
            duration: 0,
        });
        let mut iter = data
            .into_iter()
            .enumerate()
            .map(|(i, v)| (i as Real, -v as Real))
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
        let detector2 = ThresholdDetector::new(&ThresholdDuration {
            threshold: -2.5,
            cool_off: 2,
            duration: 1,
        });
        let mut iter = data
            .iter()
            .copied()
            .enumerate()
            .map(|(i, v)| (i as Real, -v as Real))
            .events(detector2);
        assert_eq!(iter.next(), Some((2.0, Data { pulse_height: -2.0 })));
        assert_eq!(iter.next(), Some((5.0, Data { pulse_height: -1.0 })));
        assert_eq!(iter.next(), Some((8.0, Data { pulse_height: -2.0 })));
        assert_eq!(iter.next(), None);

        let detector1 = ThresholdDetector::new(&ThresholdDuration {
            threshold: -2.5,
            cool_off: 1,
            duration: 1,
        });

        let mut iter = data
            .into_iter()
            .enumerate()
            .map(|(i, v)| (i as Real, -v as Real))
            .events(detector1);
        assert_eq!(iter.next(), Some((2.0, Data { pulse_height: -2.0 })));
        assert_eq!(iter.next(), Some((4.0, Data { pulse_height: -1.0 })));
        assert_eq!(iter.next(), Some((8.0, Data { pulse_height: -2.0 })));
        assert_eq!(iter.next(), None);

        let detector0 = ThresholdDetector::new(&ThresholdDuration {
            threshold: -2.5,
            cool_off: 0,
            duration: 1,
        });

        let mut iter = data
            .into_iter()
            .enumerate()
            .map(|(i, v)| (i as Real, -v as Real))
            .events(detector0);
        assert_eq!(iter.next(), Some((2.0, Data { pulse_height: -2.0 })));
        assert_eq!(iter.next(), Some((4.0, Data { pulse_height: -1.0 })));
        assert_eq!(iter.next(), Some((8.0, Data { pulse_height: -2.0 })));
        assert_eq!(iter.next(), None);
    }
}

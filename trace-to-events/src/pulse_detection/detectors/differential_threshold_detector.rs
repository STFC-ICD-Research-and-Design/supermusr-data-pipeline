use crate::pulse_detection::{
    datatype::tracevalue::TraceArray, threshold_detector::ThresholdDuration,
};

use super::{Detector, EventData, Real};
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

#[derive(Default, Clone)]
pub(crate) struct DifferentialThresholdDetector {
    trigger: ThresholdDuration,

    time_of_last_return: Option<Real>,
    constant_multiple: Real,
    time_crossed: Option<Real>,
    temp_time: Option<Real>,
    max_derivative: TraceArray<2, Real>,
}

impl DifferentialThresholdDetector {
    pub(crate) fn new(trigger: &ThresholdDuration, constant_multiple: Real) -> Self {
        Self {
            trigger: trigger.clone(),
            constant_multiple,
            ..Default::default()
        }
    }
}

pub(crate) type ThresholdEvent = (Real, Data);

impl Detector for DifferentialThresholdDetector {
    type TracePointType = (Real, TraceArray<2, Real>);
    type EventPointType = (Real, Data);

    fn signal(&mut self, time: Real, value: TraceArray<2, Real>) -> Option<ThresholdEvent> {
        match self.time_crossed {
            Some(time_crossed) => {
                // If we are already over the threshold
                if self.max_derivative[1] < value[1] {
                    // Set update the max derivative if the current derivative is higher.
                    self.max_derivative = value;
                }

                if time - time_crossed == self.trigger.duration as Real {
                    // If the current value is below the threshold
                    self.temp_time = Some(time_crossed);
                }

                if value[1] <= 0.0 {
                    // If the current differential is non-positive
                    self.time_crossed = None;
                    if time - time_crossed >= self.trigger.duration as Real {
                        self.time_of_last_return = Some(time);

                        if let Some(time) = &self.temp_time {
                            let result = (
                                *time,
                                Data {
                                    pulse_height: self.max_derivative[0]*self.constant_multiple,
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
                if value[1] > self.trigger.threshold {
                    // If the current value as over the threshold
                    // If we have a "time_of_last_return", then test if we have passed the cool-down time
                    match self.time_of_last_return {
                        Some(time_of_last_return) => {
                            if time - time_of_last_return >= self.trigger.cool_off as Real {
                                self.max_derivative = value;
                                self.time_crossed = Some(time);
                            }
                        }
                        None => {
                            self.max_derivative = value;
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
                    pulse_height: self.max_derivative[0]*self.constant_multiple,
                },
            )
        })
    }
}

#[cfg(test)]
mod tests {
    use supermusr_common::Intensity;

    use super::*;
    use crate::pulse_detection::{window::{FiniteDifferences, SmoothingWindow}, EventFilter, Real, WindowFilter};

    #[test]
    fn zero_data() {
        let data: [Real; 0] = [];
        let detector = DifferentialThresholdDetector::new(&ThresholdDuration {
            threshold: 2.0,
            cool_off: 0,
            duration: 2,
        }, 2.0);
        let mut iter = data
            .into_iter()
            .enumerate()
            .map(|(i, v)| (i as Real, v as Real))
            .window(FiniteDifferences::<2>::new())
            .events(detector);
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_positive_threshold() {
        let data = [4, 3, 2, 5, 6, 1, 5, 7, 2, 4];
        let detector = DifferentialThresholdDetector::new(&ThresholdDuration {
            threshold: 2.0,
            cool_off: 0,
            duration: 2,
        }, 2.0);
        let mut iter = data
            .into_iter()
            .enumerate()
            .map(|(i, v)| (i as Real, v as Real))
            .window(FiniteDifferences::<2>::new())
            .events(detector);
        assert_eq!(iter.next(), Some((3.0, Data { pulse_height: 10.0 })));
        assert_eq!(iter.next(), Some((6.0, Data { pulse_height: 10.0 })));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_zero_duration() {
        let data = [4, 3, 2, 5, 2, 1, 5, 7, 2, 2];
        let detector = DifferentialThresholdDetector::new(&ThresholdDuration {
            threshold: -2.5,
            cool_off: 0,
            duration: 0,
        }, 2.0);
        let mut iter = data
            .into_iter()
            .enumerate()
            .map(|(i, v)| (i as Real, -v as Real))
            .window(FiniteDifferences::<2>::new())
            .events(detector);
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_cool_off() {
        // With a 1 sample cool-off the detector triggers at the following points
        //          .  .  .  x  .  .  x  .  .  x  .  x  .  x
        // With a 2 sample cool-off the detector triggers at the following points
        //          .  .  .  x  .  .  x  .  .  .  .  x  .  .
        // With a 3 sample cool-off the detector triggers at the following points
        //          .  .  .  x  .  .  .  .  .  x  .  .  .  x
        let data = [4, 3, 2, 5, 2, 1, 5, 7, 2, 6, 5, 8, 8, 11, 0];
        let detector2 = DifferentialThresholdDetector::new(&ThresholdDuration {
            threshold: 2.5,
            cool_off: 3,
            duration: 1,
        }, 2.0);
        let mut iter = data
            .iter()
            .copied()
            .enumerate()
            .map(|(i, v)| (i as Real, v as Real))
            .window(FiniteDifferences::<2>::new())
            .events(detector2);
        assert_eq!(iter.next(), Some((3.0, Data { pulse_height: 10.0 })));
        assert_eq!(iter.next(), Some((9.0, Data { pulse_height: 12.0 })));
        assert_eq!(iter.next(), Some((13.0, Data { pulse_height: 22.0 })));
        assert_eq!(iter.next(), None);

        let detector1 = DifferentialThresholdDetector::new(&ThresholdDuration {
            threshold: 2.5,
            cool_off: 2,
            duration: 1,
        }, 2.0);

        let mut iter = data
            .into_iter()
            .enumerate()
            .map(|(i, v)| (i as Real, v as Real))
            .window(FiniteDifferences::<2>::new())
            .events(detector1);
        assert_eq!(iter.next(), Some((3.0, Data { pulse_height: 10.0 })));
        assert_eq!(iter.next(), Some((6.0, Data { pulse_height: 10.0 })));
        assert_eq!(iter.next(), Some((11.0, Data { pulse_height: 16.0 })));
        assert_eq!(iter.next(), None);

        let detector0 = DifferentialThresholdDetector::new(&ThresholdDuration {
            threshold: 2.5,
            cool_off: 1,
            duration: 1,
        }, 2.0);

        let mut iter = data
            .into_iter()
            .enumerate()
            .map(|(i, v)| (i as Real, v as Real))
            .window(FiniteDifferences::<2>::new())
            .events(detector0);
        assert_eq!(iter.next(), Some((3.0, Data { pulse_height: 10.0 })));
        assert_eq!(iter.next(), Some((6.0, Data { pulse_height: 10.0 })));
        assert_eq!(iter.next(), Some((9.0, Data { pulse_height: 12.0 })));
        assert_eq!(iter.next(), Some((11.0, Data { pulse_height: 16.0 })));
        assert_eq!(iter.next(), Some((13.0, Data { pulse_height: 22.0 })));
        assert_eq!(iter.next(), None);
    }
}

use super::{
    threshold_detector::{LowerThreshold, ThresholdDetector, ThresholdDuration, UpperThreshold},
    Assembler, Detector, EventData, EventPoint, Pulse, Real, RealArray, TimeValue,
};
use std::fmt::Display;

#[derive(Default, Debug, Clone, PartialEq)]
pub(crate) enum Class {
    #[default]
    Onset,
    Peak,
    End,
}

impl Display for Class {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Onset => "0",
            Self::Peak => "2",
            Self::End => "-1",
        })
    }
}

#[derive(Default, Debug, Clone, PartialEq)]
pub(crate) struct Data {
    class: Class,
    value: Real,
    superlative: Option<TimeValue<RealArray<2>>>,
}

impl Data {
    pub(crate) fn get_class(&self) -> Class {
        self.class.clone()
    }

    pub(crate) fn get_value(&self) -> Real {
        self.value
    }

    pub(crate) fn get_superlative(&self) -> Option<TimeValue<RealArray<2>>> {
        self.superlative.clone()
    }
}

impl EventData for Data {}

impl Display for Data {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{0},{1}", self.class, self.value))
    }
}

type BasicMuonEvent = (Real, Data);

type SuperlativeValue = TimeValue<Real>;

type SuperlativeDiff = TimeValue<RealArray<2>>;

#[derive(Clone, Debug, PartialEq)]
enum Mode {
    Rise,
    Fall,
}

#[derive(Clone, Debug)]
struct State(Mode, SuperlativeValue, SuperlativeDiff);

impl State {
    fn from_mode(mode: Option<Mode>, time: Real, value: Real) -> Option<Self> {
        mode.map(|mode| match mode {
            Mode::Rise => State(
                Mode::Rise,
                SuperlativeValue { time, value },
                SuperlativeDiff {
                    time,
                    value: RealArray::new([value, Real::default()]),
                },
            ),
            Mode::Fall => State(
                Mode::Fall,
                SuperlativeValue { time, value },
                SuperlativeDiff {
                    time,
                    value: RealArray::new([value, Real::default()]),
                },
            ),
        })
    }
}

#[derive(Default, Clone)]
pub(crate) struct BasicMuonDetector {
    // Value of the derivative at which an event is said to have been detected
    // Time for which the voltage should rise for the rise to be considered genuine.
    onset_detector: ThresholdDetector<UpperThreshold>,
    // Value of the derivative at which an event is said to have peaked
    // Time for which the voltage should drop for the peak to be considered genuine
    fall_detector: ThresholdDetector<LowerThreshold>,
    // Value of the derivative at which an event is said to have finished
    // Time for which the voltage should level for the finish to be considered genuine
    termination_detector: ThresholdDetector<UpperThreshold>,

    // If a change in signal behavior is detected then it is recorded in potential_mode.
    //If the change lasts the requisite duration then the mode is changed.
    state: Option<State>,
}

impl BasicMuonDetector {
    pub(crate) fn new(
        onset: &ThresholdDuration,
        fall: &ThresholdDuration,
        termination: &ThresholdDuration,
    ) -> Self {
        Self {
            onset_detector: ThresholdDetector::new(onset),
            fall_detector: ThresholdDetector::new(fall),
            termination_detector: ThresholdDetector::new(termination),
            ..Default::default()
        }
    }
}

impl Detector for BasicMuonDetector {
    type TracePointType = (Real, RealArray<2>);
    type EventPointType = (Real, Data);

    fn signal(&mut self, time: Real, value: RealArray<2>) -> Option<BasicMuonEvent> {
        match &mut self.state {
            Some(State(Mode::Rise, peak, steepest_rise)) => {
                //  Update Steepest Rise
                if value[1] >= steepest_rise.value[1] {
                    steepest_rise.time = time;
                    steepest_rise.value = value;
                }
                //  Update Peak
                if value[0] >= peak.value {
                    peak.time = time;
                    peak.value = value[0];
                }
                if self.fall_detector.signal(time, value[1]).is_some() {
                    let event = (
                        peak.time,
                        Data {
                            class: Class::Peak,
                            value: peak.value,
                            superlative: Some(steepest_rise.clone()),
                        },
                    );
                    self.state = State::from_mode(Some(Mode::Fall), time, value[0]);
                    Some(event)
                } else {
                    None
                }
            }
            Some(State(Mode::Fall, nadir, sharpest_fall)) => {
                if value[1] <= sharpest_fall.value[1] {
                    sharpest_fall.time = time;
                    sharpest_fall.value = value;
                }
                //  Update Nadir
                if value[0] <= nadir.value {
                    nadir.time = time;
                    nadir.value = value[0];
                }
                if self.termination_detector.signal(time, value[1]).is_some() {
                    let event = (
                        nadir.time,
                        Data {
                            class: Class::End,
                            value: nadir.value,
                            superlative: Some(sharpest_fall.clone()),
                        },
                    );
                    self.state = State::from_mode(None, time, value[0]);
                    Some(event)
                } else {
                    None
                }
            }
            None => {
                if self.onset_detector.signal(time, value[1]).is_some() {
                    let event = (
                        time,
                        Data {
                            class: Class::Onset,
                            value: value[0],
                            superlative: None,
                        },
                    );
                    self.state = State::from_mode(Some(Mode::Rise), time, value[0]);
                    Some(event)
                } else {
                    None
                }
            }
        }
    }
}

#[derive(Default, Clone, Debug)]
enum AssemblerMode {
    #[default]
    Waiting,
    Rising {
        start: TimeValue<Real>,
    },
    Falling {
        start: TimeValue<Real>,
        steepest_rise: Option<TimeValue<RealArray<2>>>,
        peak: TimeValue<Real>,
    },
}

#[derive(Default, Clone)]
pub(crate) struct BasicMuonAssembler {
    mode: AssemblerMode,
}

impl Assembler for BasicMuonAssembler {
    type DetectorType = BasicMuonDetector;

    fn assemble_pulses(&mut self, source: (Real, Data)) -> Option<Pulse> {
        match self.mode.clone() {
            AssemblerMode::Waiting => match source.get_data().get_class() {
                Class::Onset => {
                    let start = TimeValue {
                        time: source.get_time(),
                        value: source.get_data().get_value(),
                    };
                    self.mode = AssemblerMode::Rising { start };
                    None
                }
                _ => None,
            },
            AssemblerMode::Rising { start } => match source.get_data().get_class() {
                Class::Peak => {
                    let peak = TimeValue::<Real> {
                        time: source.get_time(),
                        value: source.get_data().get_value(),
                    };
                    self.mode = AssemblerMode::Falling {
                        start,
                        steepest_rise: source.get_data().get_superlative(),
                        peak,
                    };
                    None
                }
                _ => None,
            },
            AssemblerMode::Falling {
                start,
                steepest_rise,
                mut peak,
            } => match source.get_data().get_class() {
                Class::End => {
                    self.mode = AssemblerMode::Waiting;
                    Some(TimeValue {
                        time: source.get_time(),
                        value: source.get_data().get_value(),
                    })
                }
                _ => None,
            }
            .map(|end| {
                let mut steepest_rise = steepest_rise.unwrap_or_default();
                let mut sharpest_fall = source.get_data().get_superlative().unwrap_or_default();

                let gradient = (peak.time - start.time) / (end.time - start.time);
                peak.value -= (peak.value - start.value) * gradient;
                steepest_rise.value[0] -= (steepest_rise.value[0] - start.value) * gradient;
                sharpest_fall.value[0] -= (sharpest_fall.value[0] - start.value) * gradient;

                Pulse {
                    start: start.into(),
                    peak: peak.into(),
                    end: end.into(),
                    steepest_rise: steepest_rise.into(),
                    sharpest_fall: sharpest_fall.into(),
                }
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pulse_detection::{
        datatype::tracevalue::TraceArray, window::FiniteDifferences, EventFilter, WindowFilter,
    };

    #[test]
    fn test_gate_zero_threshold() {
        let data = [4, 3, 2, 5, 6, 1, 5, 7, 2, 4];
        let detector = BasicMuonDetector::new(
            &ThresholdDuration {
                threshold: 1.0,
                cool_off: 0,
                duration: 1,
            },
            &ThresholdDuration {
                threshold: 1.0,
                cool_off: 0,
                duration: 1,
            },
            &ThresholdDuration {
                threshold: 1.0,
                cool_off: 0,
                duration: 1,
            },
        );
        let mut iter = data
            .into_iter()
            .enumerate()
            .map(|(i, v)| (i as Real, v as Real))
            .window(FiniteDifferences::<2>::new())
            .events(detector);
        assert_eq!(
            iter.next(),
            Some((
                3.0,
                Data {
                    class: Class::Onset,
                    value: 5.0,
                    superlative: None
                }
            ))
        );
        assert_eq!(
            iter.next(),
            Some((
                4.0,
                Data {
                    class: Class::Peak,
                    value: 6.0,
                    superlative: Some(TimeValue {
                        time: 4.0,
                        value: TraceArray([6.0, 1.0])
                    })
                }
            ))
        );
        assert_eq!(
            iter.next(),
            Some((
                5.0,
                Data {
                    class: Class::End,
                    value: 1.0,
                    superlative: Some(TimeValue {
                        time: 5.0,
                        value: TraceArray([1.0, 0.0])
                    })
                }
            ))
        );
        assert_eq!(
            iter.next(),
            Some((
                7.0,
                Data {
                    class: Class::Onset,
                    value: 7.0,
                    superlative: None
                }
            ))
        );
        assert_eq!(
            iter.next(),
            Some((
                7.0,
                Data {
                    class: Class::Peak,
                    value: 7.0,
                    superlative: Some(TimeValue {
                        time: 7.0,
                        value: TraceArray([7.0, 0.0])
                    })
                }
            ))
        );
        assert_eq!(
            iter.next(),
            Some((
                8.0,
                Data {
                    class: Class::End,
                    value: 2.0,
                    superlative: Some(TimeValue {
                        time: 8.0,
                        value: TraceArray([2.0, 0.0])
                    })
                }
            ))
        );
        assert_eq!(iter.next(), None);
    }
}

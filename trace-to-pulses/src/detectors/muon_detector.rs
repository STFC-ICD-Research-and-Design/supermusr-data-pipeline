use std::collections::VecDeque;
use std::fmt::Display;

use crate::events::Event;
use crate::trace_iterators::feedback::FeedbackParameter;
use crate::tracedata::EventData;
use crate::{ode, Detector, Real, RealArray};

use super::FeedbackDetector;

#[derive(Default, Debug, Clone)]
pub struct ODEData {
    start: Real,
    end: Real,
    //quadratic : Real,
    //linear : Real,
    //constant : Real,
    lambda: Real,
    theta: Real,
    coef_cos: Real,
    coef_sin: Real,
    coef_const: Real,
    residual: Real,
}
impl ODEData {
    pub fn new(
        start: Real,
        end: Real,
        //quadratic : Real, linear : Real, constant : Real,
        lambda: Real,
        theta: Real,
        coef_cos: Real,
        coef_sin: Real,
        coef_const: Real,
        residual: Real,
    ) -> Self {
        ODEData {
            start,
            end,
            //quadratic, linear, constant,
            lambda,
            theta,
            coef_cos,
            coef_sin,
            coef_const,
            residual,
        }
    }
    pub fn value(&self, time: Real) -> Real {
        Real::exp(self.lambda * time)
            * (self.coef_cos * Real::cos(self.theta * time)
                + self.coef_sin * Real::sin(self.theta * time))
            + self.coef_const
    }
    pub fn deriv1(&self, time: Real) -> Real {
        Real::exp(self.lambda * time)
            * ((self.lambda * self.coef_cos + self.theta * self.coef_sin)
                * Real::cos(self.theta * time)
                + (self.lambda * self.coef_sin - self.theta * self.coef_cos)
                    * Real::sin(self.theta * time))
    }
    pub fn deriv2(&self, time: Real) -> Real {
        Real::exp(self.lambda * time)
            * (((self.lambda * self.lambda - self.theta * self.theta) * self.coef_cos
                + 2.0 * self.lambda * self.theta * self.coef_sin)
                * Real::cos(self.theta * time)
                + ((self.lambda * self.lambda - self.theta * self.theta) * self.coef_sin
                    + 2.0 * self.lambda * self.theta * self.coef_cos)
                    * Real::sin(self.theta * time))
    }
}
impl EventData for ODEData {}

impl Display for ODEData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "{0},{1},{2},{3},{4},{5},{6},{7}",
            self.start,
            self.end,
            self.residual,
            self.lambda,
            self.theta,
            self.coef_cos,
            self.coef_sin,
            self.coef_const,
        ))
    }
}

pub type ODEEvent = Event<Real, ODEData>;

#[derive(Default, Clone, PartialEq)]
enum Mode {
    #[default]
    Flat,
    Rising,
    Peaking,
    Falling,
}

#[derive(Default, Clone)]
pub struct MuonDetector {
    mode: Mode,
    active: bool,
    start: Real,
    //baseline : Real,
    //peak: (Real,Real),
    //max_slope: (Real,Real,Real),
    //min_slope: (Real,Real,Real),
    estimator: ode::ParameterEstimator,
    recent_muons: VecDeque<ODEData>,
    //peak_finder: LocalExtremumDetector,
    //change_finder: ChangeDetector,

    //threshold: Real,
    //slope_threshold: Real,
}

impl MuonDetector {
    pub fn new() -> Self {
        Self {
            active: false,
            /*threshold, slope_threshold, change_finder: ChangeDetector::new(slope_threshold),*/
            ..Default::default()
        }
    }
    pub fn init(&mut self, time: Real) {
        println!("Event Initialised at {time}");
        //self.baseline = baseline;
        //self.max_slope = (0.0, self.baseline, baseslope);
        //self.peak = (0.0, self.baseline);
        self.estimator.clear();
        self.start = time;
        self.active = true;
    }
    pub fn new_event(&mut self, time: Real) -> Option<ODEEvent> {
        println!("Event Finalised at {time}");
        let result = self.estimator.get_parameters();
        //Which order do the coefficients come in?
        match result {
            Ok(((lambda, theta), coefs, residual)) => {
                let data = ODEData {
                    start: self.start,
                    end: time,
                    residual,
                    lambda,
                    theta,
                    coef_cos: coefs.0,
                    coef_sin: coefs.1,
                    coef_const: coefs.2,
                };
                self.recent_muons.push_back(data.clone());
                Some(data.make_event(self.start))
            }
            _ => None,
        }
    }
}

impl Detector for MuonDetector {
    type TimeType = Real;
    type ValueType = RealArray<3>;
    type DataType = ODEData;

    fn signal(&mut self, time: Self::TimeType, diff: Self::ValueType) -> Option<ODEEvent> {
        match self.mode {
            Mode::Flat => {
                if diff[1] > 0.0 {
                    self.mode = Mode::Rising
                } else if diff[1] < -0.0 {
                    self.mode = Mode::Falling
                }
            }
            Mode::Rising => {
                if diff[1] < -0.0 {
                    self.mode = Mode::Peaking
                }
            }
            Mode::Peaking => {
                if diff[1] > 0.0 {
                    self.mode = Mode::Rising
                } else if diff[1] < -0.0 {
                    self.mode = Mode::Falling
                }
            }
            Mode::Falling => {
                if diff[1] > 0.0 {
                    self.mode = Mode::Flat
                }
            }
        }
        if self.active {
            if self.mode == Mode::Flat {
                self.active = false;
                self.new_event(time)
            } else {
                self.estimator.push(diff[0], diff[1], diff[2]);
                None
            }
        } else {
            if self.mode == Mode::Rising {
                self.init(time);
            }
            None
        }
    }
}

impl FeedbackDetector for MuonDetector {
    fn is_active(&self) -> bool {
        self.active
    }
    fn modify_parameter(
        &mut self,
        time: Self::TimeType,
        param: &FeedbackParameter<Self::ValueType>,
    ) {
        while let Some(pulse) = self.recent_muons.front() {
            if pulse.value(time) > 1.0 {
                break;
            }
            self.recent_muons.pop_front();
        }
        let val = self
            .recent_muons
            .iter()
            .map(|pulse| pulse.value(time + 1.))
            .sum::<Real>();
        let deriv1 = self
            .recent_muons
            .iter()
            .map(|pulse| pulse.deriv1(time + 1.))
            .sum::<Real>();
        let deriv2 = self
            .recent_muons
            .iter()
            .map(|pulse| pulse.deriv2(time + 1.))
            .sum::<Real>();
        param.set(RealArray::<3>::new([-val, -deriv1, -deriv2]));
    }
}

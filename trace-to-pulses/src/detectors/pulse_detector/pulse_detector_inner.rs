use std::collections::VecDeque;
use std::f64::consts::PI;
use std::fmt::Debug;
use std::fmt::Display;
use std::marker::PhantomData;

use crate::change_detector::ChangeDetector;
use crate::change_detector::{ChangeClass, ChangeData};
use crate::events::Event;
use crate::peak_detector::LocalExtremumClass;
use crate::peak_detector::LocalExtremumData;
use crate::peak_detector::{LocalExtremumDetector, PeakData};
use crate::trace_iterators::feedback::FeedbackParameter;
use crate::tracedata::{EventData, Stats};
use crate::{Detector, Real};

use super::PulseModel;
use super::{EventValuedDetector, FeedbackDetector};

#[derive(Default, Debug, Clone)]
pub struct Gaussian {
    amplitude: Real,
    mean: Real,
    standard_deviation: Real,
}
impl Gaussian {
    pub fn new(amplitude: Real, mean: Real, standard_deviation: Real) -> Self {
        Self {
            amplitude,
            mean,
            standard_deviation,
        }
    }
}

impl PulseModel for Gaussian {
    fn get_value_at(&self, t: Real) -> Real {
        self.amplitude * (-0.5 * ((t - self.mean) / self.standard_deviation).powi(2)).exp()
    }
    fn get_derivative_at(&self, t: Real) -> Real {
        self.get_value_at(t) * (t - self.mean) / self.standard_deviation.powi(2)
    }
    fn get_second_derivative_at(&self, t: Real) -> Real {
        self.get_value_at(t) * (((t - self.mean) / self.standard_deviation).powi(2) - 1.)
            / self.standard_deviation.powi(2)
    }
    fn from_data(peak_time: Real, peak_value: Real, area_under_curve: Real) -> Self {
        let standard_deviation_estimate = 2. * area_under_curve / Real::sqrt(2. * PI) / peak_value;
        Self {
            amplitude: peak_value,
            mean: peak_time,
            standard_deviation: standard_deviation_estimate,
        }
    }
    fn get_effective_interval(&self, bound: Real) -> (Real, Real) {
        (
            self.mean - self.standard_deviation * bound,
            self.mean + self.standard_deviation * bound,
        )
    }

    fn from_basic(mean: Real, amplitude: Real) -> Self {
        Gaussian {
            mean,
            amplitude,
            standard_deviation: 1.,
        }
    }
}
impl Display for Gaussian {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "{0},{1},{2}",
            self.amplitude, self.mean, self.standard_deviation
        ))
    }
}

#[derive(Default, Debug, Clone)]
pub struct PulseData<Model: PulseModel> {
    model: Model,
    _uncertainty: Option<(Model, Model)>,
    effective_interval: Option<(Real, Real)>,
    cache: Vec<Real>,
}
impl<Model: PulseModel> PulseData<Model> {
    pub fn new(
        model: Model,
        _uncertainty: Option<(Model, Model)>,
        effective_interval: Option<(Real, Real)>,
    ) -> Self {
        PulseData {
            model,
            _uncertainty,
            effective_interval,
            ..Default::default()
        }
    }
    pub fn new_basic(mean: Real, amplitude: Real) -> Self {
        PulseData {
            model: Model::from_basic(mean, amplitude),
            ..Default::default()
        }
    }
    pub fn with_cache(
        model: Model,
        _uncertainty: Option<(Model, Model)>,
        effective_bound: Real,
    ) -> Self {
        let effective_interval = Some(model.get_effective_interval(effective_bound));
        PulseData {
            model,
            _uncertainty,
            effective_interval,
            ..Default::default()
        }
    }
    pub fn get_effective_value_at(&self, time: Real) -> Real {
        if self.is_effective_nonzero_at(time) {
            if self.cache.is_empty() {
                self.model.get_value_at(time)
            } else {
                self.cache
                    [(time - Real::ceil(self.effective_interval.unwrap_or_default().0)) as usize]
            }
        } else {
            Real::default()
        }
    }
    fn _build_cache(&mut self) {
        if let Some((start, end)) = self.effective_interval {
            self.cache = Vec::<f64>::with_capacity(Real::ceil(end - start) as usize);
            for i in 0..Real::ceil(end - start) as usize {
                self.cache
                    .push(self.model.get_value_at(Real::ceil(start) + i as Real));
            }
        }
    }
    pub fn is_effective_nonzero_at(&self, t: Real) -> bool {
        self.effective_interval
            .map(|eff_intv| eff_intv.0 <= t && t <= eff_intv.1)
            .unwrap_or(true)
    }
    pub fn get_model(&self) -> &Model {
        &self.model
    }
    pub fn get_model_mut(&mut self) -> &mut Model {
        &mut self.model
    }
}

impl<Model: PulseModel> EventData for PulseData<Model> {}
impl<Model: PulseModel> Display for PulseData<Model> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{0}", self.model)
    }
}
pub type PulseEvent<Model> = Event<Real, PulseData<Model>>;

impl<Model: PulseModel> From<Event<Real, PeakData>> for PulseEvent<Model> {
    fn from(value: Event<Real, PeakData>) -> Self {
        PulseData::<Model>::new_basic(value.time, value.data.get_value().unwrap_or_default())
            .make_event(value.time)
    }
}

#[derive(Default, Clone, PartialEq)]
enum Mode {
    #[default]
    Waiting,
    Started,
}

#[derive(Default, Clone)]
pub struct PulseDetector<Model: PulseModel, Input: Detector> {
    mode: Mode,
    area_under_curve: Real,
    bound: Real,
    prev_pulses: VecDeque<PulseData<Model>>,
    data: Vec<Real>,
    max: Real,
    start: Real,
    phantom: PhantomData<Input>,
}

impl<Model: PulseModel, Input: Detector> PulseDetector<Model, Input> {
    pub fn new(bound: Real) -> PulseDetector<Model, Input> {
        PulseDetector {
            bound,
            ..Default::default()
        }
    }

    fn remove_distant_pulses(&mut self, time: Real) {
        while let Some(pulse) = self.prev_pulses.front() {
            if pulse.is_effective_nonzero_at(time) {
                break;
            }
            // LOG
            //log::info!("Old pulse removed from cache: {0:?}",pulse);
            self.prev_pulses.pop_front();
        }
    }
}

impl<Model: PulseModel, Input: Detector> Detector for PulseDetector<Model, Input> {
    type TimeType = Real;
    type ValueType = Stats;
    type DataType = PulseData<Model>;

    fn signal(&mut self, _time: Real, value: Self::ValueType) -> Option<PulseEvent<Model>> {
        self.area_under_curve += value.mean;
        self.data.push(value.mean);
        self.max = Real::max(self.max, value.mean);
        None
    }
}
impl<Model: PulseModel, Input: Detector> FeedbackDetector for PulseDetector<Model, Input> {
    fn is_active(&self) -> bool {
        self.mode == Mode::Started
    }
    fn modify_parameter(&mut self, time: Real, param: &FeedbackParameter<Self::ValueType>) {
        self.remove_distant_pulses(time);
        //let r = Rc::strong_count(&param.clone().unwrap().0);
        // LOG
        //log::info!("Number of references: {0:?}",r);
        let val = self
            .prev_pulses
            .iter()
            .map(|pulse| pulse.get_effective_value_at(time + 1.))
            .sum::<Real>();
        // LOG
        //log::info!("New correction calculated: {val:?} from {0} pulses", self.prev_pulses.len());
        param.set(-val);
    }
}

impl<Model: PulseModel> EventValuedDetector for PulseDetector<Model, ChangeDetector> {
    type DataValueType = ChangeData;

    fn on_event(
        &mut self,
        event: Event<Self::TimeType, Self::DataValueType>,
    ) -> Option<Event<Self::TimeType, PulseData<Model>>> {
        if event.get_data().get_class() == ChangeClass::Rising {
            self.area_under_curve = 0.;
            self.data.clear();
            self.max = Real::default();
            self.start = event.get_time();
            return None;
        }
        if event.get_data().get_class() == ChangeClass::Flat {
            return None;
        }
        let data = PulseData::with_cache(
            //Model::from_data(event.get_time(),event.get_data().get_value_from()?,self.area_under_curve),
            Model::from_data2(self.data.clone(), self.start, self.max),
            None,
            self.bound,
        );
        self.data.clear();
        self.max = Real::default();
        self.area_under_curve = 0.;
        // LOG
        //log::info!("{time}: Pulse data created {data} with window {0}",3.*sigma);
        self.prev_pulses.push_back(data.clone());
        self.area_under_curve = 0.;
        Some(data.make_event(event.get_time()))
    }
}

impl<Model: PulseModel> EventValuedDetector for PulseDetector<Model, LocalExtremumDetector<Real>> {
    type DataValueType = LocalExtremumData;

    fn on_event(
        &mut self,
        event: Event<Self::TimeType, Self::DataValueType>,
    ) -> Option<Event<Self::TimeType, PulseData<Model>>> {
        use LocalExtremumClass as L;
        if event.get_data().get_class() == L::LocalMin {
            let new_event = if self.mode == Mode::Started {
                Some(
                    PulseData::with_cache(
                        Model::from_data2(self.data.clone(), self.start, self.max),
                        None,
                        self.bound,
                    )
                    .make_event(event.get_time()),
                )
            } else {
                self.mode = Mode::Started;
                None
            };
            self.data.clear();
            self.area_under_curve = 0.;
            self.max = Real::default();
            self.start = event.get_time();
            new_event
        } else {
            None
        }
    }
}

use std::{f64::consts::PI, fmt::Display};

use optimization::Minimizer;

use crate::Real;

use super::PulseModel;

#[derive(Default, Debug, Clone)]
pub struct BiexponentialObjective {
    peak: Real,
    values: Vec<Real>,
}

impl BiexponentialObjective {
    fn new(peak: Real, values: Vec<Real>) -> Self {
        Self { peak, values }
    }
    fn get_amplitude(&self, rise: Real, decay: Real) -> Real {
        let b = decay / rise;
        self.peak / (b.powf(1.0 / (1.0 - b)) - b.powf(1.0 / (1.0 / b - 1.0)))
    }
    fn get_value_at(&self, rise: Real, decay: Real) -> Real {
        let amplitude = self.get_amplitude(rise, decay);
        self.values
            .iter()
            .enumerate()
            .map(|(t, f)| {
                let t = t as Real;
                (amplitude * (-t / decay).exp() - (-t / rise).exp() - f).powi(2)
            })
            .sum()
    }
    /*fn get_amp_deriv_at(&self, amplitude : Real, rise : Real, decay : Real) -> Real {
        2.*self.values.iter().enumerate().map(|(t,f)|
        amplitude*((t as Real/rise).exp() - (t as Real/decay).exp())*((t as Real/rise).exp() - (t as Real/decay).exp() - f)
        ).sum::<Real>()
    }*/
    fn get_rise_deriv_at(&self, rise: Real, decay: Real) -> Real {
        let amplitude = self.get_amplitude(rise, decay);
        2. * self
            .values
            .iter()
            .enumerate()
            .map(|(t, f)| {
                let t = t as Real;
                -(amplitude * (-t / decay).exp() - (-t / rise).exp() - f)
                    * amplitude
                    * (-t / rise).exp()
                    / rise
            })
            .sum::<Real>()
    }
    fn get_decay_deriv_at(&self, rise: Real, decay: Real) -> Real {
        let amplitude = self.get_amplitude(rise, decay);
        2. * self
            .values
            .iter()
            .enumerate()
            .map(|(t, f)| {
                let t = t as Real;
                (amplitude * (-t / decay).exp() - (-t / rise).exp() - f)
                    * amplitude
                    * (-t / decay).exp()
                    / decay
            })
            .sum::<Real>()
    }
}

impl optimization::Function for BiexponentialObjective {
    fn value(&self, position: &[f64]) -> f64 {
        self.get_value_at(position[0], position[1])
    }
}

impl optimization::Function1 for BiexponentialObjective {
    fn gradient(&self, position: &[f64]) -> Vec<f64> {
        vec![
            //self.get_amp_deriv_at(position[0],position[1],position[2]),
            self.get_rise_deriv_at(position[0], position[1]),
            self.get_decay_deriv_at(position[0], position[1]),
        ]
    }
}

#[derive(Default, Debug, Clone)]
pub struct Biexponential {
    amplitude: Real,
    start_time: Real,
    rise_time: Real,
    decay_time: Real,
}
impl Biexponential {
    pub fn new(amplitude: Real, start_time: Real, rise_time: Real, decay_time: Real) -> Self {
        Self {
            amplitude,
            start_time,
            rise_time,
            decay_time,
        }
    }
}

impl PulseModel for Biexponential {
    fn get_value_at(&self, t: Real) -> Real {
        if t < self.start_time {
            return Real::default();
        }
        self.amplitude
            * (((t - self.start_time) / self.rise_time).exp()
                - ((t - self.start_time) / self.decay_time).exp())
    }
    fn get_derivative_at(&self, t: Real) -> Real {
        if t < self.start_time {
            return Real::default();
        }
        self.amplitude
            * (((t - self.start_time) / self.decay_time).exp() / self.decay_time
                - ((t - self.start_time) / self.rise_time).exp() / self.rise_time)
    }
    fn get_second_derivative_at(&self, t: Real) -> Real {
        if t < self.start_time {
            return Real::default();
        }
        self.amplitude
            * (((t - self.start_time) / self.rise_time).exp() / self.rise_time / self.rise_time
                - ((t - self.start_time) / self.decay_time).exp()
                    / self.decay_time
                    / self.decay_time)
    }
    fn from_data(_peak_time: Real, peak_value: Real, area_under_curve: Real) -> Self {
        let _standard_deviation_estimate = 2. * area_under_curve / Real::sqrt(2. * PI) / peak_value;
        Self::default()
    }
    fn get_effective_interval(&self, bound: Real) -> (Real, Real) {
        (self.start_time, self.start_time + bound)
    }

    fn from_basic(mean: Real, amplitude: Real) -> Self {
        Biexponential {
            start_time: mean,
            amplitude,
            rise_time: 1.,
            decay_time: 1.,
        }
    }
    fn from_data2(data: Vec<Real>, start: Real, peak: Real) -> Self {
        let gd = optimization::GradientDescent::new().max_iterations(Some(1000));
        let ob = BiexponentialObjective::new(peak, data);
        let sol = gd.minimize(&ob, vec![10., 15.]);
        Biexponential::new(
            ob.get_amplitude(sol.position[0], sol.position[1]),
            start,
            sol.position[0],
            sol.position[1],
        )
    }
}
impl Display for Biexponential {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "{0},{1},{2},{3}",
            self.amplitude, self.start_time, self.rise_time, self.decay_time
        ))
    }
}

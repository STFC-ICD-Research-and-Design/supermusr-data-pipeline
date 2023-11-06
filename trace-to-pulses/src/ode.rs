use anyhow::{anyhow, Result};
use lstsq;
use nalgebra::{self as na, OMatrix, OVector, U3};

use crate::Real;

/*
#[derive(Default, Debug, Clone)]
pub enum ODESolution {
    #[default]Trivial,
    BiExp{
        amp_1 : Real,
        amp_2 : Real,
        lambda_1 : Real,
        lambda_2 : Real,
        baseline : Real,
    },
    SinCos{
        amp_1 : Real,
        amp_2 : Real,
        lambda : Real,
        theta : Real,
        baseline : Real,
    }
}
impl ODESolution {
    pub fn value(&self, t : Real) -> Real {
        match self {
            ODESolution::BiExp { amp_1, amp_2, lambda_1, lambda_2, baseline } =>
                amp_1*Real::exp(lambda_1*t) + amp_2*Real::exp(lambda_2*t) + baseline,
            ODESolution::SinCos { amp_1, amp_2, lambda, theta, baseline } =>
                Real::exp(lambda*t)*(amp_1*Real::cos(theta*t) + amp_2*Real::sin(theta*t)) + baseline,
            _ => Real::default(),
        }

    }
    fn calc_solution(&mut self, peak : (Real,Real), max_slope : (Real,Real,Real)) -> bool {
        match self {
            ODESolution::BiExp { amp_1, amp_2, lambda_1, lambda_2, baseline } => false,
            ODESolution::SinCos { amp_1, amp_2, lambda, theta, baseline } => {
                // 0 = amp_1*(lambda*lambda - theta*theta - 2*lambda*theta*max_slope.0) + amp_2*((lambda*lambda - theta*theta)*max_slope.0 + 2*lambda*theta)
                // max_slope.1*Real::exp(-lambda*max_slope.0) =
                //  amp_1*((lambda*lambda - theta*theta)*Real::cos(theta*max_slope.0) - 2*lambda*theta*Real::sin(theta*max_slope.0))
                // + amp_2*(2*lambda*theta*Real::cos(theta*max_slope.0) + (lambda*lambda - theta*theta)*Real::sin(theta*max_slope.0))

                // amp_1 = amp_2*((lambda*lambda - theta*theta)*max_slope.0 + 2*lambda*theta)/(-lambda*lambda + theta*theta + 2*lambda*theta*max_slope.0)

                // max_slope.1*Real::exp(-lambda*max_slope.0)/[
                //    ((lambda*lambda - theta*theta)*Real::cos(theta*max_slope.0) - 2*lambda*theta*Real::sin(theta*max_slope.0))*((lambda*lambda - theta*theta)*max_slope.0 + 2*lambda*theta)/(-lambda*lambda + theta*theta + 2*lambda*theta*max_slope.0)
                // + (2*lambda*theta*Real::cos(theta*max_slope.0) + (lambda*lambda - theta*theta)*Real::sin(theta*max_slope.0))
                // ] =
                //  amp_2
                let l = *lambda;
                let t = *theta;
                let d = Real::powi(l,2) - Real::powi(t,2);
                let e = 2.0*l*t;
                let (max_dy_t,_,max_dy) = max_slope;
                let max_y_t = peak.0;
                let cos = Real::cos(t*max_dy_t);
                let sin = Real::sin(t*max_dy_t);
                *amp_2 = max_dy*Real::exp(-l*max_dy_t)*(e*max_dy_t - d)/((d*d + e*e)*(cos*max_dy_t - sin));
                *amp_1 = -max_dy*Real::exp(-l*max_dy_t)*(d*max_dy_t + e)/((d*d + e*e)*(cos*max_dy_t - sin));
                *baseline = peak.1 - Real::exp(l*max_y_t)*(*amp_1*Real::cos(t*max_y_t) + *amp_2*Real::sin(t*max_y_t));
                Real::is_infinite(*baseline) || Real::is_infinite(*amp_1) || Real::is_infinite(*amp_2)
            },
            _ => false,
        }
    }
}
impl Display for ODESolution {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ODESolution::BiExp { amp_1, amp_2, lambda_1, lambda_2, baseline } =>
                f.write_fmt(format_args!("1,{amp_1},{amp_2},{lambda_1},{lambda_2},{baseline}")),
            ODESolution::SinCos { amp_1, amp_2, lambda, theta, baseline } =>
                f.write_fmt(format_args!("-1,{amp_1},{amp_2},{lambda},{theta},{baseline}")),
            _ => f.write_fmt(format_args!("")),
        }
    }
}*/

#[derive(Default)]
pub struct MonicQuadratic {
    quadratic: Real,
    linear: Real,
    constant: Real,
}

impl MonicQuadratic {
    pub fn new(quadratic: Real, linear: Real, constant: Real) -> Self {
        Self {
            quadratic,
            linear,
            constant,
        }
    }
    pub fn get_coefficients(&self) -> (Real, Real, Real) {
        (self.quadratic, self.linear, self.constant)
    }
    pub fn discriminant(&self) -> Real {
        self.linear * self.linear - 4. * self.constant * self.quadratic
    }
    pub fn calc_solutions(&self) -> (Real, Real) {
        let discr_sqrt = self.discriminant().sqrt();
        (
            (-self.linear + discr_sqrt) * 0.5 / self.quadratic,
            (-self.linear - discr_sqrt) * 0.5 / self.quadratic,
        )
    }
    pub fn calc_complex_solutions(&self) -> ((Real, Real), (Real, Real)) {
        let discr = self.discriminant();
        if discr < 0. {
            let discr_sqrt = (-discr).sqrt();
            (
                (
                    -self.linear * 0.5 / self.quadratic,
                    discr_sqrt * 0.5 / self.quadratic,
                ),
                (
                    -self.linear * 0.5 / self.quadratic,
                    -discr_sqrt * 0.5 / self.quadratic,
                ),
            )
        } else {
            let discr_sqrt = discr.sqrt();
            (
                ((-self.linear + discr_sqrt) * 0.5 / self.quadratic, 0.),
                ((-self.linear - discr_sqrt) * 0.5 / self.quadratic, 0.),
            )
        }
    }
}

#[derive(Default, Clone)]
pub struct ParameterEstimator {
    y: Vec<Real>,
    dy: Vec<Real>,
    dy2: Vec<Real>,
}
/*impl Default for ParameterEstimator {
    fn default() -> Self {
        Self { a: Default::default(), b: Default::default() }
    }
}*/
//use ndarray::{array, Array1, Array2, ArrayView};
//use ndarray_linalg::{LeastSquaresSvd, LeastSquaresSvdInto, LeastSquaresSvdInPlace};

impl ParameterEstimator {
    pub fn push(&mut self, y: Real, dy: Real, dy2: Real) {
        self.y.push(y);
        self.dy.push(dy);
        self.dy2.push(dy2);
    }
    pub fn clear(&mut self) {
        self.y.clear();
        self.dy.clear();
        self.dy2.clear();
    }
    pub fn get_parameters(&self) -> Result<((Real, Real), (Real, Real, Real), Real)> {
        if self.y.len() < 5 {
            return Err(anyhow!("Insufficient Data {}", self.y.len()));
        }
        let a_y = OVector::<f64, na::Dyn>::from_row_slice(self.y.as_slice());
        let a_dy = OVector::<f64, na::Dyn>::from_row_slice(self.dy.as_slice());
        let a_dy2 = OVector::<f64, na::Dyn>::from_row_slice(self.dy2.as_slice());
        let a = OMatrix::<Real, na::Dyn, U3>::from_columns(&[a_y.clone(), a_dy, a_dy2]);

        let b = OVector::<f64, na::Dyn>::from_row_slice(&vec![1.0; self.y.len()]);

        let epsilon = 1e-16;
        // -y'' = x.0 y + x.1 y'
        let sol = lstsq::lstsq(&a, &b, epsilon).map_err(|s| anyhow!(s))?;
        let residuals = sol.residuals;
        let x = MonicQuadratic::new(sol.solution[2], sol.solution[1], sol.solution[0]);
        let (root, _) = x.calc_complex_solutions();

        let cos: Vec<Real> = a_y
            .iter()
            .enumerate()
            .map(|(i, _)| Real::exp(-root.0 * i as Real) * Real::cos(0.5 * root.1 * i as Real))
            .collect();
        let sin: Vec<Real> = a_y
            .iter()
            .enumerate()
            .map(|(i, _)| Real::exp(-root.0 * i as Real) * Real::sin(0.5 * root.1 * i as Real))
            .collect();
        let _exp: Vec<Real> = a_y
            .iter()
            .enumerate()
            .map(|(i, _)| Real::exp(-root.0 * i as Real))
            .collect();

        let a = OMatrix::<Real, na::Dyn, U3>::from_columns(&[cos.into(), sin.into(), b]);
        let sol = lstsq::lstsq(&a, &a_y, epsilon).map_err(|s| anyhow!(s))?;

        Ok((
            root,
            (sol.solution[0], sol.solution[1], sol.solution[2]),
            residuals / self.y.len() as Real,
        ))
    }
}

#[cfg(test)]
mod tests {
    use itertools::Itertools;

    //use crate::processing;
    use super::*;

    fn _biexp_peak_time(kappa: Real, rho: Real) -> Real {
        (Real::ln(kappa) - Real::ln(rho)) / (1. / rho - 1. / kappa)
    }
    fn biexp_value(t: Real, kappa: Real, rho: Real) -> Real {
        (Real::exp(-t / kappa) - Real::exp(-t / rho)) + (rand::random::<Real>() - 0.5) * 0.002
    }
    #[test]
    fn test_with_data() {
        let mut pe = ParameterEstimator::default();
        let mut peak = (0.0, 0.0);
        let rho = 2.5;
        let kappa = 13.0;
        let dt = 1.0;
        let n = 10;
        let amplitude = 10.0;
        let y = (0..n)
            .map(|i| amplitude * biexp_value((i as Real) * dt, kappa, rho))
            .collect_vec();
        for i in 1..(n - 1) {
            let dy = (y[i + 1] - y[i]) / dt;
            let dy2 = (y[i - 1] + y[i + 1] - 2.0 * y[i]) / dt / dt;
            pe.push(y[i], dy, dy2);
            if y[i] > peak.1 {
                peak = (i as Real * dt, y[i]);
            }
        }
        /*if let Status::Ok(((xrho, xkappa),(xa,xb),res)) = pe.get_parameters().unwrap() {
            println!("{:?}",(xrho, xkappa));
            println!("{:?}",(xa, xb));
            println!("{:?}",(((y[0] + y[2] - 2.0*y[1])/dt)/(y[1] - y[0]), 1./xrho + 1./xkappa, 1./rho + 1./kappa));
            println!("{:?}",res);
            println!("{peak:?}");
            println!("{0},{1}",biexp_peak_time(kappa,rho),biexp_value(biexp_peak_time(kappa,rho),kappa,rho));
            println!("{0},{1}",biexp_peak_time(xkappa,xrho),biexp_value(biexp_peak_time(xkappa,xrho),xkappa,xrho));
        }*/
    }
}

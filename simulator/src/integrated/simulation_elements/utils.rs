use chrono::Utc;
use rand::{Rng, SeedableRng};
use rand_distr::{Distribution, Exp, Normal};
use serde::Deserialize;
use std::{
    env::{self, VarError},
    num::{ParseFloatError, ParseIntError},
    ops::RangeInclusive,
};
use thiserror::Error;

#[derive(Debug, Error)]
pub(crate) enum JsonFloatError {
    #[error("Cannot Extract Environment Variable")]
    EnvVar(#[from] VarError),
    #[error("Invalid String to Float: {0}")]
    FloatFromStr(#[from] ParseFloatError),
    #[error("Invalid Normal Distribution: {0}")]
    NormalDistribution(#[from] rand_distr::NormalError),
    #[error("Invalid Exponential Distribution: {0}")]
    ExpDistribution(#[from] rand_distr::ExpError),
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum FloatExpression {
    Float(f64),
    FloatEnv(String),
    FloatFunc(Transformation<f64>),
}

impl FloatExpression {
    pub(crate) fn value(&self, frame_index: usize) -> Result<f64, JsonFloatError> {
        match self {
            FloatExpression::Float(v) => Ok(*v),
            FloatExpression::FloatEnv(environment_variable) => {
                Ok(env::var(environment_variable)?.parse()?)
            }
            FloatExpression::FloatFunc(frame_function) => {
                Ok(frame_function.transform(frame_index as f64))
            }
        }
    }
}

#[derive(Debug, Error)]
pub(crate) enum JsonIntError {
    #[error("Cannot Extract Environment Variable")]
    EnvVar(#[from] VarError),
    #[error("Invalid String to Float: {0}")]
    FloatFromStr(#[from] ParseIntError),
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum IntConstant {
    Int(i32),
    IntEnv(String),
}

impl IntConstant {
    pub(crate) fn value(&self) -> Result<i32, JsonIntError> {
        match self {
            IntConstant::Int(v) => Ok(*v),
            IntConstant::IntEnv(environment_variable) => {
                Ok(env::var(environment_variable)?.parse()?)
            }
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum TextConstant {
    Text(String),
    TextEnv(String),
}

impl TextConstant {
    pub(crate) fn value(&self) -> String {
        match self {
            TextConstant::Text(v) => v.clone(),
            TextConstant::TextEnv(environment_variable) => env::var(environment_variable).unwrap(),
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum IntExpression {
    Int(i32),
    IntEnv(String),
    IntFunc(Transformation<i32>),
}

impl IntExpression {
    pub(crate) fn value(&self, frame_index: usize) -> Result<i32, JsonIntError> {
        match self {
            IntExpression::Int(v) => Ok(*v),
            IntExpression::IntEnv(environment_variable) => {
                Ok(env::var(environment_variable)?.parse()?)
            }
            IntExpression::IntFunc(frame_function) => {
                Ok(frame_function.transform(frame_index as i32))
            }
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "kebab-case", tag = "random-type")]
pub(crate) enum FloatRandomDistribution {
    Constant {
        value: FloatExpression,
    },
    Uniform {
        min: FloatExpression,
        max: FloatExpression,
    },
    Normal {
        mean: FloatExpression,
        sd: FloatExpression,
    },
    Exponential {
        lifetime: FloatExpression,
    },
}

impl FloatRandomDistribution {
    pub(crate) fn sample(&self, frame_index: usize) -> Result<f64, JsonFloatError> {
        match self {
            Self::Constant { value } => value.value(frame_index),
            Self::Uniform { min, max } => {
                let val =
                    rand::rngs::StdRng::seed_from_u64(Utc::now().timestamp_subsec_nanos() as u64)
                        .gen_range(min.value(frame_index)?..max.value(frame_index)?);
                Ok(val)
            }
            Self::Normal { mean, sd } => {
                let val = Normal::new(mean.value(frame_index)?, sd.value(frame_index)?)?.sample(
                    &mut rand::rngs::StdRng::seed_from_u64(
                        Utc::now().timestamp_subsec_nanos() as u64
                    ),
                );
                Ok(val)
            }
            Self::Exponential { lifetime } => {
                let val = Exp::new(1.0 / lifetime.value(frame_index)?)?.sample(
                    &mut rand::rngs::StdRng::seed_from_u64(
                        Utc::now().timestamp_subsec_nanos() as u64
                    ),
                );
                Ok(val)
            }
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "kebab-case", tag = "random-type")]
pub(crate) enum IntRandomDistribution {
    Constant {
        value: IntExpression,
    },
    Uniform {
        min: IntExpression,
        max: IntExpression,
    },
}

impl IntRandomDistribution {
    pub(crate) fn sample(&self, frame_index: usize) -> Result<i32, JsonIntError> {
        match self {
            Self::Constant { value } => value.value(frame_index),
            Self::Uniform { min, max } => {
                let seed = Utc::now().timestamp_subsec_nanos() as u64;
                let value = rand::rngs::StdRng::seed_from_u64(seed)
                    .gen_range(min.value(frame_index)?..max.value(frame_index)?);
                Ok(value)
            }
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct Interval<T>
where
    T: Clone,
{
    pub(crate) min: T,
    pub(crate) max: T,
}

impl<T: PartialOrd + Copy> Interval<T> {
    pub(crate) fn range_inclusive(&self) -> RangeInclusive<T> {
        self.min..=self.max
    }

    pub(crate) fn is_in(&self, value: T) -> bool {
        self.range_inclusive().contains(&value)
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct Transformation<T> {
    pub(crate) scale: T,
    pub(crate) translate: T,
}

impl Transformation<f64> {
    pub(crate) fn transform(&self, x: f64) -> f64 {
        x * self.scale + self.translate
    }
}

impl Transformation<i32> {
    pub(crate) fn transform(&self, x: i32) -> i32 {
        x * self.scale + self.translate
    }
}

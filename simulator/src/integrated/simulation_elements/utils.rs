use chrono::Utc;
use rand::{Rng, SeedableRng};
use rand_distr::{Distribution, Exp, Normal};
use serde::Deserialize;
use std::{env, ops::RangeInclusive};

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum FloatExpression {
    Float(f64),
    FloatEnv(String),
    FloatFunc(Transformation<f64>),
}

impl FloatExpression {
    pub(crate) fn value(&self, frame_index: usize) -> f64 {
        match self {
            FloatExpression::Float(v) => *v,
            FloatExpression::FloatEnv(environment_variable) => {
                env::var(environment_variable).unwrap().parse().unwrap()
            }
            FloatExpression::FloatFunc(frame_function) => {
                frame_function.transform(frame_index as f64)
            }
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum IntConstant {
    Int(i32),
    IntEnv(String),
}

impl IntConstant {
    pub(crate) fn value(&self) -> i32 {
        match self {
            IntConstant::Int(v) => *v,
            IntConstant::IntEnv(environment_variable) => {
                env::var(environment_variable).unwrap().parse().unwrap()
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
    pub(crate) fn value(&self, frame_index: usize) -> i32 {
        match self {
            IntExpression::Int(v) => *v,
            IntExpression::IntEnv(environment_variable) => {
                env::var(environment_variable).unwrap().parse().unwrap()
            }
            IntExpression::IntFunc(frame_function) => frame_function.transform(frame_index as i32),
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
    pub(crate) fn sample(&self, frame_index: usize) -> f64 {
        match self {
            Self::Constant { value } => value.value(frame_index),
            Self::Uniform { min, max } => {
                rand::rngs::StdRng::seed_from_u64(Utc::now().timestamp_subsec_nanos() as u64)
                    .gen_range(min.value(frame_index)..max.value(frame_index))
            }
            Self::Normal { mean, sd } => {
                Normal::new(mean.value(frame_index), sd.value(frame_index))
                    .unwrap()
                    .sample(&mut rand::rngs::StdRng::seed_from_u64(
                        Utc::now().timestamp_subsec_nanos() as u64,
                    ))
            }
            Self::Exponential { lifetime } => Exp::new(1.0 / lifetime.value(frame_index))
                .unwrap()
                .sample(&mut rand::rngs::StdRng::seed_from_u64(
                    Utc::now().timestamp_subsec_nanos() as u64,
                )),
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
    pub(crate) fn sample(&self, frame_index: usize) -> i32 {
        match self {
            Self::Constant { value } => value.value(frame_index),
            Self::Uniform { min, max } => {
                rand::rngs::StdRng::seed_from_u64(Utc::now().timestamp_subsec_nanos() as u64)
                    .gen_range(min.value(frame_index)..max.value(frame_index))
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

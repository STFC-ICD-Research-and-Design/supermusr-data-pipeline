use supermusr_common::{Intensity, Time};

use crate::json;


pub(crate) enum Noise {
    Uniform (Intensity),
    SmoothUniform {
        max: Intensity,
        factor: f64,
        prev: f64,
    }
    /*Perlin {
        #[serde(skip, default = "Perlin::new(Utc::now().timestamp_subsec_nanos())")]
        perlin: Perlin,
    }*/
}

impl Noise {
    pub(crate) fn sample(template: &json::NoiseSource) -> Self {
        match template {
            json::NoiseSource::Uniform(max) => Self::Uniform(*max),
            json::NoiseSource::SmoothUniform { max, factor } => Self::SmoothUniform { max: *max, factor: *factor, prev: f64::default() },
        }
    }

    pub(crate) fn noisify(&mut self, value: f64, time: Time) -> f64 {
        match self {
            Self::Uniform(max) => *max as f64*2.0*(rand::random::<f64>() - 1.0),
            //let per = noise::Perlin::new(Utc::now().timestamp_subsec_nanos());
            //per.get([time as f64,0.0])
            Self::SmoothUniform { max, factor, prev } => {
                *prev = *prev * (1.0 - *factor) + 2.0*(rand::random::<f64>() - 1.0)* *factor;
                value + *max as f64 * *prev
            },
        }
    }
}
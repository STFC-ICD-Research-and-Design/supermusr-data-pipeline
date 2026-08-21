use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum MetricOutputGeneric<A, B, C> {
    Scalar(A),
    ScalarWithBand(B),
    BoxPlot(C),
}

pub(crate) type MetricOutput =
    MetricOutputGeneric<Option<f64>, Option<(f64, f64)>, Option<Vec<f64>>>;
pub(crate) type MetricOutputSeries =
    MetricOutputGeneric<Vec<Option<f64>>, Vec<Option<(f64, f64)>>, Vec<Option<Vec<f64>>>>;
/*
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum MetricOutput {
    Scalar(Option<f64>),
    ScalarWithBand(Option<(f64, f64)>),
    BoxPlot(Option<Vec<f64>>),
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum MetricOutputSeries {
    Vector(Vec<Option<f64>>),
    VectorWithBand(Vec<Option<(f64, f64)>>),
    BoxPlot(Vec<Option<Vec<f64>>>),
}*/

impl FromIterator<MetricOutput> for MetricOutputSeries {
    fn from_iter<T: IntoIterator<Item = MetricOutput>>(iter: T) -> Self {
        let iter = iter.into_iter();
        iter.fold(None::<MetricOutputSeries>, |series, value| {
            if let Some(mut series) = series {
                match (&mut series, value) {
                    (MetricOutputSeries::Scalar(vec), MetricOutput::Scalar(value)) => {
                        vec.push(value)
                    }
                    (
                        MetricOutputSeries::ScalarWithBand(vec),
                        MetricOutput::ScalarWithBand(value),
                    ) => vec.push(value),
                    (MetricOutputSeries::BoxPlot(vec), MetricOutput::BoxPlot(value)) => {
                        vec.push(value)
                    }
                    _ => unreachable!(),
                }
                Some(series)
            } else {
                Some(match value {
                    MetricOutput::Scalar(value) => MetricOutputSeries::Scalar(vec![value]),
                    MetricOutput::ScalarWithBand(value) => {
                        MetricOutputSeries::ScalarWithBand(vec![value])
                    }
                    MetricOutput::BoxPlot(value) => MetricOutputSeries::BoxPlot(vec![value]),
                })
            }
        })
        .unwrap_or(MetricOutputSeries::Scalar(Default::default()))
        /*let first = iter.next();
        if let Some(first) = first {
            match first {
                MetricOutput::Scalar(value) => {
                    Self::Vector(iter.fold(vec![value], |mut vec, value| {
                        if let MetricOutput::Scalar(value) = value {
                            vec.push(value);
                        }
                        vec
                    }))
                }
                MetricOutput::ScalarWithBand(value) => {
                    Self::VectorWithBand(iter.fold(vec![value], |mut vec, value| {
                        if let MetricOutput::ScalarWithBand(value) = value {
                            vec.push(value);
                        }
                        vec
                    }))
                }
                MetricOutput::BoxPlot(value) => {
                    Self::BoxPlot(iter.fold(vec![value], |mut vec, value| {
                        if let MetricOutput::BoxPlot(value) = value {
                            vec.push(value);
                        }
                        vec
                    }))
                }
            }
        } else {
            Self::Vector(vec![])
        }*/
    }
}

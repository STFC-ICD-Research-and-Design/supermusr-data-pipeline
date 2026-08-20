mod complete;
mod partial;

use std::ops::Deref;

use crate::analysis::metrics::FittingError;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use thiserror::Error;

pub(crate) use complete::{CompleteMetricResultClass, CompletedMetricResult};
pub(crate) use partial::{PartialMetricResult, PartialMetricResultClass};

/// Type which stores metric results by bucket within a block.
type BucketStore<C> = Vec<C>;

/// Type which stores metric results by bucket block.
type BucketBlockStore<C> = Vec<BucketStore<MetricObject<C>>>;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct MetricObject<C> {
    pub(crate) num_messages: usize,
    pub(crate) object: C,
}

impl<C> Deref for MetricObject<C> {
    type Target = C;

    fn deref(&self) -> &Self::Target {
        &self.object
    }
}

/// A generic type which stores
#[derive(Debug, Serialize, Deserialize)]
#[serde(bound = "C: Serialize + DeserializeOwned")]
pub(crate) struct MetricResultByBucket<C>
where
    C: Clone + Serialize + DeserializeOwned,
{
    /// Metric results storage by bucket block and bucket.
    by_bucket: BucketBlockStore<C>,
}

#[derive(Debug, Error)]
pub(crate) enum MetricResultError {
    #[error("{0}")]
    Fitting(#[from] FittingError),
}

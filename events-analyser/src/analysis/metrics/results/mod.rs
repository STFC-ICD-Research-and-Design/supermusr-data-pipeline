mod complete;
mod partial;

use std::ops::Deref;

use crate::analysis::metrics::{FittingError, MetricResultClass};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use thiserror::Error;

pub(crate) use complete::CompletedMetricResult;
pub(crate) use partial::PartialMetricResult;

/// Type which stores metric results by bucket within a block.
type BucketStore<C> = Vec<C>;

/// Type which stores metric results by bucket block.
type BucketBlockStore<C> = Vec<BucketStore<StoreObject<C>>>;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct StoreObject<C> {
    pub(crate) num_messages: usize,
    pub(crate) object: C,
}

impl<C> Deref for StoreObject<C> {
    type Target = C;

    fn deref(&self) -> &Self::Target {
        &self.object
    }
}

/// A generic type which stores
#[derive(Debug, Serialize, Deserialize)]
#[serde(bound = "C: Serialize + DeserializeOwned")]
pub(crate) struct MetricResultStore<C>
where
    C: MetricResultClass,
{
    /// Metric results storage by bucket block and bucket. FIXME
    by_bucket: BucketBlockStore<C>,
}

#[derive(Debug, Error)]
pub(crate) enum MetricResultError {
    #[error("{0}")]
    Fitting(#[from] FittingError),
    #[error("No Error")]
    NullError,
}

impl From<()> for MetricResultError {
    fn from(_: ()) -> Self {
        MetricResultError::NullError
    }
}

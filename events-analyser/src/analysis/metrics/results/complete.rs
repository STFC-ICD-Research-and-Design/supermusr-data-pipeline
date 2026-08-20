use crate::{
    analysis::metrics::{
        MetricOutput, MetricResultError,
        event_counts::CompletedEventCount,
        false_counts::CompletedFalseCount,
        muon_lifetime::CompletedMuonLifetime,
        results::{MetricResultByBucket, PartialMetricResultClass},
    },
    engine::MetricProperty,
};
use serde::{Deserialize, Serialize, de::DeserializeOwned};

pub(crate) trait CompleteMetricResultClass: Clone + Serialize + DeserializeOwned {
    type Partial: PartialMetricResultClass<Complete = Self>;
    type Error: Into<MetricResultError>;

    fn aggregate(source: &Self::Partial) -> Result<Self, Self::Error>;
    fn get_property(
        &self,
        property: &MetricProperty,
    ) -> Result<MetricOutput<Option<f64>>, Self::Error>;
}

impl<C: CompleteMetricResultClass> MetricResultByBucket<C> {
    pub(super) fn get_property(
        &self,
        block: usize,
        property: &MetricProperty,
    ) -> Result<MetricOutput<Vec<Option<f64>>>, C::Error> {
        let block = self
            .by_bucket
            .get(block)
            .expect("Bucket block should exist, this should never fail.");
        let output = if let Some((first, rest)) = block.split_first() {
            let mut agg: MetricOutput<Vec<Option<f64>>> = first
                .get_property(property)?
                .to_vector(self.by_bucket.len());

            for metric in rest {
                agg.append(&metric.get_property(property)?);
            }
            Some(agg)
        } else {
            None
        }
        .expect("Buckets should exist, this should never fail.");
        Ok(output)
    }
}

#[derive(Serialize, Deserialize)]
pub(crate) enum CompletedMetricResult {
    EventCount(MetricResultByBucket<CompletedEventCount>),
    FalseCount(MetricResultByBucket<CompletedFalseCount>),
    MuonLifetime(MetricResultByBucket<CompletedMuonLifetime>),
}

impl CompletedMetricResult {
    pub(crate) fn get_aggregate_property(
        &self,
        block: usize,
        property: &MetricProperty,
    ) -> Result<MetricOutput<Vec<Option<f64>>>, MetricResultError> {
        Ok(match self {
            Self::EventCount(completed) => completed.get_property(block, property)?,
            Self::FalseCount(completed) => completed.get_property(block, property)?,
            Self::MuonLifetime(completed) => completed.get_property(block, property)?,
        })
    }
}

use crate::{
    analysis::metrics::{
        MetricOutput, MetricResultError,
        event_counts::CompletedEventCount,
        false_counts::CompletedFalseCount,
        muon_lifetime::CompletedMuonLifetime,
        results::{MetricResultByBucket, PartialMetricResultClass},
    },
    engine::{MetricProperty, PropertyOfMetric},
};
use serde::{Deserialize, Serialize, de::DeserializeOwned};

pub(crate) trait CompleteMetricResultClass: Clone + Serialize + DeserializeOwned {
    type Partial: PartialMetricResultClass<Complete = Self>;
    type Error: Into<MetricResultError>;
    type Property: Clone;

    fn aggregate(source: &Self::Partial) -> Result<Self, Self::Error>;
    fn get_property(
        &self,
        property: Self::Property,
    ) -> Result<MetricOutput<Option<f64>>, Self::Error>;
}

impl<C: CompleteMetricResultClass> MetricResultByBucket<C> {
    pub(super) fn get_property(
        &self,
        block: usize,
        property: C::Property,
    ) -> Result<MetricOutput<Vec<Option<f64>>>, C::Error> {
        let block = self
            .by_bucket
            .get(block)
            .expect("Bucket block should exist, this should never fail.");
        let output = if let Some((first, rest)) = block.split_first() {
            let mut agg: MetricOutput<Vec<Option<f64>>> = first
                .get_property(property.clone())?
                .to_vector(self.by_bucket.len());

            for metric in rest {
                agg.append(&metric.get_property(property.clone())?);
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
        property: PropertyOfMetric,
    ) -> Result<MetricOutput<Vec<Option<f64>>>, MetricResultError> {
        Ok(match (self, property) {
            (Self::EventCount(completed), PropertyOfMetric::EventCount(property)) => completed.get_property(block, property)?,
            (Self::FalseCount(completed), PropertyOfMetric::FalseCount(property)) => completed.get_property(block, property)?,
            (Self::MuonLifetime(completed), PropertyOfMetric::MuonLifetime(property)) => completed.get_property(block, property)?,
            _ => unreachable!()
        })
    }
}

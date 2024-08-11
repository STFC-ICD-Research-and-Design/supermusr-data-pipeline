use super::{utils::IntExpression, Interval};
use crate::integrated::simulation_engine::engine::SimulationEngineDigitiser;
use serde::Deserialize;
use supermusr_common::{Channel, DigitizerId};
use tracing::instrument;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum DigitiserConfig {
    #[serde(rename_all = "kebab-case")]
    AutoAggregatedFrame { num_channels: IntExpression },
    #[serde(rename_all = "kebab-case")]
    ManualAggregatedFrame { channels: Vec<Channel> },
    #[serde(rename_all = "kebab-case")]
    AutoDigitisers {
        num_digitisers: IntExpression,
        num_channels_per_digitiser: IntExpression,
    },
    #[serde(rename_all = "kebab-case")]
    ManualDigitisers(Vec<Digitiser>),
}

impl DigitiserConfig {
    pub(crate) fn generate_channels(&self) -> Vec<Channel> {
        match self {
            DigitiserConfig::AutoAggregatedFrame { num_channels } => {
                (0..num_channels.value(0) as Channel).collect()
            }
            DigitiserConfig::ManualAggregatedFrame { channels } => channels.clone(),
            DigitiserConfig::AutoDigitisers {
                num_digitisers,
                num_channels_per_digitiser,
            } => (0..((num_digitisers.value(0) * num_channels_per_digitiser.value(0)) as Channel))
                .collect(),
            DigitiserConfig::ManualDigitisers(digitisers) => digitisers
                .iter()
                .flat_map(|digitiser| digitiser.channels.range_inclusive())
                .collect(),
        }
    }

    #[instrument(skip_all, target = "otel")]
    pub(crate) fn generate_digitisers(&self) -> Vec<SimulationEngineDigitiser> {
        match self {
            DigitiserConfig::AutoAggregatedFrame { .. } => Default::default(),
            DigitiserConfig::ManualAggregatedFrame { .. } => Default::default(),
            DigitiserConfig::AutoDigitisers {
                num_digitisers,
                num_channels_per_digitiser,
            } => (0..num_digitisers.value(0))
                .map(|d| {
                    SimulationEngineDigitiser::new(
                        d as DigitizerId,
                        ((d as usize * num_channels_per_digitiser.value(0) as usize)
                            ..((d as usize + 1) * num_channels_per_digitiser.value(0) as usize))
                            .collect(),
                    )
                })
                .collect(),
            DigitiserConfig::ManualDigitisers(digitisers) => digitisers
                .iter()
                .map(|digitiser| SimulationEngineDigitiser {
                    id: digitiser.id,
                    channel_indices: Vec::<_>::new(), //TODO
                })
                .collect(),
        }
    }

    pub(crate) fn get_num_channels(&self) -> usize {
        match self {
            DigitiserConfig::AutoAggregatedFrame { num_channels } => num_channels.value(0) as usize,
            DigitiserConfig::ManualAggregatedFrame { channels } => channels.len(),
            DigitiserConfig::AutoDigitisers {
                num_digitisers,
                num_channels_per_digitiser,
            } => num_digitisers.value(0) as usize * num_channels_per_digitiser.value(0) as usize,
            DigitiserConfig::ManualDigitisers(_) => 0,
        }
    }

    pub(crate) fn get_num_digitisers(&self) -> usize {
        match self {
            DigitiserConfig::AutoAggregatedFrame { .. } => 0,
            DigitiserConfig::ManualAggregatedFrame { .. } => 0,
            DigitiserConfig::AutoDigitisers { num_digitisers, .. } => {
                num_digitisers.value(0) as usize
            }
            DigitiserConfig::ManualDigitisers(digitiser) => digitiser.len(),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct Digitiser {
    pub(crate) id: DigitizerId,
    pub(crate) channels: Interval<Channel>,
}

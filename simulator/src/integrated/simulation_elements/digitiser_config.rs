use super::Interval;
use crate::integrated::simulation_engine::engine::SimulationEngineDigitiser;
use serde::Deserialize;
use supermusr_common::{Channel, DigitizerId};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum DigitiserConfig {
    #[serde(rename_all = "kebab-case")]
    AutoAggregatedFrame { num_channels: usize },
    #[serde(rename_all = "kebab-case")]
    ManualAggregatedFrame { channels: Vec<Channel> },
    #[serde(rename_all = "kebab-case")]
    AutoDigitisers {
        num_digitisers: usize,
        num_channels_per_digitiser: usize,
    },
    #[serde(rename_all = "kebab-case")]
    ManualDigitisers(Vec<Digitiser>),
}

impl DigitiserConfig {
    pub(crate) fn generate_channels(&self) -> Vec<Channel> {
        match self {
            DigitiserConfig::AutoAggregatedFrame { num_channels } => {
                (0..*num_channels as Channel).collect()
            }
            DigitiserConfig::ManualAggregatedFrame { channels } => channels.clone(),
            DigitiserConfig::AutoDigitisers {
                num_digitisers,
                num_channels_per_digitiser,
            } => (0..((*num_digitisers * *num_channels_per_digitiser) as Channel)).collect(),
            DigitiserConfig::ManualDigitisers(digitisers) => digitisers
                .iter()
                .flat_map(|digitiser| digitiser.channels.range_inclusive())
                .collect(),
        }
    }

    pub(crate) fn generate_digitisers(&self) -> Vec<SimulationEngineDigitiser> {
        match self {
            DigitiserConfig::AutoAggregatedFrame { .. } => Default::default(),
            DigitiserConfig::ManualAggregatedFrame { .. } => Default::default(),
            DigitiserConfig::AutoDigitisers {
                num_digitisers,
                num_channels_per_digitiser,
            } => (0..*num_digitisers)
                .map(|d| SimulationEngineDigitiser {
                    id: d as DigitizerId,
                    channel_indices: ((d * num_channels_per_digitiser)
                        ..((d + 1) * num_channels_per_digitiser))
                        .collect(),
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
            DigitiserConfig::AutoAggregatedFrame { num_channels } => *num_channels,
            DigitiserConfig::ManualAggregatedFrame { channels } => channels.len(),
            DigitiserConfig::AutoDigitisers {
                num_digitisers,
                num_channels_per_digitiser,
            } => *num_digitisers * *num_channels_per_digitiser,
            DigitiserConfig::ManualDigitisers(_) => 0,
        }
    }

    pub(crate) fn get_num_digitisers(&self) -> usize {
        match self {
            DigitiserConfig::AutoAggregatedFrame { .. } => 0,
            DigitiserConfig::ManualAggregatedFrame { .. } => 0,
            DigitiserConfig::AutoDigitisers { num_digitisers, .. } => *num_digitisers,
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

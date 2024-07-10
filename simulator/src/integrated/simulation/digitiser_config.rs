use serde::Deserialize;
use supermusr_common::{Channel, DigitizerId};

use crate::integrated::{simulation_engine::engine::SimulationEngineDigitiser, Interval};

pub(crate) type TraceSourceId = usize;

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
            DigitiserConfig::ManualDigitisers(_) => Default::default(),
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
            DigitiserConfig::ManualDigitisers(_) => Default::default(),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct Digitiser {
    pub(crate) id: DigitizerId,
    pub(crate) channels: Interval<Channel>,
    pub(crate) source: TraceSourceId,
}

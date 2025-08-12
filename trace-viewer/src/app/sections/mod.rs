//! Defines collapsible top-level containers used to present data and allow data entry.
mod broker_poll;
mod display_settings;
mod results;
mod search;

pub(crate) use broker_poll::BrokerSection;
pub(crate) use results::ResultsSection;
pub(crate) use search::SearchSection;

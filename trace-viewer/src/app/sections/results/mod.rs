//! Implements the [Section] which displays the results of a search,
//! and allows the user to select a trace message to be displayed.
mod display_trace;
mod node_refs;
mod results_section;
mod results_settings;
mod select_trace;

pub(crate) use results_section::ResultsSection;
pub(crate) use results_settings::ResultsSettings;

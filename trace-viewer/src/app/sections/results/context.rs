use crate::app::server_functions::CreateAndFetchPlotly;
use leptos::prelude::*;

/// This struct enable a degree of type-checking for the [use_context]/[use_context] functions.
/// Any component making use of the following fields should call `use_context::<ResultsLevelContext>()`
/// and select the desired field.
#[derive(Clone)]
pub(super) struct ResultsLevelContext {
    pub(super) create_and_fetch_plotly: ServerAction<CreateAndFetchPlotly>,
    pub(super) selected_channels_only: RwSignal<bool>,
}

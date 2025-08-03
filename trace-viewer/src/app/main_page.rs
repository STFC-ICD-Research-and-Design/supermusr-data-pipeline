use leptos::prelude::*;

use super::sections::{GetSearchResultsServerAction, SearchResults};
use crate::app::sections::{
    Broker, BrokerSettingsNodeRefs, BrokerSetup, Display, DisplaySettings, DisplaySettingsNodeRefs,
    Search,
};

#[component]
pub(crate) fn Main() -> impl IntoView {
    provide_context(BrokerSettingsNodeRefs::default());
    provide_context(DisplaySettingsNodeRefs::default());

    let get_search_results_action = GetSearchResultsServerAction::new();

    let (selected_trace, set_selected_trace) = signal::<Option<Vec<u16>>>(None);

    view! {
        <div class = "main">
            <div class = "left-column">
                <SearchResults get_search_results_action set_selected_trace />
            </div>
            <div class = "middle-column">
                <BrokerSetup />
                <Broker />
                <DisplaySettings />
                <Search get_search_results_action />
                <Display selected_trace />
            </div>
        </div>
    }
}

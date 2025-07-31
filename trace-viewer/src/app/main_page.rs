
use leptos::prelude::*;

use crate::{app::sections::{Broker, BrokerSettingsNodeRefs, BrokerSetup, Display, DisplaySettings, DisplaySettingsNodeRefs, Search }};
use super::sections::{SearchResults, SearchBrokerServerAction};

#[component]
pub(crate) fn Main() -> impl IntoView {
    provide_context(BrokerSettingsNodeRefs::default());
    provide_context(DisplaySettingsNodeRefs::default());

    let search_broker_action = SearchBrokerServerAction::new();

    let (selected_trace, set_selected_trace) = signal::<Option<Vec<u16>>>(None);

    view! {
        <div class = "main">
            <div class = "left-column">
                <SearchResults search_broker_action set_selected_trace />
            </div>
            <div class = "middle-column">
                <BrokerSetup />
                <Broker />
                <DisplaySettings />
                <Search search_broker_action />
                <Display selected_trace />
            </div>
        </div>
    }
}

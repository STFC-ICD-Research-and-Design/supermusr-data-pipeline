
use leptos::prelude::*;

use crate::app::sections::{Broker, BrokerSettingsNodeRefs, BrokerSetup, DisplaySettings, DisplaySettingsNodeRefs, Search };
use super::sections::{SearchResults, SearchBrokerServerAction};

#[component]
pub(crate) fn Main() -> impl IntoView {
    provide_context(BrokerSettingsNodeRefs::default());
    provide_context(DisplaySettingsNodeRefs::default());

    let search_broker_action = SearchBrokerServerAction::new();

    view! {
        <div class = "middle-column">
            <BrokerSetup />
            <Broker />
            <DisplaySettings />
            <Search search_broker_action />
            <SearchResults search_broker_action />
        </div>
    }
}

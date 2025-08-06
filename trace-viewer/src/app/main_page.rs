use leptos::prelude::*;

use super::sections::Results;
use crate::app::{sections::{
    Broker, BrokerSettingsNodeRefs, BrokerSetup, DisplaySettings, DisplaySettingsNodeRefs,
    Search,
}, server_functions::{CreateNewSearch, GetSearchResults}};

#[component]
pub(crate) fn Main() -> impl IntoView {
    provide_context(BrokerSettingsNodeRefs::default());
    provide_context(DisplaySettingsNodeRefs::default());

    provide_context(ServerAction::<GetSearchResults>::new());
    provide_context(ServerAction::<CreateNewSearch>::new());

    view! {
        <div class = "main">
            <BrokerSetup />
            <Broker />
            <DisplaySettings />
            <Search />
            <Results />
        </div>
    }
}

use leptos::prelude::*;

use super::sections::Results;
use crate::{app::{
    sections::{Broker, DisplaySettings, DisplaySettingsNodeRefs, Search},
    server_functions::{CreateAndFetchPlotlyOfSelectedTrace, CreateNewSearch, FetchSearchSummaries},
}, structs::SelectedTraceIndex};

#[component]
pub(crate) fn Main() -> impl IntoView {
    //provide_context(BrokerSettingsNodeRefs::default());

    provide_context(DisplaySettingsNodeRefs::default());

    provide_context(ServerAction::<FetchSearchSummaries>::new());
    provide_context(ServerAction::<CreateNewSearch>::new());

    // Currently Selected Digitiser Trace Message
    provide_context(ServerAction::<CreateAndFetchPlotlyOfSelectedTrace>::new());

    view! {
        <div class = "main">
            //<BrokerSetup />
            <Broker />
            <DisplaySettings />
            <Search />
            <Results />
        </div>
    }
}

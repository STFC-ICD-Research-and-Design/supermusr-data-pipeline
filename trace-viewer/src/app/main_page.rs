use leptos::{logging, prelude::*};

use super::sections::ResultsSection;
use crate::{
    app::{
        Uuid,
        sections::{BrokerSection, DisplaySettingsNodeRefs, SearchSection},
        server_functions::{
            AwaitSearch, CreateAndFetchPlotlyOfSelectedTrace, CreateNewSearch,
            FetchSearchSummaries,
        },
    }
};

#[component]
pub(crate) fn Main() -> impl IntoView {
    provide_context(DisplaySettingsNodeRefs::default());

    let create_new_search = ServerAction::<CreateNewSearch>::new();
    let await_search = ServerAction::<AwaitSearch>::new();
    let fetch_search_summaries = ServerAction::<FetchSearchSummaries>::new();

    //let (selected_trace_index, set_selected_trace_index) = signal::<Option<SelectedTraceIndex>>(None);
    let (uuid, set_uuid) = signal::<Uuid>(None);

    provide_context(create_new_search);
    provide_context(await_search);
    provide_context(fetch_search_summaries);
    provide_context(uuid);

    Effect::new(move || {
        if create_new_search.pending().get() {
            await_search.clear();
            fetch_search_summaries.clear();
        }
    });

    Effect::new(move || {
        if let Some(uuid) = create_new_search.value().get() {
            match uuid {
                Ok(uuid) => set_uuid.set(Some(uuid)),
                Err(e) => {
                    logging::warn!("{e}");
                    set_uuid.set(None)
                }
            }
        }
    });

    Effect::new(move || {
        if let Some(uuid) = uuid.get() {
            await_search.dispatch(AwaitSearch { uuid });
        }
    });

    Effect::new(move || {
        if let Some(uuid) = uuid.get() {
            if let Some(result) = await_search.value().get() {
                match result {
                    Ok(_) => {
                        fetch_search_summaries.dispatch(FetchSearchSummaries { uuid });
                    }
                    Err(e) => logging::warn!("{e}"),
                }
            }
        }
    });

    // Currently Selected Digitiser Trace Message
    provide_context(ServerAction::<CreateAndFetchPlotlyOfSelectedTrace>::new());

    view! {
        <div class = "main">
            <BrokerSection />
            //<DisplaySettings />
            <SearchSection />
            <ResultsSection />
        </div>
    }
}

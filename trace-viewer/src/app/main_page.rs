use leptos::prelude::*;
use tracing::warn;

use super::sections::ResultsSection;
use crate::app::{
    sections::{Broker, DisplaySettings, DisplaySettingsNodeRefs, SearchSection},
    server_functions::{
        AwaitSearch, CreateAndFetchPlotlyOfSelectedTrace, CreateNewSearch, FetchSearchSummaries,
    },
};

fn with_uuid<F>(f: F)
where
    F: Fn(String),
{
    let create_new_search = use_context::<ServerAction<CreateNewSearch>>().expect("");
    if let Some(uuid) = create_new_search.value().get() {
        match uuid {
            Ok(uuid) => f(uuid),
            Err(e) => warn!("{e}"),
        }
    }
}

#[component]
pub(crate) fn Main() -> impl IntoView {
    provide_context(DisplaySettingsNodeRefs::default());

    let create_new_search = ServerAction::<CreateNewSearch>::new();
    let await_search = ServerAction::<AwaitSearch>::new();
    let fetch_search_summaries = ServerAction::<FetchSearchSummaries>::new();

    provide_context(create_new_search);
    provide_context(await_search);
    provide_context(fetch_search_summaries);

    Effect::new(move || {
        with_uuid(move |uuid| {
            await_search.dispatch(AwaitSearch { uuid });
        })
    });

    Effect::new(move || {
        if create_new_search.pending().get() {
            await_search.clear();
            fetch_search_summaries.clear();
        }
    });

    Effect::new(move || {
        with_uuid(move |uuid| {
            if let Some(result) = await_search.value().get() {
                match result {
                    Ok(_) => {
                        fetch_search_summaries.dispatch(FetchSearchSummaries { uuid });
                    }
                    Err(e) => warn!("{e}"),
                }
            }
        })
    });
    Effect::new(move || {
        if let Some(uuid) = create_new_search.value().get() {
            match uuid {
                Ok(uuid) => {
                    if let Some(result) = await_search.value().get() {
                        match result {
                            Ok(_) => {
                                fetch_search_summaries.dispatch(FetchSearchSummaries { uuid });
                            }
                            Err(e) => warn!("{e}"),
                        }
                    }
                }
                Err(e) => warn!("{e}"),
            }
        }
    });

    // Currently Selected Digitiser Trace Message
    provide_context(ServerAction::<CreateAndFetchPlotlyOfSelectedTrace>::new());

    view! {
        <div class = "main">
            <Broker />
            //<DisplaySettings />
            <SearchSection />
            <ResultsSection />
        </div>
    }
}

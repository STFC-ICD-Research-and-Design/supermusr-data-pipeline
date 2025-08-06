mod node_refs;
mod search_control;
mod search_settings;
mod statusbar;

use leptos::{IntoView, component, prelude::*, view};

use search_settings::SearchSettings;
use tracing::warn;

use crate::{
    app::{
        components::Section, sections::search::{node_refs::SearchBrokerNodeRefs, search_control::SearchControl}, server_functions::{CreateNewSearch, FetchSearchSummaries}
    },
    structs::{SearchTarget, SearchTargetBy, SearchTargetMode, SelectedTraceIndex},
};

#[component]
pub(crate) fn Search() -> impl IntoView {
    let search_broker_node_refs = SearchBrokerNodeRefs::default();
    provide_context(search_broker_node_refs);

    /*let (_, set_selected_message) = use_context::<(
        ReadSignal<Option<SelectedTraceIndex>>,
        WriteSignal<Option<SelectedTraceIndex>>,
    )>()
    .expect("");*/

    let fetch_search_summaries = use_context::<ServerAction<FetchSearchSummaries>>().expect("");
    let create_new_search = use_context::<ServerAction<CreateNewSearch>>().expect("");

    let on_submit = move || {
        let time = search_broker_node_refs.get_time();
        let date = search_broker_node_refs.get_date();
        let timestamp = date.and_time(time).and_utc();

        let channels = search_broker_node_refs.get_channels();
        let number = search_broker_node_refs.get_number();

        let target = SearchTarget {
            mode: SearchTargetMode::Timestamp { timestamp },
            by: SearchTargetBy::ByChannels { channels },
            number,
        };

        create_new_search.dispatch(CreateNewSearch { target });
    };

    Effect::new(move || {
        if let Some(uuid) = create_new_search.value().get() {
            match uuid {
                Ok(uuid) => {
                    fetch_search_summaries.dispatch(FetchSearchSummaries { uuid });
                }
                Err(e) => warn!("{e}"),
            }
        }
    });

    //set_selected_message.set(None);
    view! {
        <form on:submit = move |e|{ e.prevent_default(); on_submit() }>
            <Section name = "Search" classes = vec!["search-setup"] closable = false>
                <SearchSettings />
                <SearchControl />
            </Section>
        </form>
    }
}

use leptos::{IntoView, component, prelude::*, view};

use crate::{
    app::{
        components::Section,
        main_content::MainLevelContext,
        sections::search::{
            node_refs::SearchBrokerNodeRefs, search_control::SearchControl,
            search_settings::SearchSettings,
        },
        server_functions::CreateNewSearch,
    },
    structs::{SearchTarget, SearchTargetBy, SearchTargetMode},
};

#[derive(Clone)]
pub(crate) struct SearchLevelContext {
    pub(crate) search_broker_node_refs: SearchBrokerNodeRefs,
}

#[component]
pub(crate) fn SearchSection() -> impl IntoView {
    let main_context = use_context::<MainLevelContext>()
        .expect("MainLevelContext should be provided, this should never fail.");
    let create_new_search = main_context.create_new_search;

    let search_broker_node_refs = SearchBrokerNodeRefs::default();
    provide_context(SearchLevelContext {
        search_broker_node_refs,
    });

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

    view! {
        <form on:submit = move |e|{ e.prevent_default(); on_submit() }>
            <Section text = "Search" id = "search-setup">
                <SearchSettings />
                <SearchControl />
            </Section>
        </form>
    }
}

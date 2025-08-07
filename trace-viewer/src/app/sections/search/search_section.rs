use leptos::{IntoView, component, prelude::*, view};

use crate::{
    app::{
        components::Section,
        sections::search::{node_refs::SearchBrokerNodeRefs, search_control::SearchControl, search_settings::SearchSettings},
        server_functions::CreateNewSearch,
    },
    structs::{SearchTarget, SearchTargetBy, SearchTargetMode},
};

#[component]
pub(crate) fn SearchSection() -> impl IntoView {
    let search_broker_node_refs = SearchBrokerNodeRefs::default();
    provide_context(search_broker_node_refs);

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

        create_new_search.clear();
        create_new_search.dispatch(CreateNewSearch { target });
    };

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

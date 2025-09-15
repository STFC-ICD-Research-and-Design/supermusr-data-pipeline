use crate::{
    app::{
        TopLevelContext,
        components::Section,
        main_content::MainLevelContext,
        sections::search::{
            SearchLevelContext,
            search_control::SearchControl,
            search_settings::{SearchBy, SearchMode, SearchSettings},
        },
        server_functions::CreateNewSearch,
    },
    structs::{SearchTarget, SearchTargetBy, SearchTargetMode},
};
use leptos::{IntoView, component, prelude::*, view};

#[component]
pub(crate) fn SearchSection() -> impl IntoView {
    let default_data = use_context::<TopLevelContext>()
        .expect("TopLevelContext should be provided, this should never fail.")
        .client_side_data
        .default_data;

    let main_context = use_context::<MainLevelContext>()
        .expect("MainLevelContext should be provided, this should never fail.");
    let create_new_search = main_context.create_new_search;

    let search_level_context = SearchLevelContext::new(&default_data);
    provide_context(search_level_context.clone());

    let on_submit = move || {
        let target = SearchTarget {
            mode: match search_level_context.search_mode.get() {
                SearchMode::Timestamp => SearchTargetMode::Timestamp {
                    timestamp: search_level_context
                        .date
                        .get()
                        .and_time(search_level_context.time.get())
                        .and_utc(),
                },
            },
            by: match search_level_context.search_by.get() {
                SearchBy::All => SearchTargetBy::All,
                SearchBy::ByChannels => SearchTargetBy::ByChannels {
                    channels: search_level_context.channels.get(),
                },
                SearchBy::ByDigitiserIds => SearchTargetBy::ByDigitiserIds {
                    digitiser_ids: search_level_context.digitiser_ids.get(),
                },
            },
            number: search_level_context.number.get(),
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

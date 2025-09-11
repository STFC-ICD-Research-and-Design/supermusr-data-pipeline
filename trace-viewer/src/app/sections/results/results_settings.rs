use crate::{app::sections::results::context::ResultsLevelContext, structs::{SearchTarget, SearchTargetBy}};
use leptos::{component, either::Either, prelude::*, view, IntoView};

#[component]
pub(crate) fn ResultsSettingsPanel(target: SearchTarget) -> impl IntoView {
    view!{
        <div class = "search-results-settings">
            <ShowSelectedChannelsOnly by = target.by />
        </div>
    }
}

#[component]
pub(crate) fn ShowSelectedChannelsOnly(by: SearchTargetBy) -> impl IntoView {
    let result_level_context = use_context::<ResultsLevelContext>()
        .expect("results_settings_node_refs should be provided, this should never fail.");

    match by {
        SearchTargetBy::All => Either::Left(()),
        _ => Either::Right(view! {
            <label for = "selected-channels-only">
                "Show selected channels only:"
                <input name = "selected-channels-only" id = "selected-channels-only" type = "checkbox"
                    bind:value = result_level_context.display_all_channels
                />
            </label>
        })
    }
}
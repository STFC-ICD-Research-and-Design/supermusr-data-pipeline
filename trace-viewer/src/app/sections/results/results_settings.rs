use crate::app::sections::results::context::ResultsLevelContext;
use leptos::{IntoView, component, prelude::*, view};
use strum::{Display, EnumIter, EnumString, IntoEnumIterator};

#[component]
pub(crate) fn ResultsSettings() -> impl IntoView {
    let result_level_context = use_context::<ResultsLevelContext>()
        .expect("results_settings_node_refs should be provided, this should never fail.");

    view! {
        <label for = "only-selected-channels">
            "Show only selected channels:"
            <input name = "only-selected-channels" id = "only_selected_channels" type = "checkbox"
                bind:value = result_level_context.display_all_channels
            />
        </label>

        <label for = "display-mode">
            "Display Mode:"
            <select name = "display-mode" id = "display-mode"
                on:change = move |ev|
                    result_level_context.display_mode.set(
                        event_target_value(&ev)
                            .parse()
                            .expect("SearchMode value should parse, this should never fail.")
                        ) >
                <For each = DisplayType::iter key = ToOwned::to_owned let(mode)>
                    <option selected={result_level_context.display_mode.get() == mode} value = {mode.to_string()}> {mode.to_string()} </option>
                </For>
            </select>
        </label>
    }
}

#[derive(Default, Clone, EnumString, Display, EnumIter, PartialEq, Eq, Hash, Copy)]
pub(crate) enum DisplayType {
    #[default]
    #[strum(to_string = "Single Chart")]
    Single,
    #[strum(to_string = "Multiple Overlayed Chart")]
    Overlayed,
    #[strum(to_string = "Multiple Stacked Chart")]
    Stacked,
}

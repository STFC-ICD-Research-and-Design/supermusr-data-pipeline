use leptos::{component, view, IntoView, prelude::*};

use crate::app::components::{Display, Panel, Section};

#[component]
pub(crate) fn Results() -> impl IntoView {
    view! {
        <Section name = "Results">
        <Panel>
            "No Results"
        </Panel>
        <Display />
        </Section>
    }
}
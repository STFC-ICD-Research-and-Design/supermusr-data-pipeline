use leptos::{component, view, IntoView};

use crate::{app::components::{Panel, Section}};

#[component]
pub fn Status() -> impl IntoView {
    view! {
        <Section name = "Status">
            <Panel>
                " "
            </Panel>
        </Section>
    }
}

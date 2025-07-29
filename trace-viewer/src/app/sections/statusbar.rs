use leptos::{IntoView, component, view};

use crate::app::components::{Panel, Section};

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

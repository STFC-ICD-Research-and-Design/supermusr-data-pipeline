use leptos::{component, prelude::*, view, IntoView};

use crate::{app::components::{Panel, Section}};

#[component]
pub fn Status() -> impl IntoView {
    view! {
        <Section name = "Status">
            <Panel name = "Status">
                " "
            </Panel>
            <Panel name = "Progress">
                " "
            </Panel>
        </Section>
    }
}

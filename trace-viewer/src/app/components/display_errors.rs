use leptos::{IntoView, component, prelude::*, view};

use crate::app::components::Section;

/// This component displays any errors accrued, usually accrued from a call to a server function.
#[component]
pub(crate) fn DisplayErrors(errors: ArcRwSignal<Errors>) -> impl IntoView {
    view! {
        <Section text = "Errors" id = "error">
            {move ||errors.get().into_iter().map(|(e_id, error)| view!{
                <div>{format!("{e_id}: {error}")}</div>
            }).collect::<Vec<_>>()}
        </Section>
    }
}

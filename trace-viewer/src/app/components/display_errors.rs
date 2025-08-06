use leptos::{IntoView, component, either::Either, prelude::*, view};

use crate::app::components::Section;

#[component]
pub(crate) fn DisplayErrors(errors: ArcRwSignal<Errors>) -> impl IntoView {
    view!{
        <Section name = "Errors">
            {errors.get().into_iter().map(|(e_id, error)| view!{
                <div>{format!("{e_id}: {}", error.to_string())}</div>
            }).collect::<Vec<_>>()}
        </Section>
    }
}
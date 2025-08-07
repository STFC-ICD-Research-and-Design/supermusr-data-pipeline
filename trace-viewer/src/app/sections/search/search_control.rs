use leptos::either::Either;
use leptos::{IntoView, component, prelude::*, view};

use crate::app::{
    sections::search::statusbar::Statusbar,
    server_functions::{
        AwaitSearch, CancelSearch
    },
    components::SubmitBox,
    Uuid
};

#[component]
pub(crate) fn SearchControl() -> impl IntoView {
    let await_search = use_context::<ServerAction<AwaitSearch>>().expect("");
    let uuid = use_context::<ReadSignal<Uuid>>().expect("");

    let cancel_search_server_action = ServerAction::<CancelSearch>::new();

    move || {
        if !await_search.pending().get() {
            Either::Left(view! {
                <SubmitBox label = "Search" classes = vec!["search-button"] />
            })
        } else {
            Either::Right(view! {
                <Statusbar />
                {move ||
                    {uuid.get().map(|uuid| view! {
                        <input type = "button" class = "cancel-button panel-item across-two-cols" value = "Cancel"
                            on:click = move |_| { let uuid = uuid.clone(); cancel_search_server_action.dispatch(CancelSearch { uuid: uuid.clone() }); }
                        />
                    })}
                }
            })
        }
    }
}

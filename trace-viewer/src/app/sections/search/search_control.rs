use leptos::either::Either;
use leptos::{IntoView, component, prelude::*, view};

use crate::app::main_content::MainLevelContext;
use crate::app::{
    sections::search::statusbar::Statusbar,
    server_functions::CancelSearch,
};

#[component]
pub(crate) fn SearchControl() -> impl IntoView {
    let main_context = use_context::<MainLevelContext>().expect("");
    let await_search = main_context.await_search;
    let uuid = main_context.uuid;

    let cancel_search_server_action = ServerAction::<CancelSearch>::new();

    move || {
        if !await_search.pending().get() {
            Either::Left(view! {
                <input type = "submit" class = "search-button" value = "Search" />
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

use leptos::either::Either;
use leptos::{IntoView, component, prelude::*, view};

use crate::app::sections::search::statusbar::Statusbar;
use crate::app::server_functions::{
    AwaitSearch, CancelSearch, CreateNewSearch, FetchSearchSummaries,
};

use crate::app::components::{DisplayErrors, SubmitBox};

#[component]
pub(crate) fn SearchControl() -> impl IntoView {
    let fetch_search_summaries = use_context::<ServerAction<FetchSearchSummaries>>().expect("");
    let await_search = use_context::<ServerAction<AwaitSearch>>().expect("");
    let create_new_search = use_context::<ServerAction<CreateNewSearch>>().expect("");

    let cancel_search_server_action = ServerAction::<CancelSearch>::new();

    move || {
        if !create_new_search.pending().get()
            && !await_search.pending().get()
            && !fetch_search_summaries.pending().get()
        {
            Either::Left(view! {
                <SubmitBox label = "Search" classes = vec!["search-button"] />
            })
        } else {
            Either::Right(view! {
                <Statusbar />
                {move ||
                    create_new_search.value().get().map(move |uuid| view! {
                        <ErrorBoundary fallback = |errors| view! { <DisplayErrors errors /> }>
                            {uuid.map(|uuid| view! {
                                <input type = "button" class = "cancel-button panel-item across-two-cols" value = "Cancel"
                                    on:click = move |_| { let uuid = uuid.clone(); cancel_search_server_action.dispatch(CancelSearch { uuid: uuid.clone() }); }
                                />
                            })}
                        </ErrorBoundary>
                    })
                }
            })
        }
    }
}

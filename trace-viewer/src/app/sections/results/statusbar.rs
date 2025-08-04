use leptos::{IntoView, component, prelude::*, view};
use strum::{Display, EnumString};
//use leptos_sse::create_sse_signal;

use crate::{
    app::{components::Panel, server_functions::{CreateNewSearch, GetStatus}},
    structs::SearchStatus,
};

#[derive(Default, Clone, PartialEq, Eq, Hash, EnumString, Display)]
pub(crate) enum StatusMessage {
    #[default]
    #[strum(to_string = "")]
    Waiting,
    #[strum(to_string = "Searching for Traces.")]
    TraceSearchInProgress,
    #[strum(to_string = "Search for Traces Finished.")]
    TraceSearchFinished,
    #[strum(to_string = "Searching for Event Lists.")]
    EventListSearchInProgress,
    #[strum(to_string = "Search for Event Lists Finished.")]
    EventListSearchFinished,
    #[strum(to_string = "Search Complete. Found {num} traces, in {secs},{ms} ms.")]
    SearchFinished { num: usize, secs: i64, ms: i32 },
    #[strum(to_string = "{0}")]
    Text(String),
}

impl SearchStatus {
    pub(crate) fn progress(&self) -> Option<f64> {
        match self {
            SearchStatus::TraceSearchInProgress(progress) => Some(*progress),
            SearchStatus::EventListSearchInProgress(progress) => Some(*progress),
            _ => None,
        }
    }

    pub(crate) fn message(&self) -> StatusMessage {
        match self {
            SearchStatus::Off => StatusMessage::Waiting,
            SearchStatus::TraceSearchInProgress(_) => StatusMessage::TraceSearchInProgress,
            SearchStatus::TraceSearchFinished => StatusMessage::TraceSearchFinished,
            SearchStatus::EventListSearchInProgress(_) => StatusMessage::EventListSearchInProgress,
            SearchStatus::EventListSearchFinished => StatusMessage::EventListSearchFinished,
            SearchStatus::Successful { num, time } => StatusMessage::SearchFinished {
                num: *num,
                secs: time.num_seconds(),
                ms: time.subsec_millis(),
            },
        }
    }
}

#[component]
pub fn Statusbar() -> impl IntoView {
    let create_new_search_action = use_context::<ServerAction<CreateNewSearch>>().expect("");
    let get_status_server_action = ServerAction::<GetStatus>::new();
    
    let (statuses, set_statuses) = signal::<Vec<StatusMessage>>(Vec::new());
    
    create_new_search_action.value().get().and_then(Result::ok).map(|uuid| {
        Effect::new(move || {
            get_status_server_action.value().get().map(|status| match status {
                Ok(status) => {
                    set_statuses.write().push(status.clone().message());
                    get_status_server_action.dispatch(GetStatus { old_status: status, uuid: uuid.clone() });
                },
                Err(e) => {},
            });
        });
        view!{
            <For
                each = move || statuses.get()
                key = |key|key.clone()
                let(status)
            >
                <div>{status.to_string()}</div>
            </For>
        }
    })
}

#[component]
fn ProgressBar(progress: Option<f64>) -> impl IntoView {
    progress.map(|progress| {
        let style = format!("'width: {};'", 100.0 * progress);
        view! {
            <Panel>
                <div class = "progress-bar">
                    <div class = "progress-made" style = {style}>
                    </div>
                </div>
            </Panel>
        }
    })
}

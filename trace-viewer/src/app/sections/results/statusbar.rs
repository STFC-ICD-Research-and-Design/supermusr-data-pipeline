use leptos::{IntoView, component, prelude::*, view};
use strum::{Display, EnumString};
use tracing::warn;
//use leptos_sse::create_sse_signal;

use crate::{
    app::{components::{DisplayErrors, Panel, SubmitBox}, server_functions::{get_status, CancelSearch, CreateNewSearch, GetStatus}},
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
    
    create_new_search_action.value()
        .get()
        .map(|uuid|
            view!{
                <ErrorBoundary fallback = |errors|view!{ <DisplayErrors errors /> }>
                    {uuid.map(|uuid| view!{ <StatusbarOfUuid uuid /> })}
                </ErrorBoundary>
            }
        )
}

#[component]
pub fn StatusbarOfUuid(uuid: String) -> impl IntoView {

    let status = {
        let uuid = uuid.clone();
        Resource::new(||(), move |_|get_status(uuid.clone()))
    };

    let cancel_search_server_action = ServerAction::<CancelSearch>::new();

    Effect::new(move |prev: Option<()>| {
        if prev.is_some() {
            leptos::tracing::info!("Hiya");
            match status.get() {
                Some(Err(_)) => {},
                Some(Ok(SearchStatus::Successful { .. })) => {},
                _ => {
            leptos::tracing::info!("Hiyo");
            status.refetch()
        },
            }
        }
    });

    view!{
        <Panel classes = vec!["status-bar"]>
            <Transition fallback = || view!{<div>"Loading..."</div> }>
                <ErrorBoundary fallback = |errors|view!{ <DisplayErrors errors /> }>
                    {move ||status.get().map(|status|status.map(|status|
                        view!{
                            <DisplayStatusbar message = status.message()/>
                            <DisplayProgressBar progress = status.progress()/>
                        }
                    ))}
                </ErrorBoundary>
            </Transition>
        </Panel>
        <Panel>
            <form on:submit = move |_|{
                let uuid = uuid.clone();
                cancel_search_server_action.dispatch(CancelSearch { uuid: uuid.clone() });
            }>
                <SubmitBox label = "Cancel" classes = vec!["cancel-button"]/>
            </form>
        </Panel>
    }
}

#[component]
pub fn DisplayStatusbar(message: StatusMessage) -> impl IntoView {
    view!{
        <Panel>
            <div class = "status-message">
                {message.to_string()}
            </div>
        </Panel>
    }
}

#[component]
fn DisplayProgressBar(progress: Option<f64>) -> impl IntoView {
    progress.map(|progress| {
        let style = format!("'width: {}%;'", 100.0 * progress);
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

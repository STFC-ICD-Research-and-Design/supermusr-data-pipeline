use leptos::{component, logging, prelude::*, view, IntoView};
use strum::{Display, EnumString};

use crate::{
    app::{
        components::{DisplayErrors, Panel, SubmitBox},
        server_functions::{fetch_status, CancelSearch, CreateNewSearch}, Uuid,
    },
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
    let uuid = use_context::<ReadSignal<Uuid>>().expect("");

    uuid.get().map(|uuid|
        view! {
            <StatusbarOfUuid uuid />
        }
    )
}

#[component]
pub fn StatusbarOfUuid(uuid: String) -> impl IntoView {
    let status = {
        let uuid = uuid.clone();
        Resource::new(|| (), move |_| fetch_status(uuid.clone()))
    };

    Effect::new(move |prev: Option<()>| {
        leptos::logging::log!("Attempting to run statusbar effect");
        if prev.is_some() {
            leptos::logging::log!("Hiya");
            match status.get() {
                Some(Err(e)) => {
                    logging::warn!("{e}");
                }
                Some(Ok(SearchStatus::Successful { .. })) => {}
                _ => {
                    leptos::logging::log!("Hiyo");
                    status.refetch()
                }
            }
        }
    });

    view! {
        <Panel classes = vec!["status-bar", "across-two-cols"]>
            {move || view! {
            <Transition fallback = || view!{<div>"Loading..."</div> }>
                {move || view! {
                    <ErrorBoundary fallback = |errors|view!{ <DisplayErrors errors /> }>
                        {move ||status.get().map(|status|status.map(|status|
                            view!{
                                <DisplayStatusbar message = status.message()/>
                                <DisplayProgressBar progress = status.progress()/>
                            }
                        ))}
                    </ErrorBoundary>
                }}
            </Transition>
            }}
        </Panel>
    }
}

#[component]
pub fn DisplayStatusbar(message: StatusMessage) -> impl IntoView {
    view! {
        <Panel classes = vec!["across-two-cols"]>
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

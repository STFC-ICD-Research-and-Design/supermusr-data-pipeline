use leptos::{component, prelude::*, view, IntoView};
use strum::{Display, EnumString};
use leptos_sse::create_sse_signal;

use crate::{app::{components::{Panel, Section}, sections::SearchBrokerServerAction}, structs::SearchStatus};

#[derive(Default, EnumString, Display)]
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
    #[strum(
        to_string = "Search Complete. Found {num} traces, in {secs},{ms} ms."
    )]
    SearchFinished { num: usize, secs: i64, ms: i32 },
    #[strum(to_string = "{0}")]
    Text(String),
}

impl SearchStatus {
    pub(crate) fn progress(&self) -> Option<f64> {
        match self {
            SearchStatus::TraceSearchInProgress(progress) => Some(*progress),
            SearchStatus::EventListSearchInProgress(progress) => Some(*progress),
            _ => None
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
            }
        }
    }
}

#[server]
pub async fn get_status(old_status: SearchStatus) -> Result<SearchStatus, ServerFnError> {
    use std::sync::{Arc, Mutex};
    use std::time::Duration;
    use tokio::time::sleep;

    let status = use_context::<Arc<Mutex<SearchStatus>>>().expect("");
    loop {
        let new_status = status.lock().expect("").clone();
        if old_status != new_status {
            return Ok(new_status)
        }
        sleep(Duration::from_millis(50)).await;
    }
}

#[component]
pub fn Statusbar(search_broker_action: SearchBrokerServerAction) -> impl IntoView {
    use leptos_reactive::signal_prelude::SignalGet;
    //let (current_status, set_current_status) = signal(SearchStatus::Off);
    //let get_status_action = ServerAction::<GetStatus>::new();

    let status = create_sse_signal::<SearchStatus>("search_status");
    
    view!{
        <Show when = move ||search_broker_action.pending().get()>
            {move || {
                let current_status = status.get();
                let status_message = current_status.message();
                let progress = current_status.progress();
                view!{
                    <Section name = "Search Status">
                        <Panel>
                            <div class = "status-message">
                                {status_message.to_string()}
                            </div>
                        </Panel>
                        <ProgressBar progress/>
                    </Section>
                }
            }
        }
        </Show>
    }
}

#[component]
fn ProgressBar(progress: Option<f64>) -> impl IntoView {
    progress.map(|progress|{
        let style = format!("'width: {};'", 100.0*progress);
        view!{
            <Panel>
                <div class = "progress-bar">
                    <div class = "progress-made" style = {style}>
                    </div>
                </div>
            </Panel>
        }
    })
}
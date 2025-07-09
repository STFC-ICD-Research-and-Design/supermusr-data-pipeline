use leptos::{component, view, IntoView};
use rdkafka::consumer::StreamConsumer;
use tokio::{task::JoinHandle, select};
use crate::Cli;

pub(crate) async fn run_server(consumer: StreamConsumer, args: Cli) -> JoinHandle<()> {
    /*loop {
        select!{
            _ => {}
        }
    }*/
}

#[component]
fn App() -> impl IntoView {
    view! {

    }
}
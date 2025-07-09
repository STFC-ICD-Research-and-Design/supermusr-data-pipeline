use leptos::{component, view, IntoView, prelude::{ElementChild, mount_to_body}};
use rdkafka::consumer::StreamConsumer;
//use tokio::{task::JoinHandle, select};
use crate::Cli;

pub(crate) async fn run_server(consumer: StreamConsumer, args: Cli) -> anyhow::Result<()> {
    /*loop {
        select!{
            _ => {}
        }
    }*/
    mount_to_body(|| view! { <p>"Hello, world!"</p> });
    Ok(())
}

#[component]
fn App() -> impl IntoView {
    view! {

    }
}
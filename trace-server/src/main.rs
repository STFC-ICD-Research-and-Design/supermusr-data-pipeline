use std::net::SocketAddr;
use miette::{self, IntoDiagnostic};

use thiserror as _;
use rdkafka as _;
use strum as _;
use clap::Parser;
use supermusr_streaming_types as _;
use tracing as _;
use supermusr_common::CommonKafkaOpts;
use leptos::prelude::*;
use trace_server::App;
use leptos_meta as _;
use chrono as _;
use serde as _;
use serde_json as _;
use tracing as _;
//use tokio as _;

use trace_server::{cli_structs::{Select, Topics}, finder::SearchEngine};

#[derive(Parser)]
#[clap(author, version, about)]
struct Cli {
    #[clap(flatten)]
    common_kafka_options: CommonKafkaOpts,

    /// Kafka consumer group.
    #[clap(long)]
    consumer_group: String,

    #[clap(flatten)]
    topics: Topics,

    /// If set, then OpenTelemetry data is sent to the URL specified, otherwise the standard tracing subscriber is used.
    #[clap(long)]
    otel_endpoint: Option<String>,

    /// All OpenTelemetry spans are emitted with this as the "service.namespace" property. Can be used to track different instances of the pipeline running in parallel.
    #[clap(long, default_value = "")]
    otel_namespace: String,

    /// Endpoint on which OpenMetrics flavour metrics are available.
    #[clap(long, default_value = "127.0.0.1:9090")]
    observability_address: SocketAddr,

    #[clap(flatten)]
    select: Select,

    /// Kafka timeout for polling the broker for topic info.
    /// If this feature is failing, then increasing this value may help.
    #[clap(long, default_value = "1000")]
    poll_broker_timeout_ms: u64,

    /// Interval for refreshing the app.
    #[clap(long, default_value = "100")]
    update_app_ms: u64,

    /// Interval for refreshing the app.
    #[clap(long, default_value = "1")]
    update_search_engine_ns: u64,
}

fn main() -> miette::Result<()> {
    // set up logging
    //_ = console_log::init_with_level(log::Level::Debug);
    console_error_panic_hook::set_once();

    let args = Cli::parse();

    let consumer = supermusr_common::create_default_consumer(
        &args.common_kafka_options.broker,
        &args.common_kafka_options.username,
        &args.common_kafka_options.password,
        &args.consumer_group,
        None,
    ).into_diagnostic()?;

    let topics = args.topics.clone();
    let select = args.select.clone();
    mount_to_body(|| {
        view! {
            <App topics = topics select = select />
        }
    });
    Ok(())
}
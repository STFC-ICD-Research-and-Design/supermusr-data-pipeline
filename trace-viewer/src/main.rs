//!
mod app;
mod cli_structs;
mod finder;
mod graphics;
mod messages;
mod tui;

use chrono::{DateTime, Utc};
use clap::Parser;
use crossterm::{
    event::{self, Event},
    execute,
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode},
};
use ratatui::{Terminal, prelude::CrosstermBackend};
use rdkafka::{
    consumer::{Consumer, StreamConsumer},
    error::KafkaError,
};
use std::{fs::File, net::SocketAddr};
use supermusr_common::{
    //init_tracer,
    //tracer::{TracerEngine, TracerOptions},
    CommonKafkaOpts,
};
use tokio::{
    signal::unix::{SignalKind, signal},
    time,
};
use tracing_subscriber::{EnvFilter, Layer, layer::SubscriberExt};

use crate::{
    app::{App, AppDependencies},
    cli_structs::{Select, Topics, UserBounds},
    finder::SearchEngine,
    graphics::{GraphSaver, SvgSaver},
    tui::{Component, InputComponent},
};

type Timestamp = DateTime<Utc>;

/// [clap] derived stuct to parse command line arguments.
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
}

pub fn create_default_consumer(
    broker_address: &String,
    username: &Option<String>,
    password: &Option<String>,
    consumer_group: &String,
    topics_to_subscribe: Option<&[&str]>,
) -> Result<StreamConsumer, KafkaError> {
    // Setup consumer with arguments and default parameters.
    let consumer: StreamConsumer =
        supermusr_common::generate_kafka_client_config(broker_address, username, password)
            .set("group.id", consumer_group)
            .set("enable.partition.eof", "false")
            .set("session.timeout.ms", "6000")
            .set("enable.auto.commit", "false")
            .create()?;

    // Subscribe to if topics are provided.
    if let Some(topics_to_subscribe) = topics_to_subscribe {
        // Note this fails if the topics list is empty
        consumer.subscribe(topics_to_subscribe)?;
    }

    Ok(consumer)
}

/// Empty struct to encapsultate dependencies to inject into [App].
struct TheAppDependencies;

impl AppDependencies for TheAppDependencies {
    type MessageFinder = SearchEngine;
    type GraphSaver = SvgSaver;
}

/// Entry point.
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Cli::parse();

    /*let _tracer = init_tracer!(TracerOptions::new(
        args.otel_endpoint.as_deref(),
        args.otel_namespace.clone()
    ));*/

    std::fs::create_dir_all("Saves").expect("");
    let file = File::create("Saves/tracing.log").expect("");
    let stdout_tracer = tracing_subscriber::fmt::layer()
        .with_writer(file)
        .with_ansi(false);

    // This filter is applied to the stdout tracer
    let log_filter = EnvFilter::from_default_env();

    let subscriber =
        tracing_subscriber::Registry::default().with(stdout_tracer.with_filter(log_filter));

    //  This is only called once, so will never panic
    tracing::subscriber::set_global_default(subscriber)
        .expect("tracing::subscriber::set_global_default should only be called once");

    let consumer = create_default_consumer(
        &args.common_kafka_options.broker,
        &args.common_kafka_options.username,
        &args.common_kafka_options.password,
        &args.consumer_group,
        None,
    )?;

    // Set up terminal.
    terminal::enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let search_engine = SearchEngine::new(consumer, &args.select, &args.topics);
    let mut app = App::<TheAppDependencies>::new(search_engine, &args.select);

    let mut sigint = signal(SignalKind::interrupt())?;
    let mut app_update = tokio::time::interval(time::Duration::from_millis(100));
    let mut search_engine_update = tokio::time::interval(time::Duration::from_nanos(1));

    terminal.draw(|frame| app.render(frame, frame.area()))?;

    loop {
        tokio::select! {
            _ = app_update.tick() => {
                match event::poll(time::Duration::from_millis(10)) {
                    Ok(true) => match event::read() {
                        Ok(Event::Key(key)) => app.handle_key_press(key),
                        Err(e) => panic!("{e}"),
                        _ => {}
                    },
                    Err(e) => panic!("{e}"),
                    _ => {}
                }
                if app.changed() {
                    terminal.draw(|frame|app.render(frame, frame.area()))?;
                }
                if app.is_quit() {
                    break;
                }
                app.update();
            },
            _ = search_engine_update.tick() => {
                app.async_update().await
            },
            _ = sigint.recv() => {
                break;
            }
        }
    }
    // Clean up terminal.
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    terminal.clear()?;
    Ok(())
}

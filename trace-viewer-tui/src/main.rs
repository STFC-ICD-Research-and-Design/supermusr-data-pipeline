//! Defines main CLI struct and entry point for application.
mod app;
mod cli_structs;
mod finder;
mod graphics;
mod messages;
mod tui;

use chrono::{DateTime, Utc};
use clap::Parser;
use crossterm::{
    event, execute,
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode},
};
use ratatui::{Terminal, prelude::CrosstermBackend};
use std::{fs::File, net::SocketAddr};
use supermusr_common::CommonKafkaOpts;
use tokio::{
    signal::unix::{SignalKind, signal},
    time,
};
use tracing_subscriber::{EnvFilter, Layer, layer::SubscriberExt};

use crate::{
    app::{App, AppDependencies},
    cli_structs::{Select, Topics},
    finder::SearchEngine,
    graphics::{GraphSaver, SvgSaver},
    tui::{Component, InputComponent},
};

type Timestamp = DateTime<Utc>;

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

    let consumer = supermusr_common::create_default_consumer(
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

    let search_engine = SearchEngine::new(consumer, &args.topics, args.poll_broker_timeout_ms);
    let mut app = App::<TheAppDependencies>::new(search_engine, &args.select);

    let mut sigint = signal(SignalKind::interrupt())?;
    let mut app_update = tokio::time::interval(time::Duration::from_millis(args.update_app_ms));
    let mut search_engine_update =
        tokio::time::interval(time::Duration::from_nanos(args.update_search_engine_ns));

    terminal.draw(|frame| app.render(frame, frame.area()))?;

    loop {
        tokio::select! {
            _ = app_update.tick() => {
                match event::poll(time::Duration::from_millis(10)) {
                    Ok(true) => match event::read() {
                        Ok(event::Event::Key(key)) => app.handle_key_event(key),
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

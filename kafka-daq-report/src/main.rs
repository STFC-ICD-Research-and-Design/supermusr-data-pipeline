mod app;
mod ui;

use anyhow::Result;
use app::App;
use clap::Parser;
use crossterm::event::{EnableMouseCapture, DisableMouseCapture, Event as CEvent, self, KeyEventKind, KeyCode};
use crossterm::execute;
use crossterm::terminal::{enable_raw_mode, EnterAlternateScreen, disable_raw_mode, LeaveAlternateScreen};
use hdf5::globals::H5FD_STDIO;
use kagiyama::{AlwaysReady, Watcher};
use ratatui::prelude::{Backend, Layout, Direction, Constraint, Alignment};
use ratatui::style::{Style, Color, Modifier};
use ratatui::text::Text;
use ratatui::widgets::{Paragraph, Block, Borders, Row, Table, Cell, TableState};
use ratatui::{terminal, Frame};
use ratatui::{prelude::CrosstermBackend, Terminal};
use rdkafka::{
    consumer::{stream_consumer::StreamConsumer, CommitMode, Consumer},
    message::Message,
};
use std::{io, net::SocketAddr, path::PathBuf, thread, time::{Duration, Instant}, sync::{Arc, Mutex, mpsc}};
use streaming_types::dat1_digitizer_analog_trace_v1_generated::{
    digitizer_analog_trace_message_buffer_has_identifier, root_as_digitizer_analog_trace_message,
};
use tokio::task;
use ui::ui;

use crate::app::DAQReport;

#[derive(Debug, Parser)]
#[clap(author, version, about)]
struct Cli {
    #[clap(long)]
    broker: String,

    #[clap(long)]
    username: Option<String>,

    #[clap(long)]
    password: Option<String>,

    #[clap(long = "group")]
    consumer_group: String,

    #[clap(long)]
    trace_topic: String,

    #[clap(long, default_value = ".")]
    output: PathBuf,

    #[clap(long, default_value = "127.0.0.1:9090")]
    observability_address: SocketAddr,
}

enum Event<I> {
    Input(I),
    Tick,
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let args = Cli::parse();
    log::debug!("Args: {:?}", args);

    let mut watcher = Watcher::<AlwaysReady>::default();
    watcher.start_server(args.observability_address).await;

    let consumer: StreamConsumer =
        common::generate_kafka_client_config(&args.broker, &args.username, &args.password)
            .set("group.id", &args.consumer_group)
            .set("enable.partition.eof", "false")
            .set("session.timeout.ms", "6000")
            .set("enable.auto.commit", "false")
            .create()?;

    consumer.subscribe(&[&args.trace_topic])?;

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Initialise App
    let mut app = App::new();
    app.next();

    // Setup event polling
    let (tx, rx) = mpsc::channel();
    let tick_rate = Duration::from_millis(200);

    // Event polling thread
    thread::spawn(move || {
        let mut last_tick = Instant::now();
        loop {
            let timeout = tick_rate
                .checked_sub(last_tick.elapsed())
                .unwrap_or_else(|| Duration::from_secs(0));

            if event::poll(timeout).expect("poll works") {
                if let CEvent::Key(key) = event::read().expect("can read events") {
                    tx.send(Event::Input(key)).expect("can send events");
                }
            }

            if last_tick.elapsed() >= tick_rate {
                if let Ok(_) = tx.send(Event::Tick) {
                    last_tick = Instant::now();
                }
            }
        }
    });

    // Test data
    let daq_data = Arc::new(Mutex::new(0));

    // Message polling thread
    task::spawn(
        poll_msg(
            consumer, 
            Arc::clone(&daq_data)
        )
    );

    // Run app
    loop {
        // Draw terminal with information
        terminal.draw(|frame| ui(frame, &app.table_body, &mut app.table_state))?;

        // Poll events
        match rx.recv()? {
            Event::Input(event) => match event.code {
                KeyCode::Char('q') => break,
                KeyCode::Down => app.next(),
                KeyCode::Up => app.previous(),
                _ => (),
            },
            Event::Tick => (),
        }
        
        // app.update_table(daq_data.lock().unwrap());
    }

    // Clean up terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
    terminal.show_cursor()?;
    terminal.clear()?;

    Ok(())
}

async fn poll_kafka_msg(consumer: StreamConsumer, daq_data: Arc<Mutex<i32>>) {
    // Poll Kafka messages
    loop {
        let mut shared_data = daq_data.lock().unwrap();
        *shared_data += 1;
        match consumer.recv().await {
            Err(e) => log::warn!("Kafka error: {}", e),
            Ok(msg) => {
                log::debug!(
                    "key: '{:?}', topic: {}, partition: {}, offset: {}, timestamp: {:?}",
                    msg.key(),
                    msg.topic(),
                    msg.partition(),
                    msg.offset(),
                    msg.timestamp()
                );
    
                if let Some(payload) = msg.payload() {
                    if digitizer_analog_trace_message_buffer_has_identifier(payload) {
                        match root_as_digitizer_analog_trace_message(payload) {
                            Ok(data) => {
                                log::info!(
                                    "Trace packet: dig. ID: {}, metadata: {:?}",
                                    data.digitizer_id(),
                                    data.metadata()
                                );
                            }
                            Err(e) => {
                                log::warn!("Failed to parse message: {}", e);
                            }
                        }
                    } else {
                        log::warn!("Unexpected message type on topic \"{}\"", msg.topic());
                    }
                }
    
                consumer.commit_message(&msg, CommitMode::Async).unwrap();
            }
        };
    }
}

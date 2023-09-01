mod app;
mod ui;

use anyhow::Result;
use chrono::{DateTime, Utc};
use clap::Parser;
use crossterm::event::{self, DisableMouseCapture, EnableMouseCapture, Event as CEvent, KeyCode};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use kagiyama::{AlwaysReady, Watcher};
use ratatui::{prelude::CrosstermBackend, Terminal};
use rdkafka::{
    consumer::{stream_consumer::StreamConsumer, CommitMode, Consumer},
    message::Message,
};
use std::collections::HashMap;
use std::{
    io,
    net::SocketAddr,
    path::PathBuf,
    sync::{mpsc, Arc, Mutex},
    thread,
    time::{Duration, Instant},
};
use streaming_types::dat1_digitizer_analog_trace_v1_generated::{
    digitizer_analog_trace_message_buffer_has_identifier, root_as_digitizer_analog_trace_message,
};
use tokio::task;
use ui::ui;

use crate::app::App;

/// Holds required data for a specific digitiser.
pub struct DigitiserData {
    pub num_msg_received: usize,
    pub first_msg_timestamp: Option<DateTime<Utc>>,
    pub last_msg_timestamp: Option<DateTime<Utc>>,
    pub last_msg_frame: u32,
    pub num_channels_present: usize,
    pub has_num_channels_changed: bool,
    pub num_samples_in_first_channel: usize,
    pub is_num_samples_identical: bool,
    pub has_num_samples_changed: bool,
}

impl DigitiserData {
    /// Create a new instance with default values.
    pub fn new(
        timestamp: Option<DateTime<Utc>>,
        frame: u32,
        num_channels_present: usize,
        num_samples_in_first_channel: usize,
        is_num_samples_identical: bool,
    ) -> Self {
        DigitiserData {
            num_msg_received: 1,
            first_msg_timestamp: timestamp,
            last_msg_timestamp: timestamp,
            last_msg_frame: frame,
            num_channels_present,
            has_num_channels_changed: false,
            num_samples_in_first_channel,
            is_num_samples_identical,
            has_num_samples_changed: false,
        }
    }
}

type SharedData = Arc<Mutex<HashMap<u8, DigitiserData>>>;

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

    // Set up terminal.
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Set up app and shared data.
    let mut app = App::new();

    let shared_data: SharedData = Arc::new(Mutex::new(HashMap::new()));

    // Set up event polling.
    let (tx, rx) = mpsc::channel();
    let tick_rate = Duration::from_millis(200);

    // Event polling thread.
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

            if last_tick.elapsed() >= tick_rate && tx.send(Event::Tick).is_ok() {
                last_tick = Instant::now();
            }
        }
    });

    // Message polling thread.
    task::spawn(poll_kafka_msg(consumer, Arc::clone(&shared_data)));

    // Run app.
    loop {
        // Poll events.
        match rx.recv()? {
            Event::Input(event) => match event.code {
                KeyCode::Char('q') => break,
                KeyCode::Down => app.next(),
                KeyCode::Up => app.previous(),
                _ => (),
            },
            Event::Tick => (),
        }

        // Use the current data to regenerate the table body (may be inefficient to call every time).
        app.generate_table_body(Arc::clone(&shared_data));

        // Draw terminal using shared data.
        terminal.draw(|frame| ui(frame, &mut app))?;
    }

    // Clean up terminal.
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    terminal.clear()?;

    Ok(())
}

/// Poll kafka messages and update digitiser data.
async fn poll_kafka_msg(consumer: StreamConsumer, shared_data: SharedData) {
    loop {
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
                                // Update digitiser data.
                                let mut logged_data = shared_data.lock().unwrap();

                                let frame_number = data.metadata().frame_number();

                                let num_channels_present = match data.channels() {
                                    Some(c) => c.len(),
                                    None => 0,
                                };
                                let num_samples_in_first_channel = match data.channels() {
                                    Some(c) => c.get(0).voltage().unwrap().len(),
                                    None => 0,
                                };
                                let is_num_samples_identical = match data.channels() {
                                    Some(c) => || -> bool {
                                        for trace in c.iter() {
                                            if trace.voltage().unwrap().len()
                                                != num_samples_in_first_channel
                                            {
                                                return false;
                                            }
                                        }
                                        true
                                    }(),
                                    None => false,
                                };

                                let timestamp =
                                    data.metadata().timestamp().copied().map(|t| t.into());

                                logged_data
                                    .entry(data.digitizer_id())
                                    .and_modify(|d| {
                                        d.num_msg_received += 1;

                                        d.last_msg_timestamp = timestamp;
                                        let num_channels = match data.channels() {
                                            Some(c) => c.len(),
                                            None => 0,
                                        };
                                        if !d.has_num_channels_changed {
                                            d.has_num_channels_changed =
                                                num_channels != d.num_channels_present;
                                        }
                                        d.num_channels_present = num_channels;
                                        if !d.has_num_channels_changed {
                                            d.has_num_samples_changed = num_samples_in_first_channel
                                                != d.num_samples_in_first_channel;
                                        }
                                        d.num_samples_in_first_channel =
                                            num_samples_in_first_channel;
                                        d.is_num_samples_identical = is_num_samples_identical;
                                    })
                                    .or_insert(DigitiserData::new(
                                        timestamp,
                                        frame_number,
                                        num_channels_present,
                                        num_samples_in_first_channel,
                                        is_num_samples_identical,
                                    ));

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

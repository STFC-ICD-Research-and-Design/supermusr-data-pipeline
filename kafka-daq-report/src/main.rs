mod ui;

use anyhow::Result;
use clap::Parser;
use crossterm::event::{EnableMouseCapture, DisableMouseCapture, Event as CEvent, self, KeyCode};
use crossterm::execute;
use crossterm::terminal::{enable_raw_mode, EnterAlternateScreen, disable_raw_mode, LeaveAlternateScreen};
use kagiyama::{AlwaysReady, Watcher};
use ratatui::{prelude::CrosstermBackend, Terminal};
use rdkafka::Timestamp;
use rdkafka::{
    consumer::{stream_consumer::StreamConsumer, CommitMode, Consumer},
    message::Message,
};
use std::collections::HashMap;
use std::{io, net::SocketAddr, path::PathBuf, thread, time::{Duration, Instant}, sync::{Arc, Mutex, mpsc}};
use streaming_types::dat1_digitizer_analog_trace_v1_generated::{
    digitizer_analog_trace_message_buffer_has_identifier, root_as_digitizer_analog_trace_message,
};
use tokio::task;
use ui::ui;

pub struct DigitiserData {
    pub num_msg_received:               usize,
    pub first_msg_timestamp:            Option<Timestamp>,
    pub last_msg_timestamp:             Option<Timestamp>,
    pub last_msg_frame:                 Option<usize>,
    pub num_channels_present:           usize,
    pub has_num_channels_changed:       bool,
    pub num_samples_in_first_channel:   Option<usize>,
    pub is_num_samples_identical:       bool,
    pub has_num_samples_changed:        bool,
}

impl DigitiserData {
    pub fn default() -> Self {
        DigitiserData {
            num_msg_received:               0,
            first_msg_timestamp:            None,
            last_msg_timestamp:             None,
            last_msg_frame:                 None,
            num_channels_present:           0,
            has_num_channels_changed:       false,
            num_samples_in_first_channel:   None,
            is_num_samples_identical:       false,
            has_num_samples_changed:        false,
        }
    }

    pub fn new(timestamp: Timestamp, num_channels: usize, num_samples_in_first: usize) -> Self {
        DigitiserData {
            num_msg_received:               1,
            first_msg_timestamp:            Some(timestamp),
            last_msg_timestamp:             Some(timestamp),
            last_msg_frame:                 None,
            num_channels_present:           num_channels,
            has_num_channels_changed:       false,
            num_samples_in_first_channel:   None,
            is_num_samples_identical:       false,
            has_num_samples_changed:        false,
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

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

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
    let shared_data: SharedData =
        Arc::new(Mutex::new(HashMap::new()));

    // Message polling thread
    task::spawn(
        poll_kafka_msg(
            consumer, 
            Arc::clone(&shared_data)
        )
    );

    // Run app
    loop {
        // Draw terminal with information
        terminal.draw(|frame| ui(frame, Arc::clone(&shared_data)))?;

        // Poll events
        match rx.recv()? {
            Event::Input(event) => match event.code {
                KeyCode::Char('q') => break,
                _ => (),
            },
            Event::Tick => (),
        }
    }

    // Clean up terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
    terminal.show_cursor()?;
    terminal.clear()?;

    Ok(())
}

async fn poll_kafka_msg(consumer: StreamConsumer, shared_data: SharedData) {
    // Poll Kafka messages
    loop {
        match consumer.recv().await {
            Err(e) => log::warn!("Kafka error: {}", e),
            Ok(msg) => {
                let mut logged_data = shared_data.lock().unwrap();
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
                                let num_channels = match data.channels() {
                                    Some(c) => c.len(),
                                    None => 0,
                                };
                                /*
                                let num_samples_in_first_channel = match data.channels() {
                                    Some(c) => c.get(0).
                                    None => ()
                                };*/
                                logged_data.entry(data.digitizer_id())
                                    .and_modify(|d| {
                                        d.num_msg_received += 1;
                                        d.last_msg_timestamp = Some(msg.timestamp());
                                        let num_channels = match data.channels() {
                                            Some(c) => c.len(),
                                            None => 0,
                                        };
                                        if !d.has_num_channels_changed {
                                            d.has_num_channels_changed =
                                                num_channels != d.num_channels_present;
                                        }
                                        d.num_channels_present = num_channels;
                                    })
                                    .or_insert(DigitiserData::new(msg.timestamp(), num_channels, 0));
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

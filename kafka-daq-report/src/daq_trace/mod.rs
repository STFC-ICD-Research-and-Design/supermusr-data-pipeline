mod app;
mod ui;

use self::app::App;
use self::ui::ui;
use super::DaqTraceOpts;
use anyhow::Result;
use chrono::{DateTime, Utc};
use crossterm::event::{self, DisableMouseCapture, EnableMouseCapture, Event as CEvent, KeyCode};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::{prelude::CrosstermBackend, Terminal};
use rdkafka::{
    consumer::{stream_consumer::StreamConsumer, CommitMode, Consumer},
    message::Message,
};
use std::collections::HashMap;
use std::{
    io,
    sync::{mpsc, Arc, Mutex},
    thread,
    time::{Duration, Instant},
};
use supermusr_streaming_types::dat1_digitizer_analog_trace_v1_generated::{
    digitizer_analog_trace_message_buffer_has_identifier, root_as_digitizer_analog_trace_message,
};
use tokio::task;
use tokio::time::sleep;
use tracing::{debug, info, warn};

type DigitiserDataHashMap = Arc<Mutex<HashMap<u8, DigitiserData>>>;

enum Event<I> {
    Input(I),
    Tick,
}

/// Holds required data for a specific digitiser.
pub struct DigitiserData {
    pub msg_count: usize,
    last_msg_count: usize,
    pub msg_rate: f64,
    pub first_msg_timestamp: Option<DateTime<Utc>>,
    pub last_msg_timestamp: Option<DateTime<Utc>>,
    pub last_msg_frame: u32,
    pub num_channels_present: usize,
    pub has_num_channels_changed: bool,
    pub num_samples_in_first_channel: usize,
    pub is_num_samples_identical: bool,
    pub has_num_samples_changed: bool,
    pub bad_frame_count: usize,
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
            msg_count: 1,
            msg_rate: 0 as f64,
            last_msg_count: 1,
            first_msg_timestamp: timestamp,
            last_msg_timestamp: timestamp,
            last_msg_frame: frame,
            num_channels_present,
            has_num_channels_changed: false,
            num_samples_in_first_channel,
            is_num_samples_identical,
            has_num_samples_changed: false,
            bad_frame_count: 0,
        }
    }
}

// Trace topic diagnostic tool
pub(crate) async fn run(args: DaqTraceOpts) -> Result<()> {
    let consumer: StreamConsumer = supermusr_common::generate_kafka_client_config(
        &args.common.broker,
        &args.common.username,
        &args.common.password,
    )
    .set("group.id", &args.common.consumer_group)
    .set("enable.partition.eof", "false")
    .set("session.timeout.ms", "6000")
    .set("enable.auto.commit", "false")
    .create()?;

    consumer.subscribe(&[&args.common.trace_topic])?;

    // Set up terminal.
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Set up app and common data.
    let mut app = App::new();

    let common_dig_data_map: CommonDigitiserDataHashMap = Arc::new(Mutex::new(HashMap::new()));

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
    task::spawn(poll_kafka_msg(consumer, Arc::clone(&common_dig_data_map)));

    // Message rate calculation thread.
    task::spawn(update_message_rate(
        Arc::clone(&common_dig_data_map),
        args.message_rate_interval,
    ));

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
        app.generate_table_body(Arc::clone(&common_dig_data_map));

        // Draw terminal using common data.
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

async fn update_message_rate(
    common_dig_data_map: CommonDigitiserDataHashMap,
    recent_message_lifetime: u64,
) {
    loop {
        // Wait a set period of time before calculating average.
        sleep(Duration::from_secs(recent_message_lifetime)).await;
        let mut logged_data = common_dig_data_map.lock().unwrap();
        // Calculate message rate for each digitiser.
        for digitiser_data in logged_data.values_mut() {
            digitiser_data.msg_rate = (digitiser_data.msg_count - digitiser_data.last_msg_count)
                as f64
                / recent_message_lifetime as f64;
            digitiser_data.last_msg_count = digitiser_data.msg_count;
        }
    }
}

/// Poll kafka messages and update digitiser data.
async fn poll_kafka_msg(consumer: StreamConsumer, common_dig_data_map: CommonDigitiserDataHashMap) {
    loop {
        match consumer.recv().await {
            Err(e) => warn!("Kafka error: {}", e),
            Ok(msg) => {
                debug!(
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

                                let timestamp = data
                                    .metadata()
                                    .timestamp()
                                    .copied()
                                    .and_then(|t| t.try_into().ok());

                                let id = data.digitizer_id();
                                {
                                    let mut logged_data = common_dig_data_map.lock().unwrap();
                                    logged_data
                                        .entry(id)
                                        .and_modify(|d| {
                                            d.msg_count += 1;

                                            d.last_msg_timestamp = timestamp;
                                            d.last_msg_frame = frame_number;

                                            if timestamp.is_none() {
                                                d.bad_frame_count += 1;
                                            }

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
                                                d.has_num_samples_changed =
                                                    num_samples_in_first_channel
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
                                };

                                info!(
                                    "Trace packet: dig. ID: {}, metadata: {:?}",
                                    data.digitizer_id(),
                                    data.metadata()
                                );
                            }
                            Err(e) => {
                                warn!("Failed to parse message: {}", e);
                            }
                        }
                    } else {
                        warn!("Unexpected message type on topic \"{}\"", msg.topic());
                    }
                }

                consumer.commit_message(&msg, CommitMode::Async).unwrap();
            }
        };
    }
}

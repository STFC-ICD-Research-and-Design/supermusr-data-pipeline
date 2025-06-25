use chrono::SecondsFormat;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Gauge, LineGauge},
};
use strum::{Display, EnumString};

use crate::{
    Component,
    finder::{BrokerInfo, SearchStatus},
    tui::{ComponentStyle, ParentalFocusComponent, TextBox, TuiComponent, TuiComponentBuilder},
};

#[derive(Default, EnumString, Display)]
enum StatusMessage {
    #[default]
    #[strum(to_string = "Ready to Search. Press <Enter> to begin.")]
    Waiting,
    #[strum(to_string = "Searching for Traces.")]
    TraceSearchInProgress,
    #[strum(to_string = "Search for Trace Finished.")]
    TraceSearchFinished,
    #[strum(to_string = "Searching for Event Lists.")]
    EventListSearchInProgress,
    #[strum(to_string = "Search for Event Lists Finished.")]
    EventListSearchFinished,
    #[strum(
        to_string = "Search Complete. Found {num} traces, in {secs},{ms} ms. Press <Enter> to search again."
    )]
    SearchFinished { num: usize, secs: i64, ms: i32 },
    #[strum(to_string = "{0}")]
    Text(String),
}

/// Displays status and progress of an ongoing search, and info relating to a completed search.
pub(crate) struct Statusbar {
    parent_has_focus: bool,
    info: TuiComponent<TextBox<String>>,
    status: TuiComponent<TextBox<StatusMessage>>,
    progress: f64,
}

impl Statusbar {
    /// Create's new status bar.
    pub(crate) fn new() -> TuiComponent<Self> {
        TuiComponentBuilder::new(ComponentStyle::selectable())
            .with_block(true)
            .build(Self {
                parent_has_focus: false,
                info: TextBox::new(Default::default(), Some("Broker Info")),
                status: TextBox::new(Default::default(), Some("Status")),
                progress: 0.0,
            })
    }

    /// Set the status bar to the given [SearchStatus].
    ///
    /// # Attribute
    /// - status: TODO.
    pub(crate) fn set_status(&mut self, status: SearchStatus) {
        match status {
            SearchStatus::Off => self.status.set(StatusMessage::Waiting),
            SearchStatus::TraceSearchInProgress(progress) => {
                self.status.set(StatusMessage::TraceSearchInProgress);
                self.progress = progress;
            }
            SearchStatus::TraceSearchFinished => {
                self.status.set(StatusMessage::TraceSearchFinished);
            }
            SearchStatus::EventListSearchInProgress(progress) => {
                self.status.set(StatusMessage::EventListSearchInProgress);
                self.progress = progress;
            }
            SearchStatus::EventListSearchFinished => {
                self.status.set(StatusMessage::EventListSearchFinished);
            }
            SearchStatus::Successful { num, time } => {
                self.status.set(StatusMessage::SearchFinished {
                    num,
                    secs: time.num_seconds(),
                    ms: time.subsec_millis(),
                });
            }
            _ => {}
        }
    }

    /// Set the info bar to the given [SearchResults]
    ///
    /// # Attribute
    /// - results: TODO.
    pub(crate) fn set_broker_info_to_init(&mut self) {
        self.info.set("Broker Polling in Progress.".to_string());
    }

    /// Set the info bar to the given [SearchResults]
    ///
    /// # Attribute
    /// - results: TODO.
    pub(crate) fn set_broker_info(&mut self, broker_info: Option<BrokerInfo>) {
        const BOUND_STR_FMT: &'static str = "%Y-%m-%d %H:%M:%S";
        if let Some(broker_info) = broker_info {
            self.info.set(format!(
                "Traces: {} [{}, {}] | Eventlists: {} [{}, {}]",
                broker_info.trace.offsets.1 - broker_info.trace.offsets.0,
                broker_info
                    .trace
                    .timestamps
                    .0
                    .format(BOUND_STR_FMT)
                    .to_string(),
                broker_info
                    .trace
                    .timestamps
                    .1
                    .format(BOUND_STR_FMT)
                    .to_string(),
                broker_info.events.offsets.1 - broker_info.events.offsets.0,
                broker_info
                    .events
                    .timestamps
                    .0
                    .format(BOUND_STR_FMT)
                    .to_string(),
                broker_info
                    .events
                    .timestamps
                    .1
                    .format(BOUND_STR_FMT)
                    .to_string()
            ));
        } else {
            self.info
                .set("Poll Broker Failed. Try increasing `poll_broker_timeout_ms`.".to_string());
        }
    }
}

impl Component for Statusbar {
    fn render(&self, frame: &mut Frame, area: Rect) {
        let (info, status) = {
            let chunk = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(3), Constraint::Length(3)])
                .split(area);
            (chunk[0], chunk[1])
        };
        let (status, progress) = {
            let chunk = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Min(32), Constraint::Length(64)])
                .split(status);
            (chunk[0], chunk[1])
        };

        self.info.render(frame, info);

        let gauge = Gauge::default()
            .block(Block::new().borders(Borders::ALL))
            .style(Style::new().fg(Color::DarkGray).bg(Color::Black))
            .gauge_style(Style::new().fg(Color::LightGreen).bg(Color::Black))
            .ratio(self.progress);
        self.status.render(frame, status);
        frame.render_widget(gauge, progress);
    }
}

impl ParentalFocusComponent for Statusbar {
    fn propagate_parental_focus(&mut self, focus: bool) {
        self.parent_has_focus = focus;
    }
}

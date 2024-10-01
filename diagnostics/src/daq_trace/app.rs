use super::{data::DigitiserData, DigitiserDataHashMap};
use chrono::{DateTime, Timelike, Utc};
use ratatui::widgets::TableState;

/// Holds the current state of the program.
pub struct App {
    pub table_headers: Vec<String>,
    pub table_body: Vec<Vec<String>>,
    pub table_state: TableState,
}

impl App {
    /// Create a new instance with default values.
    pub fn new() -> App {
        App {
            table_headers: DigitiserData::generate_headers(),
            table_body: Vec::new(),
            table_state: TableState::default(),
        }
    }

    pub fn generate_table_body(&mut self, common_dig_data_map: DigitiserDataHashMap) {
        // Clear table body.
        self.table_body.clear();
        let logged_data = common_dig_data_map
            .lock()
            .expect("should be able to lock common data");
        // Sort by digitiser ID.
        let mut sorted_data: Vec<_> = logged_data.iter().collect();
        sorted_data.sort_by_key(|x| x.0);
        // Add rows to table.
        for (digitiser_id, digitiser_data) in sorted_data.iter() {
            self.table_body.push(digitiser_data.generate_row(**digitiser_id))
        }
    }

    /// Move to the next item in the table.
    pub fn next(&mut self) {
        if self.table_body.is_empty() {
            return;
        }
        let i = match self.table_state.selected() {
            Some(i) => {
                if i >= self.table_body.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.table_state.select(Some(i));
    }

    /// Move to the previous item in the table.
    pub fn previous(&mut self) {
        if self.table_body.is_empty() {
            return;
        }
        let i = match self.table_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.table_body.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.table_state.select(Some(i));
    }
}

/// Create a neatly formatted String from a timestamp.
pub(crate) fn format_timestamp(timestamp: Option<DateTime<Utc>>) -> String {
    match timestamp {
        None => "N/A".to_string(),
        Some(t) => format!(
            "{}\n{}\n{} ns",
            t.format("%d/%m/%y"),
            t.format("%H:%M:%S"),
            t.nanosecond(),
        ),
    }
}

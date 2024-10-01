use super::{data::DigitiserData, DigitiserDataHashMap};
use chrono::{DateTime, Timelike, Utc};
use ratatui::widgets::TableState;

/// Holds the current state of the program.
pub struct App {
    pub table_headers: Vec<String>,
    pub table_body: Vec<Vec<String>>,
    pub table_state: TableState
}

impl App {
    /// Create a new instance with default values.
    pub(crate) fn new() -> App {
        App {
            table_headers: DigitiserData::generate_headers(),
            table_body: Vec::new(),
            table_state: TableState::default()
        }
    }

    pub(crate) fn generate_table_body(&mut self, common_dig_data_map: DigitiserDataHashMap) {
        // Clear table body.
        self.table_body.clear();

        let logged_data = common_dig_data_map
            .lock()
            .expect("should be able to lock common data");

        let mut sorted_data: Vec<_> = logged_data.iter().collect();
        // Sort by digitiser ID.
        sorted_data.sort_by_key(|x| x.0);

        // Insert into table
        sorted_data
            .iter()
            .for_each(|(digitiser_id, digitiser_data)| {
                self.table_body
                    .push(digitiser_data.generate_row(**digitiser_id))
            });
    }

    /// Move to the next item in the table.
    pub(crate) fn selected_digitiser_channel_delta(&mut self, common_dig_data_map: DigitiserDataHashMap, delta: isize) {
        if let Some(selected_index) = self
            .table_state
            .selected() {
            let mut logged_data = common_dig_data_map
                .lock()
                .expect("should be able to lock common data");

            let mut sorted_data: Vec<_> = logged_data.iter_mut().collect();
            // Sort by digitiser ID.
            sorted_data.sort_by_key(|x| x.0);

            if let Some((_, data)) = sorted_data.get_mut(selected_index) {
                if data.num_channels_present != 0 {
                    data.channel_index = data.channel_index 
                        .checked_add((data.num_channels_present as isize + delta) as usize)
                        .expect("This should not overflow")
                        .rem_euclid(data.num_channels_present);
                }
            }
        }
    }

    /// Move to the next item in the table.
    pub(crate) fn next(&mut self) {
        if self.table_body.is_empty() {
            return;
        }

        //   Select next item, wrapping as appropriate
        let index = self
            .table_state
            .selected()
            .map(
                |index| {
                    index
                        .checked_add(1)
                        .expect("This should not overflow")
                        .rem_euclid(self.table_body.len())
                }, // Wrap around if `index > self.table_body.len()`
            )
            .unwrap_or_default();
        self.table_state.select(Some(index));
    }

    /// Move to the previous item in the table.
    pub(crate) fn previous(&mut self) {
        if self.table_body.is_empty() {
            return;
        }

        //   Select next item, wrapping as appropriate
        let index = self
            .table_state
            .selected()
            .map(
                |index| {
                    index
                        .checked_add(self.table_body.len() - 1)
                        .expect("This should not overflow")
                        .rem_euclid(self.table_body.len())
                }, // Wrap around if `index > self.table_body.len()`
            )
            .unwrap_or_default();
        self.table_state.select(Some(index));
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

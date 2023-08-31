use chrono::{Timelike, DateTime, Utc};
use ratatui::widgets::TableState;

use super::SharedData;

pub struct App {
    pub table_headers:  Vec<String>,
    pub table_body:     Vec<Vec<String>>,
    pub table_state:    TableState,
}

impl App {
    pub fn new() -> App {
        App {
            table_headers: vec![
                "Digitiser ID"                  ,   //|   1   |
                "#Msgs Received"                ,   //|   2   |
                "First Msg Timestamp"           ,   //|   3   |
                "Last Msg Timestamp"            ,   //|   4   |
                "Last Msg Frame"                ,   //|   5   |
                "#Present Channels"             ,   //|   6   |
                "#Channels Changed?"            ,   //|   7   |
                "#Samples (First Channel)"      ,   //|   8   |
                "#Samples Identical?"           ,   //|   9   |
                "#Samples Changed?"  
            ]
                .iter()
                .map(|s| s.to_string())
                .collect(),
            table_body: Vec::new(),
            table_state: TableState::default(),
        }
    }

    pub fn generate_table_body(self: &mut Self, shared_data: SharedData) {
        self.table_body.clear();
        let logged_data = shared_data.lock().unwrap();
        for (digitiser_id, digitiser_data) in logged_data.iter() {
            self.table_body.push(
                vec![
                    // 1. Digitiser ID
                    digitiser_id.to_string(),
                    // 2. Number of messages received
                    format!("{}", digitiser_data.num_msg_received),
                    // 3. First message timestamp
                    format_timestamp(digitiser_data.first_msg_timestamp),
                    // 4. Last message timestamp
                    format_timestamp(digitiser_data.last_msg_timestamp),
                    // 5. Last message frame
                    format!("{}", digitiser_data.last_msg_frame),
                    // 6. Number of channels present
                    format!("{}", digitiser_data.num_channels_present),
                    // 7. Has the number of channels changed?
                    format!("{}", 
                        match digitiser_data.has_num_channels_changed {
                            true => "Yes",
                            false => "No"
                        }
                    ),
                    // 8. Number of samples in the first channel
                    format!("{}", digitiser_data.num_samples_in_first_channel),
                    // 9. Is the number of samples identical?
                    format!("{}",
                        match digitiser_data.is_num_samples_identical {
                            true => "Yes",
                            false => "No"
                        }
                    ),
                    // 10. Has the number of samples changed?
                    format!("{}",
                        match digitiser_data.has_num_samples_changed {
                            true => "Yes",
                            false => "No"
                        }
                    )
                ]
            )
        }

    }
    

    pub fn next(self: &mut Self) {
        if self.table_body.is_empty() { return; }
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

    pub fn previous(self: &mut Self) {
        if self.table_body.is_empty() { return; }
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

fn format_timestamp(timestamp: Option<DateTime<Utc>>) -> String {
    match timestamp {
        None => "N/A".to_string(),
        Some(t) => format!(
            "{}\n{} ns",
            t.format("%H:%M:%S"),
            t.nanosecond(),
        ),
    }
}
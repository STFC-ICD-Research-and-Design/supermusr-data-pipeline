use rdkafka::Timestamp;
use ratatui::widgets::{TableState, Row};

pub type TableBody<'a> = Vec<Row<'a>>;

pub struct DigitiserData {
    pub num_msg_received:               i32,
    pub first_msg_timestamp:            Option<Timestamp>,
    pub last_msg_timestamp:             Option<Timestamp>,
    pub last_msg_frame:                 Option<i32>,
    pub num_channels_present:           i32,
    pub has_num_channels_changed:       bool,
    pub num_samples_in_first_channel:   Option<i32>,
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
}

pub struct App<'a> {
    pub table_state:    TableState,
    pub table_body:     TableBody<'a>,
}

impl App<'_> {
    pub fn new() -> App<'static> {
        App {
            table_state:    TableState::default(),
            table_body:     generate_table_rows(&DigitiserData::default()),
        }
    }

    pub fn update_table(self: &mut Self, data: &DigitiserData) {
        self.table_body = generate_table_rows(data);
    }

    pub fn next(self: &mut Self) {
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

/*
Generates table rows from data
*/
fn generate_table_rows(data: &DigitiserData) -> TableBody<'static> {
    vec![
        Row::new(vec![
            // 1. Digitiser ID
            "1".to_string(),
            // 2. Number of messages received
            format!("{}", data.num_msg_received).to_string(),
            // 3. First message timestamp
            match data.first_msg_timestamp {
                None => "N/A".to_string(),
                Some(d) => format!("{:?}", d).to_string(),
            },
            // 4. Last message timestamp
            match data.last_msg_timestamp {
                None => "N/A".to_string(),
                Some(d) => format!("{:?}", d).to_string(),
            },
            // 5. Last message frame
            match data.last_msg_frame {
                None => "N/A".to_string(),
                Some(d) => format!("{}", d).to_string(),
            },
            // 6. Number of channels present
            format!("{}", data.num_channels_present).to_string(),
            // 7. Has the number of channels changed?
            format!("{}", 
                match data.has_num_channels_changed {
                    true => "Yes",
                    false => "No"
                }
            ).to_string(),
            // 8. Number of samples in the first channel
            match data.num_samples_in_first_channel {
                None => "N/A".to_string(),
                Some(d) => format!("{}", d).to_string(),
            },
            // 9. Is the number of samples identical?
            format!("{}",
                match data.is_num_samples_identical {
                    true => "Yes",
                    false => "No"
                }
            ).to_string(),
            // 10. Has the number of samples changed?
            format!("{}",
            match data.has_num_samples_changed {
                true => "Yes",
                false => "No"
            }
        ).to_string()])
    ]    
}

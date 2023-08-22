use ratatui::widgets::{TableState, Table, Row};
use std::collections::HashMap;

pub type TableBody<'a> = Vec<Row<'a>>;

/*
pub enum DAQProperty {
    NUM_MSG_RECEIVED,
    FIRST_MSG_TIMESTAMP,
    LAST_MSG_TIMESTAMP,
    LAST_MSG_FRAME,
    NUM_CHANNELS_PRESENT,
    CHANNELS_CHANGED,
    NUM_SAMPLES_IN_FIRST_CHANNEL,
    NUM_SAMPLES_IDENTICAL,
    NUM_SAMPLES_CHANGE,
}

impl fmt::Display for Platform {
    fn fmt(&self, f: &mut fmt::Formatter) {
        match self {
            DAQProperty::NUM_MSG_RECEIVED => write!(f, "Number of messages received"),
            DAQProperty::FIRST_MSG_TIMESTAMP => write!(f, "First message timestamp"),
            DAQProperty::LAST_MSG_TIMESTAMP => write!(f, "Last message timestamp"),
            DAQProperty::LAST_MSG_FRAME => write!(f, "Last message frame number"),
            DAQProperty::NUM_CHANNELS_PRESENT => write!(f, "Number of channels in last message"),
            DAQProperty::CHANNELS_CHANGED => write!(f, "Number of channels has changed?"),
            DAQProperty::NUM_SAMPLES_IN_FIRST_CHANNEL => write!(f, "Number of samples in first channel of last message"),
            DAQProperty::NUM_SAMPLES_IDENTICAL => write!(f, "Number of samples identical in each channel?"),
            DAQProperty::NUM_SAMPLES_CHANGE => write!(f, "Number of samples has changed?"),
        }
    }
}
*/

pub struct DAQReport {
    pub num_msg_received:               i32,
    pub first_msg_timestamp:            Option<i32>,
    pub last_msg_timestamp:             Option<i32>,
    pub last_msg_frame:                 Option<i32>,
    pub num_channels_present:           i32,
    pub has_num_channels_changed:       bool,
    pub num_samples_in_first_channel:   Option<i32>,
    pub is_num_samples_identical:       bool,
    pub has_num_samples_changed:        bool,
}

impl DAQReport {
    pub fn default() -> Self {
        DAQReport {
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
    pub data:           DAQReport,
    pub table_body:     TableBody<'a>,
}

impl App<'_> {
    pub fn new() -> App<'static> {
        let mut app = App {
            table_state:    TableState::default(),
            data:           DAQReport::default(),
            table_body:     TableBody::new(),
        };
        app.update_table();
        app
    }

    pub fn update_table(self: &mut Self) {
        self.table_body = generate_table(&self.data);
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

fn generate_table(data: &DAQReport) -> TableBody<'static> {
    vec![
        Row::new(vec![
            "Number of messages received".to_string(),
            format!("{}", data.num_msg_received).to_string()
        ]),
        Row::new(vec![
            "First message timestamp".to_string(),
            match data.first_msg_timestamp {
                None => "N/A".to_string(),
                Some(d) => format!("{}", d).to_string(),
            }
        ]),
        Row::new(vec![
            "Last message timestamp".to_string(),
            match data.last_msg_timestamp {
                None => "N/A".to_string(),
                Some(d) => format!("{}", d).to_string(),
            }
        ]),
        Row::new(vec![
            "Last message frame".to_string(),
            match data.last_msg_frame {
                None => "N/A".to_string(),
                Some(d) => format!("{}", d).to_string(),
            }
        ]),
        Row::new(vec![
            "Number of present channels".to_string(),
            format!("{}", data.num_channels_present).to_string(),
        ]),
        Row::new(vec![
            "Has number of channels changed?".to_string(),
            format!("{}", 
                match data.has_num_channels_changed {
                    true => "Yes",
                    false => "No"
                }
            ).to_string(),
        ]),
        Row::new(vec![
            "Number of samples in first channel".to_string(),
            match data.num_samples_in_first_channel {
                None => "N/A".to_string(),
                Some(d) => format!("{}", d).to_string(),
            }
        ]),
        Row::new(vec![
            "Is number of samples identical?".to_string(),
            format!("{}",
                match data.is_num_samples_identical {
                    true => "Yes",
                    false => "No"
                }
            ).to_string(),
        ]),
        Row::new(vec![
            "Has number of samples changed?".to_string(),
            format!("{}",
                match data.has_num_samples_changed {
                    true => "Yes",
                    false => "No"
                }
            ).to_string(),
        ]),
    ]
    
}
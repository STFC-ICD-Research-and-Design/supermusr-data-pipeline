use chrono::{DateTime, Utc};
use supermusr_common::DigitizerId;

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::daq_trace::app::format_timestamp;

pub(crate) type DigitiserDataHashMap = Arc<Mutex<HashMap<u8, DigitiserData>>>;

enum Event<I> {
    Input(I),
    Tick,
}

/// Holds required data for a specific digitiser.
pub struct DigitiserData {
    pub msg_count: usize,
    pub last_msg_count: usize,
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

    /// Update instance with new data
    pub fn update(
        &mut self,
        timestamp: Option<DateTime<Utc>>,
        frame_number: u32,
        num_channels_present: usize,
        num_samples_in_first_channel: usize,
        is_num_samples_identical: bool,
    ) {
        self.msg_count += 1;

        self.last_msg_timestamp = timestamp;
        self.last_msg_frame = frame_number;

        if timestamp.is_none() {
            self.bad_frame_count += 1;
        }

        if !self.has_num_channels_changed {
            self.has_num_channels_changed = num_channels_present != self.num_channels_present;
        }
        self.num_channels_present = num_channels_present;
        if !self.has_num_channels_changed {
            self.has_num_samples_changed =
                num_samples_in_first_channel != self.num_samples_in_first_channel;
        }
        self.num_samples_in_first_channel = num_samples_in_first_channel;
        self.is_num_samples_identical = is_num_samples_identical;
    }

    pub(crate) fn generate_headers() -> Vec<String> {[
            "Digitiser ID",          // 1
            "#Msgs Received",        // 2
            "First Msg Timestamp",   // 3
            "Last Msg Timestamp",    // 4
            "Last Msg Frame",        // 5
            "Message Rate (Hz)",     // 6
            "#Present Channels",     // 7
            "#Channels Changed?",    // 8
            "First Channel Samples", // 9
            "#Samples Identical?",   // 10
            "#Samples Changed?",     // 11
            "#Bad Frames?",          // 12
        ]
        .into_iter()
        .map(ToString::to_string)
        .collect()
    }

    pub(crate) fn generate_row(&self, digitiser_id: DigitizerId) -> Vec<String> {
        vec![
                // 1. Digitiser ID.
                digitiser_id.to_string(),
                // 2. Number of messages received.
                format!("{}", self.msg_count),
                // 3. First message timestamp.
                format_timestamp(self.first_msg_timestamp),
                // 4. Last message timestamp.
                format_timestamp(self.last_msg_timestamp),
                // 5. Last message frame.
                format!("{}", self.last_msg_frame),
                // 6. Message rate.
                format!("{:.1}", self.msg_rate),
                // 7. Number of channels present.
                format!("{}", self.num_channels_present),
                // 8. Has the number of channels changed?
                format!(
                    "{}",
                    match self.has_num_channels_changed {
                        true => "Yes",
                        false => "No",
                    }
                ),
                // 9. Number of samples in the first channel.
                format!("{}", self.num_samples_in_first_channel),
                // 10. Is the number of samples identical?
                format!(
                    "{}",
                    match self.is_num_samples_identical {
                        true => "Yes",
                        false => "No",
                    }
                ),
                // 11. Has the number of samples changed?
                format!(
                    "{}",
                    match self.has_num_samples_changed {
                        true => "Yes",
                        false => "No",
                    }
                ),
                // 12. Number of Bad Frames
                format!("{}", self.bad_frame_count),
            ]
    }
}

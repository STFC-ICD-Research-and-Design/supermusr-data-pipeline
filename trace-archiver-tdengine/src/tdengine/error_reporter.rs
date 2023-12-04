use std::cmp::Ordering;
use supermusr_streaming_types::{
    dat1_digitizer_analog_trace_v1_generated::{ChannelTrace, DigitizerAnalogTraceMessage},
    flatbuffers::{ForwardsUOffset, Vector},
};

use super::framedata::FrameData;

/// ErrorCodes defines the codes of single errors.
/// These are combined by converting to integer types
/// composing with bitwise or. The resultant error code
/// is inserted into the database under the column "error_code".
pub(super) enum ErrorCode {
    MissingChannels = 1,
    MissingTimestamp = 2,
    NumChannelsIncorrect = 4,
    DuplicateChannelIds = 8,
    NumSamplesIncorrect = 16,
    ChannelVoltagesMissing = 32,
}

/// This struct examines a FrameData and Vector of ChanelTraces for errors
/// and records them as both an error code, and a vector of string messages.
/// The error code is read into the database, and the vector of strings logged.
#[derive(Default)]
pub(super) struct TDEngineErrorReporter {
    error: bool,
    code: u32,
    error_reports: Vec<String>,
}

impl TDEngineErrorReporter {
    pub(super) fn new() -> Self {
        TDEngineErrorReporter {
            error: false,
            code: 0,
            error_reports: Vec::<String>::new(),
        }
    }

    pub(super) fn error_code(&self) -> u32 {
        self.code
    }

    pub(super) fn report_error(&mut self, code: ErrorCode, message: String) {
        self.error_reports.push(message);
        self.code |= code as u32;
        self.error = true;
    }

    pub(super) fn test_metadata(&mut self, message: &DigitizerAnalogTraceMessage) {
        if message.channels().is_none() {
            self.report_error(ErrorCode::MissingChannels, "Channels missing".to_owned());
        }
        if message.metadata().timestamp().is_none() {
            self.report_error(ErrorCode::MissingTimestamp, "Timestamp missing".to_owned());
        }
    }

    /// Performs tests for inconsistencies and errors on a FrameData instance,
    /// and a flat buffers vector of ChanelTraces. Errors are recorded in the structure
    /// #Arguments
    /// - frame_data: a FrameData reference
    /// - channels: a FrameData reference
    pub(super) fn test_channels(
        &mut self,
        frame_data: &FrameData,
        channels: &Vector<ForwardsUOffset<ChannelTrace<'_>>>,
    ) {
        match channels.len().cmp(&frame_data.num_channels) {
            Ordering::Less => self.report_error(
                ErrorCode::NumChannelsIncorrect,
                format!(
                    "Number of channels {0} insuffient, should be {1}",
                    channels.len(),
                    frame_data.num_channels
                ),
            ),
            Ordering::Greater => self.report_error(
                ErrorCode::NumChannelsIncorrect,
                format!(
                    "Number of channels {0} too large, only the first {1} channels retained",
                    channels.len(),
                    frame_data.num_channels
                ),
            ),
            Ordering::Equal => {}
        }

        for (i, channel) in channels.iter().enumerate() {
            // If channel index is non-unique, record an error
            if channels
                .iter()
                .filter(|&c| c.channel() == channel.channel())
                .count()
                != 1
            {
                self.report_error(
                    ErrorCode::DuplicateChannelIds,
                    format!(
                        "Channel at index {i} has duplicate channel identifier of {0}",
                        channel.channel()
                    ),
                );
            }

            match channel.voltage() {
                Some(v) => {
                    if v.len() != frame_data.num_samples {
                        self.report_error(
                            ErrorCode::NumSamplesIncorrect,
                            format!(
                                "Channel at index {i} has incorrect sample count of {0}",
                                v.len()
                            ),
                        )
                    }
                }
                None => self.report_error(
                    ErrorCode::ChannelVoltagesMissing,
                    format!("Channel at index {i} has voltages null"),
                ),
            }
        }
    }
}

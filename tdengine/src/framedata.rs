use std::fmt;
use std::ops::Div;

use std::error::Error;

use anyhow::{anyhow, Result};
use chrono::{DateTime, Duration, Utc};
use common::{Channel, DigitizerId, FrameNumber};
use flatbuffers::{ForwardsUOffset, Vector};
use streaming_types::dat1_digitizer_analog_trace_v1_generated::{
    ChannelTrace, DigitizerAnalogTraceMessage,
};

use super::error::{ChannelError, FrameError, TraceMessageError};

/// Stores and handles some of the data obtained from a DigitizerAnalogTraceMessage message.
/// # Fields
/// * `timestamp` - The timestamp of the current frame.
/// * `frame_number` - The frame number of the current frame.
/// * `digitizer_id` - The id of the digitizer.
/// * `sample_time` - The duration of each sample in the current frame.
#[derive(Clone)]
pub struct FrameData {
    pub timestamp: DateTime<Utc>,
    pub digitizer_id: DigitizerId,
    pub frame_number: FrameNumber,
    pub num_channels: usize,
    pub num_samples: usize,
    pub sample_time: Duration,
    pub sample_rate: u64,
}
impl FrameData {
    /// Creates a default FrameData instance.
    /// This can't be derived directly,
    /// as Duration does not implement Default.
    pub fn default() -> Self {
        FrameData {
            timestamp: DateTime::<Utc>::default(),
            digitizer_id: DigitizerId::default(),
            frame_number: FrameNumber::default(),
            num_channels: usize::default(),
            num_samples: usize::default(),
            sample_time: Duration::nanoseconds(0),
            sample_rate: u64::default(),
        }
    }

    pub(super) fn set_channel_count(&mut self, num_channels: usize) {
        self.num_channels = num_channels;
    }

    pub(super) fn test_channel_data_non_null(
        &self,
        message: &DigitizerAnalogTraceMessage,
    ) -> Result<(), FrameError> {
        // Obtain the channel data, and error check
        message.channels().ok_or(FrameError::ChannelDataNull)?;
        Ok(())
    }
    pub(super) fn test_channel_for_errors(
        &self,
        expected_samples_count: usize,
        channel: &Option<ChannelTrace>,
    ) -> Result<(), ChannelError> {
        let channel = channel.ok_or(ChannelError::TraceMissing)?;
        let voltages = channel.voltage().ok_or(ChannelError::VoltageDataNull)?;
        if voltages.len() == expected_samples_count {
            Ok(())
        } else {
            Err(ChannelError::VoltagesMissing(voltages.len()))
        }
    }
    /// Extracts some of the data from a DigitizerAnalogTraceMessage message.
    /// Note that no channel trace data is extracted.
    /// # Arguments
    /// * `message` - A reference to a DigitizerAnalogTraceMessage message.
    /// # Returns
    /// An emtpy result, or an error.
    pub fn init(&mut self, message: &DigitizerAnalogTraceMessage) -> Result<(), TraceMessageError> {
        //  Obtain the timestamp, and error check
        self.timestamp = (*message
            .metadata()
            .timestamp()
            .ok_or(TraceMessageError::Frame(FrameError::TimestampMissing))?)
        .into();
        //  Obtain the detector data
        self.digitizer_id = message.digitizer_id();
        self.frame_number = message.metadata().frame_number();

        // Obtain the sample rate and calculate the sample time (ns)
        self.sample_rate = message.sample_rate();
        if self.sample_rate == 0 {
            return Err(TraceMessageError::Frame(FrameError::SampleRateZero));
        }
        self.sample_time = Duration::nanoseconds(1_000_000_000).div(self.sample_rate as i32);
        if self.sample_time.is_zero() {
            return Err(TraceMessageError::Frame(FrameError::SampleTimeZero));
        }

        self.test_channel_data_non_null(message)
            .map_err(|e| TraceMessageError::Frame(e))?;

        // Get the maximum number of samples from the channels,
        // Note this does not perform any tests on the channels.
        self.num_samples = message
            .channels()
            .unwrap()
            .iter()
            .map(|c| c.voltage())
            .flatten()
            .map(|v| v.len())
            .max()
            .unwrap_or_default();
        Ok(())
    }

    pub(super) fn get_table_name(&self) -> String {
        format!("d{0}", self.digitizer_id)
    }
    pub(super) fn get_frame_table_name(&self) -> String {
        format!("m{0}", self.digitizer_id)
    }

    /// Calculates the timestamp of a particular measurement relative to the timestamp of this frame.
    /// # Arguments
    /// * `measurement_number` - The measurement number to calculate the timestamp for.
    /// # Returns
    /// A `DateTime<Utc>` representing the measurement timestamp.
    pub fn calc_measurement_time(&self, measurment_number: usize) -> DateTime<Utc> {
        self.timestamp + self.sample_time * measurment_number as i32
    }
}

use super::{TDEngineError, TraceMessageErrorCode};
use anyhow::Result;
use chrono::{DateTime, Duration, Utc};
use std::ops::Div;
use supermusr_common::{DigitizerId, FrameNumber};
use supermusr_streaming_types::dat1_digitizer_analog_trace_v1_generated::DigitizerAnalogTraceMessage;

/// Stores and handles some of the data obtained from a DigitizerAnalogTraceMessage message.
/// # Fields
/// * `timestamp` - The timestamp of the current frame.
/// * `frame_number` - The frame number of the current frame.
/// * `digitizer_id` - The id of the digitizer.
/// * `sample_time` - The duration of each sample in the current frame.
#[derive(Clone)]
pub(super) struct FrameData {
    pub(super) timestamp: DateTime<Utc>,
    pub(super) digitizer_id: DigitizerId,
    pub(super) frame_number: FrameNumber,
    pub(super) num_channels: usize,
    pub(super) num_samples: usize,
    pub(super) sample_time: Duration,
    pub(super) sample_rate: u64,
}

impl Default for FrameData {
    fn default() -> Self {
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
}

impl FrameData {
    pub(super) fn set_channel_count(&mut self, num_channels: usize) {
        self.num_channels = num_channels;
    }

    /// Extracts some of the data from a DigitizerAnalogTraceMessage message.
    /// Note that no channel trace data is extracted.
    /// # Arguments
    /// * `message` - A reference to a DigitizerAnalogTraceMessage message.
    /// # Returns
    /// An emtpy result, or an error.
    pub(super) fn init(&mut self, message: &DigitizerAnalogTraceMessage) -> Result<()> {
        //  Obtain the timestamp, and error check
        self.timestamp = (*message
            .metadata()
            .timestamp()
            .ok_or(TDEngineError::TraceMessage(
                TraceMessageErrorCode::TimestampMissing,
            ))?)
        .try_into()
        .unwrap();

        //  Obtain the detector data
        self.digitizer_id = message.digitizer_id();
        self.frame_number = message.metadata().frame_number();

        // Obtain the sample rate and calculate the sample time (ns)
        self.sample_rate = message.sample_rate();
        if self.sample_rate == 0 {
            Err(TDEngineError::TraceMessage(
                TraceMessageErrorCode::SampleRateZero,
            ))?;
        }
        self.sample_time = Duration::nanoseconds(1_000_000_000).div(self.sample_rate as i32);
        if self.sample_time.is_zero() {
            Err(TDEngineError::TraceMessage(
                TraceMessageErrorCode::SampleTimeZero,
            ))?;
        }

        if message.channels().is_none() {
            Err(TDEngineError::TraceMessage(
                TraceMessageErrorCode::ChannelDataNull,
            ))?;
        }

        // Get the maximum number of samples from the channels,
        // Note this does not perform any tests on the channels.
        self.num_samples = message
            .channels()
            .unwrap()
            .iter()
            .filter_map(|c| c.voltage())
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
    pub(super) fn calc_measurement_time(&self, measurment_number: usize) -> DateTime<Utc> {
        self.timestamp + self.sample_time * measurment_number as i32
    }
}

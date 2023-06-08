use std::ops::Div;

use anyhow::{anyhow, Result};
use common::{FrameNumber, DigitizerId};
use chrono::{DateTime, Utc, Duration};
use streaming_types::dat1_digitizer_analog_trace_v1_generated::{DigitizerAnalogTraceMessage, ChannelTrace};
use flatbuffers::{Vector,ForwardsUOffset};

/// Stores and handles some of the data obtained from a DigitizerAnalogTraceMessage message.
/// # Fields
/// * `timestamp` - The timestamp of the current frame.
/// * `frame_number` - The frame number of the current frame.
/// * `digitizer_id` - The id of the digitizer.
/// * `sample_time` - The duration of each sample in the current frame.
#[derive(Clone)]
pub struct FrameData {
    pub timestamp: DateTime<Utc>,
    pub digitizer_id : DigitizerId,
    pub frame_number : FrameNumber,
    pub sample_time : Duration,
}
impl FrameData {
    /// Creates a default FrameData instance.
    /// This can't be derived directly,
    /// as Duration does not implement Default.
    pub fn default() -> Self { FrameData {
        timestamp: DateTime::<Utc>::default(),
        digitizer_id: DigitizerId::default(),
        frame_number: FrameNumber::default(),
        sample_time: Duration::nanoseconds(0),
     } }

    /// Extracts some of the data from a DigitizerAnalogTraceMessage message.
    /// Note that no channel trace data is extracted.
    /// # Arguments
    /// * `message` - A reference to a DigitizerAnalogTraceMessage message.
    /// # Returns
    /// An emtpy result, or an error.
    pub fn init(&mut self, message: &DigitizerAnalogTraceMessage) -> Result<()> {
        //  Obtain the timestamp, and error check
        self.timestamp = (*message.metadata().timestamp().ok_or(anyhow!("no timestamp in message"))?).into();
        //  Obtain the detector data
        self.digitizer_id = message.digitizer_id();
        self.frame_number = message.metadata().frame_number();

        // Obtain the sample rate and calculate the sample time (ns)
        let sample_rate: u64 = message.sample_rate();
        if sample_rate == 0 { return Err(anyhow!("Sample rate zero"));}
        self.sample_time = Duration::nanoseconds(1_000_000_000).div(sample_rate as i32);
        if self.sample_time.is_zero() { return Err(anyhow!("Sample time zero"));}
        Ok(())
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
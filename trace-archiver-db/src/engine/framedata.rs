use std::ops::Div;

use anyhow::{anyhow, Result};
use common::{FrameNumber, DigitizerId, Channel};
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
    pub num_channels : usize,
    pub num_samples : usize,
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
        num_channels: usize::default(),
        num_samples : usize::default(),
        sample_time: Duration::nanoseconds(0),
    } }

    pub(crate) fn set_channel_count (&mut self, num_channels : usize) {
        self.num_channels = num_channels;
    }

    fn test_channels_exist_and_have_data(&self, message: &DigitizerAnalogTraceMessage) -> Result<()> {
        // Obtain the channel data, and error check
        for c in message.channels()
                    .ok_or(anyhow!("no channel data in message"))?
                    .iter()
                    .filter(|c| c.voltage().is_none())
                    .map(|c|c.channel()) {
            return Err(anyhow!("Missing intensities for channel {c}"));
        }
        if message.channels().unwrap().iter().len() == 0 {
            Err(anyhow!("Message contains zero channels: {0:?}",message))
        } else {
            Ok(())
        }
    }
    fn test_channel_and_sample_count(&self, message: &DigitizerAnalogTraceMessage) -> Result<()> {
        for c in message.channels().unwrap().iter() {
            if c.voltage().unwrap().len() != self.num_samples {
                return Err(anyhow!("Channel {0} contains {1} samples, expected {2}",c.channel(),c.voltage().unwrap().len(), self.num_samples));
            }
        }
        if message.channels().unwrap().iter().len() != self.num_channels {
            Err(anyhow!("Message contains {0} channels, expected {1}: {2:?}",
                message.channels().unwrap().iter().len(),
                self.num_channels,
                message))
        } else {
            Ok(())
        }
}

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

        self.test_channels_exist_and_have_data(message)?;
        self.num_samples = message.channels().unwrap().iter().next().unwrap().voltage().unwrap_or_default().iter().len();
        self.test_channel_and_sample_count(message)?;
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
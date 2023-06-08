use std::ops::Div;

use anyhow::{anyhow, Result};
use streaming_types::dat1_digitizer_analog_trace_v1_generated::DigitizerAnalogTraceMessage;
use async_trait::async_trait;

pub mod tdengine;
pub mod influxdb;
pub mod framedata;

#[async_trait]
pub trait TimeSeriesEngine {
    async fn process_message(&mut self, msg: &DigitizerAnalogTraceMessage) -> Result<()>;
    async fn post_message(&mut self) -> Result<String>;
}

/// Checks the channel trace data exists and returns an error if not.
/// #Arguments
/// *message - A reference to the DigitizerAnalogTraceMessage to check
pub fn test_channels(message: &DigitizerAnalogTraceMessage) -> Result<()> {
    // Obtain the channel data, and error check
    for c in message.channels()
                .ok_or(anyhow!("no channel data in message"))?
                .iter()
                .filter(|c| c.voltage().is_none())
                .map(|c|c.channel()) {
        return Err(anyhow!("Missing intensities for channel {c}"));
    }
    if message.channels().unwrap().iter().len() == 0 {
        Err(anyhow!("Zero channels in message: {:?}", message))
    } else {
        Ok(())
    }
}
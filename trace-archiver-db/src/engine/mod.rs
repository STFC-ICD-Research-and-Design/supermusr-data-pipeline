use std::ops::Div;

use anyhow::{anyhow, Result};
use streaming_types::dat1_digitizer_analog_trace_v1_generated::DigitizerAnalogTraceMessage;
use async_trait::async_trait;

use self::error::TraceMessageError;

pub(crate) mod tdengine;
//pub mod influxdb;
pub(crate) mod framedata;
pub(crate) mod error;
mod tdengine_login;
mod tdengine_views;
mod error_reporter;

#[async_trait]
pub trait TimeSeriesEngine {
    async fn process_message(&mut self, msg: &DigitizerAnalogTraceMessage) -> Result<(),error::Error>;
    async fn post_message(&mut self) -> Result<usize,error::Error>;
}
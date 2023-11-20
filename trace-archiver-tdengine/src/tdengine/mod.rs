//#![allow(dead_code)]

use anyhow::Result;
use async_trait::async_trait;
use streaming_types::dat1_digitizer_analog_trace_v1_generated::DigitizerAnalogTraceMessage;

pub mod error_reporter;
pub mod framedata;
mod views;
pub mod wrapper;

mod error;
use error::{StatementErrorCode, TDEngineError, TraceMessageErrorCode};

#[async_trait]
pub(crate) trait TimeSeriesEngine {
    async fn process_message(&mut self, msg: &DigitizerAnalogTraceMessage) -> Result<()>;
    async fn post_message(&mut self) -> Result<usize>;
}

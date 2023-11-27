pub mod error_reporter;
pub mod framedata;
pub mod wrapper;
mod views;
mod error;

use anyhow::Result;
use async_trait::async_trait;
use streaming_types::dat1_digitizer_analog_trace_v1_generated::DigitizerAnalogTraceMessage;
use error::{StatementErrorCode, TDEngineError, TraceMessageErrorCode};

#[async_trait]
pub(crate) trait TimeSeriesEngine {
    async fn process_message(&mut self, msg: &DigitizerAnalogTraceMessage) -> Result<()>;
    async fn post_message(&mut self) -> Result<usize>;
}

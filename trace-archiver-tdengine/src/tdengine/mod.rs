mod error;
pub mod error_reporter;
pub mod framedata;
mod views;
pub mod wrapper;

use error::{StatementErrorCode, TDEngineError, TraceMessageErrorCode};
use supermusr_streaming_types::dat2_digitizer_analog_trace_v2_generated::DigitizerAnalogTraceMessage;

pub(crate) trait TimeSeriesEngine {
    async fn process_message(&mut self, msg: &DigitizerAnalogTraceMessage) -> anyhow::Result<()>;
    async fn post_message(&mut self) -> anyhow::Result<usize>;
}

use std::fmt::Display;

use taos::taos_query::RawError;

#[derive(Debug)]
pub(crate) enum StatementErrorCode {
    Init,
    Prepare,
    SetTableName,
    SetTags,
    Bind,
    AddBatch,
    Execute,
}

#[derive(Debug)]
pub(crate) enum TraceMessageErrorCode {
    TimestampMissing,
    SampleRateZero,
    SampleTimeZero,
    SampleTimeMissing,
    CannotCalcMeasurementTime,
    ChannelDataNull,
    ChannelsMissing,
}

#[derive(Debug)]
pub(crate) enum TDEngineError {
    TaosBuilder(RawError),
    TaosStmt(StatementErrorCode, RawError),
    SqlError(String, RawError),
    TraceMessage(TraceMessageErrorCode),
}
impl Display for TDEngineError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{self:?}"))
    }
}
impl std::error::Error for TDEngineError {}

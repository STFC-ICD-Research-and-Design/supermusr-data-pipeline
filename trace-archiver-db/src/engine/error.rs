use std::fmt;

use taos::*;
use taos::taos_query::TBuilder;

#[derive(Debug)]
pub enum EVError {
    NotFound(String),
}


#[derive(Debug)]
pub enum StatementError {
    Init, Prepare, SetTableName,SetTags,Bind,AddBatch,Execute,
}
#[derive(Debug)]
pub enum SQLError {
    DropDatabase, CreateDatabase, UseDatabase, CreateTemplateTable, CreateErrorReportTable, QueryData,
}
#[derive(Debug)]
pub enum TDEngineError {
    TaosBuilder(<TaosBuilder as TBuilder>::Error),
    Stmt(StatementError,<Stmt as Bindable<Taos>>::Error),
    SQL(SQLError, String, <Taos as AsyncQueryable>::Error),
}

#[derive(Debug)]
pub enum ChannelError {
    TraceMissing,
    VoltageDataNull,
    VoltagesMissing(usize),
}

#[derive(Debug)]
pub enum FrameError {
    TimestampMissing,
    SampleRateZero,
    SampleTimeZero,
    ChannelDataNull,
    ChannelErrors(Vec<Result<(),ChannelError>>),
}

#[derive(Debug)]
pub enum TraceMessageError {
    Frame(FrameError),
    Channel(ChannelError),
}

impl From<FrameError> for TraceMessageError { fn from(value: FrameError) -> Self { TraceMessageError::Frame(value) } }
impl From<ChannelError> for TraceMessageError { fn from(value: ChannelError) -> Self { TraceMessageError::Channel(value) } }


#[derive(Debug)]
pub enum Error {
    EnvironmentVariable(EVError),
    TDEngine(TDEngineError),
    TraceMessage(TraceMessageError),
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(),fmt::Error> {
        Ok(fmt.write_fmt(format_args!("{:?}", self))?)
    }
}

impl std::error::Error for Error {}

impl From<EVError> for Error { fn from(value: EVError) -> Self { Error::EnvironmentVariable(value) } }
impl From<TDEngineError> for Error { fn from(value: TDEngineError) -> Self { Error::TDEngine(value) } }
impl From<TraceMessageError> for Error { fn from(value: TraceMessageError) -> Self { Error::TraceMessage(value) } }

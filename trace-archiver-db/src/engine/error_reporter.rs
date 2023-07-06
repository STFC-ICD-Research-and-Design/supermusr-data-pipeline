use chrono::{Timelike, DateTime, Utc};
use common::{FrameNumber, DigitizerId};
use flatbuffers::{ForwardsUOffset, Vector};
use anyhow::Result;
use itertools::Itertools;
use streaming_types::{dat1_digitizer_analog_trace_v1_generated::{ChannelTrace, DigitizerAnalogTraceMessage}, frame_metadata_v1_generated::{GpsTime, FrameMetadataV1}};
use taos::{Taos, Stmt, Bindable, ColumnView, Value, taos_query::common::{Timestamp, views::TimestampView}};

use super::{error::{self, TDEngineError, StatementError}, framedata::FrameData};

#[derive(Default)]
pub struct TDEngineErrorReporter {
    error : bool,
    error_reports : Vec<String>,
}

impl TDEngineErrorReporter {
    pub(super) fn new() -> Self { TDEngineErrorReporter {
        error : false,
        error_reports : Vec::<String>::new(),
    }}

    pub(super) fn has_error(&self) -> bool { self.error }
    pub(super) fn num_errors(&self) -> usize { self.error_reports.len() }
    pub(super) fn reports_iter(&self) -> core::slice::Iter<String> { self.error_reports.iter() }

    pub(super) fn report_error(&mut self, message : String) {
        self.error_reports.push(message);
        self.error = true;
    }
    
    pub(super) fn test_metadata(&mut self, message : &DigitizerAnalogTraceMessage) {
        if message.channels().is_none() {
            self.report_error("Channels missing".to_owned());
        }
        if message.metadata().timestamp().is_none() {
            self.report_error("Timestamp missing".to_owned());
        }
    }
    
    pub(super) fn test_channels<'a>(&mut self, frame_data : &FrameData, channels: &Vector<ForwardsUOffset<ChannelTrace<'a>>>) {

        if channels.len() < frame_data.num_channels {
            self.report_error(format!("Number of channels {0} insuffient, should be {1}",channels.len(),frame_data.num_channels));
        } else if channels.len() > frame_data.num_channels  {
            self.report_error(format!("Number of channels {0} too large, only the first {1} channels retained",channels.len(),frame_data.num_channels));
        }

        for (i,channel) in channels.iter().enumerate(){
            // If channel index is non-unique, record an error
            if channels.iter().filter(|&c| c.channel() == channel.channel()).count() != 1 {
                self.report_error(format!("Channel at index {i} has duplicate channel identifier of {0}", channel.channel()));
            }
            
            match channel.voltage() {
                Some(v) => if v.len() != frame_data.num_samples {
                    self.report_error(format!("Channel at index {i} has incorrect sample count of {0}", v.len()))
                },
                None => self.report_error(format!("Channel at index {i} has voltages null")),
            }
        }
    }

    /// Logs all errors that have been found along with the appropriate metadata
    /// #Arguments
    /// *metadata - The FrameMetadataV1 instance that came with the message
    /// *digitizer_id - The identifier of the digitizer of the current frame
    pub(super) fn flush_reports(&mut self, metadata : &FrameMetadataV1, digitizer_id : DigitizerId) {
        if !self.error { return }
        self.error = false;

        if self.error_reports.is_empty() { return; }

        let timestamp : Option<DateTime<Utc>> = metadata.timestamp().map(|ts| (*ts).into() );
        log::error!("[{0}]: {1} errors recorded for digitizer {2}, frame number {3} at timestamp {4}",
            self.error_reports.len(),
            Utc::now().to_string(),
            digitizer_id,
            metadata.frame_number(),
            match timestamp {Some(ts) => ts.to_string(), None => "[Timestamp Not Available]".to_owned()},
        );
        for error_report in &self.error_reports {
            log::error!("{error_report}");
        }
        self.error_reports.clear();
    }
}

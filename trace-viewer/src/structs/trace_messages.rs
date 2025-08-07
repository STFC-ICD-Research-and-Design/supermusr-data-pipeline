use cfg_if::cfg_if;
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TraceSummary {
    pub date: String,
    pub time: String,
    pub id: u8,
    pub channels: Vec<u32>,
    pub index: usize,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct SelectedTraceIndex {
    pub(crate) index: usize,
    pub(crate) channel: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TracePlotly {
    pub trace_data: String,
    pub eventlist_data: Option<String>,
    pub layout: String,
}

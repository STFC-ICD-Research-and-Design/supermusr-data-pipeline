use digital_muon_common::FrameNumber;
use serde::Deserialize;

#[derive(Clone, Deserialize)]
pub(crate) enum ReadCommand {
    #[serde(rename = "f")]
    FrameRange(FrameNumber, FrameNumber),
    #[serde(rename = "fc")]
    FrameCount(FrameNumber, usize),
    #[serde(rename = "i")]
    IndexRange(usize, usize),
    #[serde(rename = "ic")]
    IndexCount(usize, usize),
    #[serde(rename = "t")]
    TimestampRange(String, String),
    #[serde(rename = "tc")]
    TimestampCount(String, usize),
    #[serde(rename = "all")]
    All,
}

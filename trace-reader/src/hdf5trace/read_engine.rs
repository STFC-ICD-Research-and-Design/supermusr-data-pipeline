use serde::Deserialize;

#[derive(Clone, Deserialize)]
pub(crate) struct ReadSequence(Vec<ReadCommand>);

impl ReadSequence {
    pub(crate) fn new(source: &str) -> Result<Self,serde_json::Error> {
        serde_json::from_str(source)
    }
}

#[derive(Clone, Deserialize)]
pub(crate) enum ReadCommand {
    #[serde(rename="f")]
    FrameRange(u64, u64),
    #[serde(rename="fc")]
    FrameCount(u64, usize),
    #[serde(rename="i")]
    IndexRange(usize, usize),
    #[serde(rename="ic")]
    IndexCount(usize, usize),
    #[serde(rename="t")]
    TimestampRange(String, String),
    #[serde(rename="tc")]
    TimestampCount(String, usize),
    #[serde(rename= "all")]
    All,
}
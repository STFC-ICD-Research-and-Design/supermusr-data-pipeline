use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};

use supermusr_common::{Channel, Intensity, Time};
use supermusr_streaming_types::{
    aev1_frame_assembled_event_v1_generated::FrameAssembledEventListMessage,
    dev1_digitizer_event_v1_generated::DigitizerEventListMessage, flatbuffers::Vector,
    frame_metadata_v1_generated::FrameMetadataV1,
};
#[derive(Debug)]
pub(crate) struct GenericEventMessage<'a> {
    pub(crate) timestamp: DateTime<Utc>,
    pub(crate) metadata: FrameMetadataV1<'a>,
    pub(crate) time: Option<Vector<'a, Time>>,
    pub(crate) channel: Option<Vector<'a, Channel>>,
    pub(crate) voltage: Option<Vector<'a, Intensity>>,
}

fn extract_timestamp_from_message(metadata: &FrameMetadataV1) -> Result<DateTime<Utc>> {
    Ok(Into::<DateTime<Utc>>::into(
        *metadata
            .timestamp()
            .ok_or(anyhow!("Message timestamp missing."))?,
    ))
}

impl<'a> GenericEventMessage<'a> {
    pub(crate) fn from_frame_assembled_event_list_message(
        message: FrameAssembledEventListMessage<'a>,
    ) -> Result<Self> {
        Ok(GenericEventMessage::<'a> {
            timestamp: extract_timestamp_from_message(&message.metadata())?,
            metadata: message.metadata(),
            time: message.time(),
            channel: message.channel(),
            voltage: message.voltage(),
        })
    }

    pub(crate) fn from_digitizer_event_list_message(
        message: DigitizerEventListMessage<'a>,
    ) -> Result<Self> {
        Ok(GenericEventMessage::<'a> {
            timestamp: extract_timestamp_from_message(&message.metadata())?,
            metadata: message.metadata(),
            time: message.time(),
            channel: message.channel(),
            voltage: message.voltage(),
        })
    }
}

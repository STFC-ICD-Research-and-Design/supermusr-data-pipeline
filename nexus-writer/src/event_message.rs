/// This type is to generalise the digitiser and frame-assembled event list message
/// In future versions it might be desirable to only opperate on frame-assembled
/// event list messages
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
    Ok((*metadata
        .timestamp()
        .ok_or(anyhow!("Message timestamp missing."))?)
    .try_into()?)
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

#[cfg(test)]
pub(crate) mod test {
    use super::GenericEventMessage;
    use supermusr_common::DigitizerId;
    use supermusr_streaming_types::{
        aev1_frame_assembled_event_v1_generated::{
            finish_frame_assembled_event_list_message_buffer,
            root_as_frame_assembled_event_list_message, FrameAssembledEventListMessage,
            FrameAssembledEventListMessageArgs,
        },
        dev1_digitizer_event_v1_generated::{
            finish_digitizer_event_list_message_buffer, root_as_digitizer_event_list_message,
            DigitizerEventListMessage, DigitizerEventListMessageArgs,
        },
        flatbuffers::{FlatBufferBuilder, InvalidFlatbuffer},
        frame_metadata_v1_generated::{FrameMetadataV1, FrameMetadataV1Args, GpsTime},
    };

    fn create_metadata(timestamp: &GpsTime) -> FrameMetadataV1Args<'_> {
        FrameMetadataV1Args {
            timestamp: Some(timestamp),
            period_number: 0,
            protons_per_pulse: 0,
            running: false,
            frame_number: 0,
            veto_flags: 0,
        }
    }

    fn create_digitiser_message<'a, 'b: 'a>(
        fbb: &'b mut FlatBufferBuilder,
        timestamp: &GpsTime,
        digitizer_id: DigitizerId,
    ) -> Result<DigitizerEventListMessage<'a>, InvalidFlatbuffer> {
        let metadata = FrameMetadataV1::create(fbb, &create_metadata(timestamp));
        let args = DigitizerEventListMessageArgs {
            digitizer_id,
            metadata: Some(metadata),
            ..Default::default()
        };
        let message = DigitizerEventListMessage::create(fbb, &args);
        finish_digitizer_event_list_message_buffer(fbb, message);
        root_as_digitizer_event_list_message(fbb.finished_data())
    }

    pub(crate) fn create_frame_assembled_message<'a, 'b: 'a>(
        fbb: &'b mut FlatBufferBuilder,
        timestamp: &GpsTime,
    ) -> Result<FrameAssembledEventListMessage<'a>, InvalidFlatbuffer> {
        let metadata = FrameMetadataV1::create(fbb, &create_metadata(timestamp));
        let args = FrameAssembledEventListMessageArgs {
            metadata: Some(metadata),
            ..Default::default()
        };
        let message = FrameAssembledEventListMessage::create(fbb, &args);
        finish_frame_assembled_event_list_message_buffer(fbb, message);
        root_as_frame_assembled_event_list_message(fbb.finished_data())
    }

    #[test]
    fn generic_events_equal() {
        let mut fbb = FlatBufferBuilder::new();

        let ts = GpsTime::new(0, 1, 0, 0, 16, 0, 0, 0);
        let message = create_digitiser_message(&mut fbb, &ts, 23).unwrap();
        let m1 = GenericEventMessage::from_digitizer_event_list_message(message).unwrap();

        let mut fbb = FlatBufferBuilder::new();
        let message = create_frame_assembled_message(&mut fbb, &ts).unwrap();
        let m2 = GenericEventMessage::from_frame_assembled_event_list_message(message).unwrap();

        assert_eq!(m1.metadata.timestamp(), m2.metadata.timestamp());
        assert_eq!(
            m1.metadata.protons_per_pulse(),
            m2.metadata.protons_per_pulse()
        );
        assert_eq!(m1.metadata.period_number(), m2.metadata.period_number());
        assert_eq!(m1.metadata.running(), m2.metadata.running());
        assert_eq!(m1.metadata.veto_flags(), m2.metadata.veto_flags());
        assert_eq!(m1.metadata.frame_number(), m2.metadata.frame_number());
        assert_eq!(m1.timestamp, m2.timestamp);
    }
}

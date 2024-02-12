use crate::frame_metadata_v1_generated::FrameMetadataV1;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FrameMetadata {
    pub timestamp: DateTime<Utc>,
    pub period_number: u64,
    pub protons_per_pulse: u8,
    pub running: bool,
    pub frame_number: u32,
    pub veto_flags: u16,
}

impl<'a> From<FrameMetadataV1<'a>> for FrameMetadata {
    fn from(metadata: FrameMetadataV1<'a>) -> Self {
        Self {
            timestamp: (*metadata.timestamp().unwrap()).into(),
            period_number: metadata.period_number(),
            protons_per_pulse: metadata.protons_per_pulse(),
            running: metadata.running(),
            frame_number: metadata.frame_number(),
            veto_flags: metadata.veto_flags(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        dev1_digitizer_event_v1_generated::{
            finish_digitizer_event_list_message_buffer, root_as_digitizer_event_list_message,
            DigitizerEventListMessage, DigitizerEventListMessageArgs,
        },
        flatbuffers::FlatBufferBuilder,
        frame_metadata_v1_generated::{FrameMetadataV1Args, GpsTime},
    };

    #[test]
    fn test_frame_metadata_from_flatbuffers() {
        let mut fbb = FlatBufferBuilder::new();

        let timestamp = Utc::now();
        let gps_time: GpsTime = timestamp.into();

        let metadata = FrameMetadataV1Args {
            period_number: 12,
            protons_per_pulse: 8,
            running: true,
            frame_number: 559,
            timestamp: Some(&gps_time),
            veto_flags: 2,
        };
        let metadata = FrameMetadataV1::create(&mut fbb, &metadata);

        let channel = vec![0, 0, 0, 0, 1, 1, 1, 1, 0, 0];
        let time = vec![0, 5, 1, 8, 1, 8, 10, 1, 9, 6];

        let event_count = time.len();
        let channel = Some(fbb.create_vector::<u32>(&channel));
        let time = Some(fbb.create_vector::<u32>(&time));
        let voltage = Some(fbb.create_vector::<u16>(&vec![0; event_count]));

        let message = DigitizerEventListMessageArgs {
            digitizer_id: 0,
            metadata: Some(metadata),
            time,
            channel,
            voltage,
        };
        let message = DigitizerEventListMessage::create(&mut fbb, &message);
        finish_digitizer_event_list_message_buffer(&mut fbb, message);

        let message = fbb.finished_data().to_vec();
        let message = root_as_digitizer_event_list_message(&message).unwrap();

        let frame_metadata: FrameMetadata = message.metadata().into();

        assert_eq!(frame_metadata.timestamp, timestamp);
        assert_eq!(frame_metadata.period_number, 12);
        assert_eq!(frame_metadata.protons_per_pulse, 8);
        assert!(frame_metadata.running);
        assert_eq!(frame_metadata.frame_number, 559);
        assert_eq!(frame_metadata.veto_flags, 2);
    }
}

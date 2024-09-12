use crate::{
    frame_metadata_v2_generated::FrameMetadataV2, time_conversions::GpsTimeConversionError,
};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Eq)]
pub struct FrameMetadata {
    pub timestamp: DateTime<Utc>,
    pub period_number: u64,
    pub protons_per_pulse: u8,
    pub running: bool,
    pub frame_number: u32,
    pub veto_flags: u16,
}

impl FrameMetadata {
    pub fn equals_ignoring_veto_flags(&self, other: &Self) -> bool {
        self.timestamp == other.timestamp
            && self.period_number == other.period_number
            && self.protons_per_pulse == other.protons_per_pulse
            && self.running == other.running
            && self.frame_number == other.frame_number
    }
}

/// This is a temporary implementation whilst the issue with veto flags being unequal in different digitisers persists.
/// When that is solved we should replace this implementation block with the derived PartialEq.
impl PartialEq for FrameMetadata {
    fn eq(&self, other: &Self) -> bool {
        self.equals_ignoring_veto_flags(other)
    }
}

impl<'a> TryFrom<FrameMetadataV2<'a>> for FrameMetadata {
    type Error = GpsTimeConversionError;

    fn try_from(metadata: FrameMetadataV2<'a>) -> Result<Self, Self::Error> {
        Ok(Self {
            timestamp: (*metadata.timestamp().expect("timestamp should be present")).try_into()?,
            period_number: metadata.period_number(),
            protons_per_pulse: metadata.protons_per_pulse(),
            running: metadata.running(),
            frame_number: metadata.frame_number(),
            veto_flags: metadata.veto_flags(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        dev2_digitizer_event_v2_generated::{
            finish_digitizer_event_list_message_buffer, root_as_digitizer_event_list_message,
            DigitizerEventListMessage, DigitizerEventListMessageArgs,
        },
        flatbuffers::FlatBufferBuilder,
        frame_metadata_v2_generated::{FrameMetadataV2Args, GpsTime},
    };

    #[test]
    fn test_frame_metadata_from_flatbuffers() {
        let mut fbb = FlatBufferBuilder::new();

        let timestamp = Utc::now();
        let gps_time: GpsTime = timestamp.into();

        let metadata = FrameMetadataV2Args {
            period_number: 12,
            protons_per_pulse: 8,
            running: true,
            frame_number: 559,
            timestamp: Some(&gps_time),
            veto_flags: 2,
        };
        let metadata = FrameMetadataV2::create(&mut fbb, &metadata);

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

        let frame_metadata: FrameMetadata = message.metadata().try_into().unwrap();

        assert_eq!(frame_metadata.timestamp, timestamp);
        assert_eq!(frame_metadata.period_number, 12);
        assert_eq!(frame_metadata.protons_per_pulse, 8);
        assert!(frame_metadata.running);
        assert_eq!(frame_metadata.frame_number, 559);
        assert_eq!(frame_metadata.veto_flags, 2);
    }

    #[test]
    fn test_are_frames_equal() {
        let m1 = FrameMetadata {
            period_number: 12,
            protons_per_pulse: 8,
            running: true,
            frame_number: 559,
            timestamp: DateTime::from_timestamp_nanos(934856374698347),
            veto_flags: 2,
        };
        let m2 = FrameMetadata {
            period_number: 18,
            protons_per_pulse: 8,
            running: true,
            frame_number: 559,
            timestamp: DateTime::from_timestamp_nanos(934856374698347),
            veto_flags: 2,
        };
        let m3 = FrameMetadata {
            period_number: 12,
            protons_per_pulse: 2,
            running: true,
            frame_number: 559,
            timestamp: DateTime::from_timestamp_nanos(934856374698347),
            veto_flags: 2,
        };
        let m4 = FrameMetadata {
            period_number: 12,
            protons_per_pulse: 8,
            running: false,
            frame_number: 559,
            timestamp: DateTime::from_timestamp_nanos(934856374698347),
            veto_flags: 2,
        };
        let m5 = FrameMetadata {
            period_number: 12,
            protons_per_pulse: 8,
            running: true,
            frame_number: 55,
            timestamp: DateTime::from_timestamp_nanos(934856374698347),
            veto_flags: 2,
        };
        let m6 = FrameMetadata {
            period_number: 12,
            protons_per_pulse: 8,
            running: true,
            frame_number: 559,
            timestamp: DateTime::from_timestamp_nanos(934856374698348),
            veto_flags: 2,
        };
        let m7 = FrameMetadata {
            period_number: 12,
            protons_per_pulse: 8,
            running: true,
            frame_number: 559,
            timestamp: DateTime::from_timestamp_nanos(934856374698347),
            veto_flags: 0,
        };

        // m1 has equal frame with m1
        assert!(m1.equals_ignoring_veto_flags(&m1));
        // The following comparisons should evalute to false
        assert!(!m1.equals_ignoring_veto_flags(&m2));
        assert!(!m1.equals_ignoring_veto_flags(&m3));
        assert!(!m1.equals_ignoring_veto_flags(&m4));
        assert!(!m1.equals_ignoring_veto_flags(&m5));
        assert!(!m1.equals_ignoring_veto_flags(&m6));
        // This one should be true however
        assert!(m1.equals_ignoring_veto_flags(&m7));
    }
}

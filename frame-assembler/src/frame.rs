use crate::{config::DigitiserConfigs, event_data::EventData};
use anyhow::Result;
use chrono::NaiveDateTime;
use flatbuffers::FlatBufferBuilder;
use std::time::Instant;
use streaming_types::{
    aev1_frame_assembled_event_v1_generated::{
        finish_frame_assembled_event_list_message_buffer, FrameAssembledEventListMessage,
        FrameAssembledEventListMessageArgs,
    },
    dev1_digitizer_event_v1_generated::DigitizerEventListMessage,
    frame_metadata_v1_generated::{FrameMetadataV1, FrameMetadataV1Args, GpsTime},
};

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct StatusPacket {
    timestamp: NaiveDateTime,
    period_number: u64,
    protons_per_pulse: u8,
    running: bool,
    frame_number: u32,
    veto_flags: u16,
}

impl Default for StatusPacket {
    fn default() -> Self {
        Self {
            timestamp: NaiveDateTime::from_timestamp(0, 0),
            period_number: 0,
            protons_per_pulse: 0,
            running: false,
            frame_number: 0,
            veto_flags: 0,
        }
    }
}

impl From<FrameMetadataV1<'_>> for StatusPacket {
    fn from(s: FrameMetadataV1) -> Self {
        Self {
            timestamp: (*s.timestamp().unwrap()).into(),
            period_number: s.period_number(),
            protons_per_pulse: s.protons_per_pulse(),
            running: s.running(),
            frame_number: s.frame_number(),
            veto_flags: s.veto_flags(),
        }
    }
}

pub(crate) struct Frame {
    birth: Instant,
    pub(crate) status: StatusPacket,
    digitizers: Vec<u8>,
    events: EventData,
}

impl Frame {
    pub(crate) fn new(status: StatusPacket) -> Self {
        Self {
            birth: Instant::now(),
            status,
            digitizers: Vec::<u8>::default(),
            events: EventData::default(),
        }
    }

    pub(crate) fn push(&mut self, msg: &DigitizerEventListMessage) -> Result<()> {
        let dig_id = msg.digitizer_id();

        if self.digitizers.contains(&dig_id) {
            log::warn!(
                "Data from digitizer with ID {} already present in this frame buffer",
                dig_id
            );
        }

        self.digitizers.push(dig_id);
        self.events.push(msg);

        Ok(())
    }

    pub(crate) fn is_complete(&self, digitizers: &DigitiserConfigs) -> bool {
        if digitizers.len() != self.digitizers.len() {
            return false;
        }

        digitizers
            .iter()
            .map(|dig| self.digitizers.contains(&dig.id))
            .all(|r| r)
    }

    pub(crate) fn as_payload(&self) -> Result<Vec<u8>> {
        let mut fbb = FlatBufferBuilder::new();

        let time: GpsTime = self.status.timestamp.into();
        let metadata = FrameMetadataV1Args {
            frame_number: self.status.frame_number,
            period_number: self.status.period_number,
            running: self.status.running,
            protons_per_pulse: self.status.protons_per_pulse,
            timestamp: Some(&time),
            veto_flags: 0,
        };
        let metadata = FrameMetadataV1::create(&mut fbb, &metadata);

        let time = Some(fbb.create_vector(&self.events.time));
        let voltage = Some(fbb.create_vector(&self.events.voltage));
        let channel = Some(fbb.create_vector(&self.events.channel));

        let message = FrameAssembledEventListMessageArgs {
            metadata: Some(metadata),
            time,
            voltage,
            channel,
        };
        let message = FrameAssembledEventListMessage::create(&mut fbb, &message);
        finish_frame_assembled_event_list_message_buffer(&mut fbb, message);

        Ok(fbb.finished_data().to_vec())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::DigitiserConfig;

    #[test]
    fn test_frame_is_complete() {
        let dig_configs: DigitiserConfigs = vec![
            DigitiserConfig { id: 0 },
            DigitiserConfig { id: 1 },
            DigitiserConfig { id: 2 },
        ];

        let frame = Frame {
            birth: Instant::now(),
            status: StatusPacket::default(),
            digitizers: vec![1, 2, 0],
            events: EventData::default(),
        };

        assert!(frame.is_complete(&dig_configs));
    }

    #[test]
    fn test_frame_is_complete_fail_length() {
        let dig_configs: DigitiserConfigs = vec![
            DigitiserConfig { id: 0 },
            DigitiserConfig { id: 1 },
            DigitiserConfig { id: 2 },
        ];

        let frame = Frame {
            birth: Instant::now(),
            status: StatusPacket::default(),
            digitizers: vec![],
            events: EventData::default(),
        };

        assert!(!frame.is_complete(&dig_configs));
    }

    #[test]
    fn test_frame_is_complete_fail_ids() {
        let dig_configs: DigitiserConfigs = vec![
            DigitiserConfig { id: 0 },
            DigitiserConfig { id: 1 },
            DigitiserConfig { id: 2 },
        ];

        let frame = Frame {
            birth: Instant::now(),
            status: StatusPacket::default(),
            digitizers: vec![1, 2, 3],
            events: EventData::default(),
        };

        assert!(!frame.is_complete(&dig_configs));
    }
}

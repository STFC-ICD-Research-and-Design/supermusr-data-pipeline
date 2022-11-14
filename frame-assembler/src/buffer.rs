use crate::{config::DigitiserConfigs, frame::Frame};
use anyhow::Result;
use streaming_types::dev1_digitizer_event_v1_generated::DigitizerEventListMessage;

pub(crate) struct Buffer {
    dig_configs: DigitiserConfigs,
    data: Vec<Frame>,
}

impl Buffer {
    pub(crate) fn new(dig_configs: DigitiserConfigs) -> Self {
        Self {
            dig_configs,
            data: Vec::default(),
        }
    }

    pub(crate) fn push(&mut self, msg: &DigitizerEventListMessage) -> Result<()> {
        let status = msg.metadata().into();
        if let Some(i) = self.data.iter_mut().find(|i| i.status == status) {
            i.push(msg)?;
        } else {
            let mut frame = Frame::new(status);
            frame.push(msg)?;
            self.data.push(frame);
        }
        Ok(())
    }

    pub(crate) fn any_frames_ready(&self) -> bool {
        self.data
            .iter()
            .map(|i| i.is_complete(&self.dig_configs))
            .any(|i| i)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::DigitiserConfig;
    use flatbuffers::FlatBufferBuilder;
    use streaming_types::{
        dev1_digitizer_event_v1_generated::{
            finish_digitizer_event_list_message_buffer, root_as_digitizer_event_list_message,
            DigitizerEventListMessageArgs,
        },
        frame_metadata_v1_generated::{FrameMetadataV1, FrameMetadataV1Args, GpsTime},
    };

    #[test]
    fn test_buffer_push() {
        let dig_configs: DigitiserConfigs =
            vec![DigitiserConfig { id: 0 }, DigitiserConfig { id: 1 }];

        let mut buffer = Buffer::new(dig_configs);
        assert_eq!(buffer.data.len(), 0);
        assert!(!buffer.any_frames_ready());

        let mut fbb = FlatBufferBuilder::new();

        {
            fbb.reset();

            let time = GpsTime::new(22, 205, 10, 55, 30, 0, 0, 20);

            let metadata = FrameMetadataV1Args {
                frame_number: 0,
                period_number: 0,
                protons_per_pulse: 0,
                running: true,
                timestamp: Some(&time),
                veto_flags: 0,
            };
            let metadata = FrameMetadataV1::create(&mut fbb, &metadata);

            let num_events = 20;
            let time = Some(fbb.create_vector(&vec![0_u32; num_events]));
            let voltage = Some(fbb.create_vector(&vec![0_u16; num_events]));
            let channel = Some(fbb.create_vector(&vec![0_u32; num_events]));

            let message = DigitizerEventListMessageArgs {
                digitizer_id: 0,
                metadata: Some(metadata),
                time,
                voltage,
                channel,
            };
            let message = DigitizerEventListMessage::create(&mut fbb, &message);
            finish_digitizer_event_list_message_buffer(&mut fbb, message);
            let message = root_as_digitizer_event_list_message(fbb.finished_data()).unwrap();

            assert!(buffer.push(&message).is_ok());
            assert_eq!(buffer.data.len(), 1);
            assert!(!buffer.any_frames_ready());
        }

        {
            fbb.reset();

            let time = GpsTime::new(22, 205, 10, 55, 30, 0, 0, 20);

            let metadata = FrameMetadataV1Args {
                frame_number: 0,
                period_number: 0,
                protons_per_pulse: 0,
                running: true,
                timestamp: Some(&time),
                veto_flags: 0,
            };
            let metadata = FrameMetadataV1::create(&mut fbb, &metadata);

            let num_events = 20;
            let time = Some(fbb.create_vector(&vec![0_u32; num_events]));
            let voltage = Some(fbb.create_vector(&vec![0_u16; num_events]));
            let channel = Some(fbb.create_vector(&vec![0_u32; num_events]));

            let message = DigitizerEventListMessageArgs {
                digitizer_id: 1,
                metadata: Some(metadata),
                time,
                voltage,
                channel,
            };
            let message = DigitizerEventListMessage::create(&mut fbb, &message);
            finish_digitizer_event_list_message_buffer(&mut fbb, message);
            let message = root_as_digitizer_event_list_message(fbb.finished_data()).unwrap();

            assert!(buffer.push(&message).is_ok());
            assert_eq!(buffer.data.len(), 1);
            assert!(buffer.any_frames_ready());
        }

        {
            fbb.reset();

            let time = GpsTime::new(22, 205, 10, 55, 30, 0, 0, 20);

            let metadata = FrameMetadataV1Args {
                frame_number: 1,
                period_number: 0,
                protons_per_pulse: 0,
                running: true,
                timestamp: Some(&time),
                veto_flags: 0,
            };
            let metadata = FrameMetadataV1::create(&mut fbb, &metadata);

            let num_events = 20;
            let time = Some(fbb.create_vector(&vec![0_u32; num_events]));
            let voltage = Some(fbb.create_vector(&vec![0_u16; num_events]));
            let channel = Some(fbb.create_vector(&vec![0_u32; num_events]));

            let message = DigitizerEventListMessageArgs {
                digitizer_id: 1,
                metadata: Some(metadata),
                time,
                voltage,
                channel,
            };
            let message = DigitizerEventListMessage::create(&mut fbb, &message);
            finish_digitizer_event_list_message_buffer(&mut fbb, message);
            let message = root_as_digitizer_event_list_message(fbb.finished_data()).unwrap();

            assert!(buffer.push(&message).is_ok());
            assert_eq!(buffer.data.len(), 2);
            assert!(buffer.any_frames_ready());
        }
    }
}

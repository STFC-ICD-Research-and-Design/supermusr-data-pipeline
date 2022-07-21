use super::base::BaseFile;
use anyhow::{anyhow, Result};
use hdf5::Dataset;
use ndarray::{s, Array};
use std::path::Path;
use streaming_types::aev1_frame_assembled_event_v1_generated::FrameAssembledEventListMessage;

pub(crate) struct EventFile {
    base: BaseFile,

    event_time: Dataset,
    event_channel: Dataset,
    event_voltage: Dataset,
}

impl EventFile {
    pub(crate) fn create(filename: &Path) -> Result<Self> {
        let base = BaseFile::create(filename)?;

        let event_time = base
            .file
            .new_dataset::<u32>()
            .shape((0..,))
            .create("event_data/time")?;

        let event_channel = base
            .file
            .new_dataset::<u32>()
            .shape((0..,))
            .create("event_data/channel")?;

        let event_voltage = base
            .file
            .new_dataset::<u32>()
            .shape((0..,))
            .create("event_data/voltage")?;

        Ok(EventFile {
            base,
            event_time,
            event_channel,
            event_voltage,
        })
    }

    pub(crate) fn push(&self, data: &FrameAssembledEventListMessage) -> Result<()> {
        let time = data.time().unwrap();
        let voltage = data.voltage().unwrap();
        let channel = data.channel().unwrap();

        if time.len() != voltage.len() || time.len() != channel.len() {
            return Err(anyhow!(
                "Event dataset sizes do not match (|time|={}, |voltage|={}, |channel|={})",
                time.len(),
                voltage.len(),
                channel.len()
            ));
        }

        let mut data_shape = self.event_time.shape();
        let frame_idx = data_shape[0];

        data_shape[0] += time.len();
        self.event_time.resize(data_shape.clone())?;
        self.event_voltage.resize(data_shape.clone())?;
        self.event_channel.resize(data_shape)?;

        {
            let time = time.safe_slice().to_vec();
            let time = Array::from_vec(time);
            self.event_time.write_slice(&time, s![frame_idx..])?;
        }

        {
            let voltage = voltage.safe_slice().to_vec();
            let voltage = Array::from_vec(voltage);
            self.event_voltage.write_slice(&voltage, s![frame_idx..])?;
        }

        {
            let channel = channel.safe_slice().to_vec();
            let channel = Array::from_vec(channel);
            self.event_channel.write_slice(&channel, s![frame_idx..])?;
        }

        self.base
            .new_frame((*data.status().timestamp().unwrap()).into(), frame_idx)?;

        self.base.file.flush()?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use flatbuffers::FlatBufferBuilder;
    use std::{env, fs, path::PathBuf};
    use streaming_types::{
        aev1_frame_assembled_event_v1_generated::{
            finish_frame_assembled_event_list_message_buffer,
            root_as_frame_assembled_event_list_message, FrameAssembledEventListMessage,
            FrameAssembledEventListMessageArgs,
        },
        status_packet_v1_generated::{GpsTime, StatusPacketV1, StatusPacketV1Args},
    };

    fn create_test_filename(name: &str) -> PathBuf {
        let mut path = env::temp_dir();
        path.push(format!("{}.h5", name));
        path
    }

    fn push_frame(file: &EventFile, num_events: usize, frame_number: u32, time: GpsTime) {
        let mut fbb = FlatBufferBuilder::new();

        let status_packet = StatusPacketV1Args {
            frame_number,
            period_number: 0,
            protons_per_pulse: 0,
            running: true,
            timestamp: Some(&time),
        };
        let status_packet = StatusPacketV1::create(&mut fbb, &status_packet);

        let time = Some(fbb.create_vector(&vec![frame_number; num_events]));
        let voltage = Some(fbb.create_vector(&vec![frame_number as u16; num_events]));
        let channel = Some(fbb.create_vector(&vec![frame_number; num_events]));

        let message = FrameAssembledEventListMessageArgs {
            status: Some(status_packet),
            time,
            voltage,
            channel,
        };
        let message = FrameAssembledEventListMessage::create(&mut fbb, &message);
        finish_frame_assembled_event_list_message_buffer(&mut fbb, message);

        let message = root_as_frame_assembled_event_list_message(fbb.finished_data()).unwrap();
        assert!(file.push(&message).is_ok());
    }

    #[test]
    fn test_basic() {
        let filepath = create_test_filename("EventFile_test_basic");
        let file = EventFile::create(&filepath).unwrap();
        let _ = fs::remove_file(filepath);

        push_frame(&file, 20, 0, GpsTime::new(22, 7, 4, 10, 55, 30, 0, 0, 0));
        push_frame(&file, 50, 1, GpsTime::new(22, 7, 4, 10, 55, 30, 20, 0, 0));
        push_frame(&file, 42, 2, GpsTime::new(22, 7, 4, 10, 55, 30, 40, 0, 0));

        let file = file.base.file;

        let expected_shape = vec![20 + 50 + 42];

        let time = file.dataset("event_data/time").unwrap();
        assert_eq!(time.shape(), expected_shape);

        let channel = file.dataset("event_data/channel").unwrap();
        assert_eq!(channel.shape(), expected_shape);

        let voltage = file.dataset("event_data/voltage").unwrap();
        assert_eq!(voltage.shape(), expected_shape);
    }
}

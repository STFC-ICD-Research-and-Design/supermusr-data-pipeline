use super::base::BaseFile;
use anyhow::Result;
use hdf5::Dataset;
use ndarray::{s, Array};
use std::path::Path;
use streaming_types::dat1_digitizer_analog_trace_v1_generated::DigitizerAnalogTraceMessage;

pub(crate) struct TraceFile {
    base: BaseFile,

    detector_data: Dataset,
}

impl TraceFile {
    pub(crate) fn create(filename: &Path, channels: usize) -> Result<Self> {
        let base = BaseFile::create(filename)?;

        let detector_data = base
            .file
            .new_dataset::<u16>()
            .shape((channels, 0..))
            .create("detector_data")?;

        Ok(TraceFile {
            base,
            detector_data,
        })
    }

    pub(crate) fn push(&self, data: &DigitizerAnalogTraceMessage) -> Result<()> {
        // TODO: handle a stream from multiple digitizers

        let mut det_data_shape = self.detector_data.shape();
        let frame_idx = det_data_shape[1];

        det_data_shape[1] += data.channels().unwrap().get(0).voltage().unwrap().len();
        self.detector_data.resize(det_data_shape)?;

        for channel in data.channels().unwrap().iter() {
            let channel_number = usize::try_from(channel.channel())?;

            let intensity = channel.voltage().unwrap().safe_slice().to_vec();
            let intensity = Array::from_vec(intensity);

            self.detector_data
                .write_slice(&intensity, s![channel_number, frame_idx..])?;
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
        dat1_digitizer_analog_trace_v1_generated::{
            finish_digitizer_analog_trace_message_buffer, root_as_digitizer_analog_trace_message,
            ChannelTrace, ChannelTraceArgs, DigitizerAnalogTraceMessage,
            DigitizerAnalogTraceMessageArgs,
        },
        status_packet_v1_generated::{GpsTime, StatusPacketV1, StatusPacketV1Args},
    };

    fn create_test_filename(name: &str) -> PathBuf {
        let mut path = env::temp_dir();
        path.push(format!("{}.h5", name));
        path
    }

    fn push_frame(file: &TraceFile, num_time_points: usize, frame_number: u32, time: GpsTime) {
        let mut fbb = FlatBufferBuilder::new();

        let status_packet = StatusPacketV1Args {
            frame_number,
            period_number: 0,
            protons_per_pulse: 0,
            running: true,
            timestamp: Some(&time),
        };
        let status_packet = StatusPacketV1::create(&mut fbb, &status_packet);

        let voltage = Some(fbb.create_vector::<u16>(&vec![0; num_time_points]));
        let channel0 = ChannelTrace::create(
            &mut fbb,
            &ChannelTraceArgs {
                channel: 0,
                voltage,
            },
        );

        let voltage = Some(fbb.create_vector::<u16>(&vec![1; num_time_points]));
        let channel1 = ChannelTrace::create(
            &mut fbb,
            &ChannelTraceArgs {
                channel: 1,
                voltage,
            },
        );

        let message = DigitizerAnalogTraceMessageArgs {
            digitizer_id: 0,
            status: Some(status_packet),
            sample_rate: 0,
            channels: Some(fbb.create_vector(&[channel0, channel1])),
        };
        let message = DigitizerAnalogTraceMessage::create(&mut fbb, &message);
        finish_digitizer_analog_trace_message_buffer(&mut fbb, message);

        let message = root_as_digitizer_analog_trace_message(fbb.finished_data()).unwrap();
        assert!(file.push(&message).is_ok());
    }

    #[test]
    fn test_basic() {
        let num_channels = 2;
        let num_time_points = 20;

        let filepath = create_test_filename("TraceFile_test_basic");
        let file = TraceFile::create(&filepath, num_channels).unwrap();
        let _ = fs::remove_file(filepath);

        push_frame(
            &file,
            num_time_points,
            0,
            GpsTime::new(22, 7, 4, 10, 55, 30, 0, 0, 0),
        );

        push_frame(
            &file,
            num_time_points,
            1,
            GpsTime::new(22, 7, 4, 10, 55, 30, 20, 0, 0),
        );

        push_frame(
            &file,
            num_time_points,
            2,
            GpsTime::new(22, 7, 4, 10, 55, 30, 40, 0, 0),
        );

        let file = file.base.file;

        let detector_data = file.dataset("detector_data").unwrap();
        assert_eq!(
            detector_data.shape(),
            vec![num_channels, 3 * num_time_points]
        );
    }
}

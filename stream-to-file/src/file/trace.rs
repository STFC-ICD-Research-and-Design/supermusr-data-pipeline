use super::base::BaseFile;
use anyhow::{anyhow, Result};
use common::{channel_index, Intensity, SampleRate, CHANNELS_PER_DIGITIZER};
use hdf5::Dataset;
use ndarray::{s, Array, Array0, Array1};
use ndarray_stats::QuantileExt;
use std::path::Path;
use streaming_types::dat1_digitizer_analog_trace_v1_generated::DigitizerAnalogTraceMessage;

pub(crate) struct TraceFile {
    base: BaseFile,
    sample_rate: Dataset,
    detector_data: Dataset,
    det_data_extents: Array1<usize>,
}

impl TraceFile {
    pub(crate) fn create(filename: &Path, digitizer_count: usize) -> Result<Self> {
        let base = BaseFile::create(filename)?;

        let channel_count = digitizer_count * CHANNELS_PER_DIGITIZER;

        let sample_rate = base
            .file
            .new_dataset::<SampleRate>()
            .create("sample_rate")?;
        sample_rate.write_scalar(&0)?;

        let detector_data = base
            .file
            .new_dataset::<Intensity>()
            .shape((channel_count, 0..))
            .create("detector_data")?;

        Ok(TraceFile {
            base,
            sample_rate,
            detector_data,
            det_data_extents: Array1::zeros((digitizer_count,)),
        })
    }

    pub(crate) fn push(&mut self, data: &DigitizerAnalogTraceMessage) -> Result<()> {
        let old_sample_rate = self.sample_rate.read_scalar::<u64>()?;
        if old_sample_rate > 0 && old_sample_rate != data.sample_rate() {
            return Err(anyhow!(
                "Sample rate has changed (old={}, new={})",
                old_sample_rate,
                data.sample_rate()
            ));
        } else if old_sample_rate == 0 {
            self.sample_rate.write_scalar(&data.sample_rate())?;
        }

        let old_det_data_shape = self.detector_data.shape();

        let frame_det_data_start_idx = match self.base.find_frame_metadata_index(
            data.metadata().frame_number(),
            (*data.metadata().timestamp().unwrap()).into(),
        ) {
            // If this frame is known then use the index into the detector data associated with it.
            Some(metadata_index) => {
                let frame_index: Array0<u64> = self
                    .base
                    .frame_start_index
                    .read_slice::<u64, _, _>(s![metadata_index])
                    .expect("frame index should be read");
                *frame_index.first().expect("doot") as usize
            }
            // If the frame has not been seen before then add it to the end of the last data
            // received for that digitizer.
            None => self.det_data_extents[data.digitizer_id() as usize],
        };

        self.det_data_extents[data.digitizer_id() as usize] +=
            data.channels().unwrap().get(0).voltage().unwrap().len();

        let mut new_det_data_shape = old_det_data_shape.clone();
        new_det_data_shape[1] = *self.det_data_extents.max().unwrap();

        if new_det_data_shape != old_det_data_shape {
            self.detector_data.resize(new_det_data_shape).unwrap();
        }

        for channel in data.channels().unwrap().iter() {
            let channel_number = channel_index(
                data.digitizer_id() as usize,
                usize::try_from(channel.channel()).unwrap(),
            );

            let intensity = channel.voltage().unwrap().iter().collect();
            let intensity = Array::from_vec(intensity);

            self.detector_data
                .write_slice(
                    &intensity,
                    s![
                        channel_number,
                        frame_det_data_start_idx..frame_det_data_start_idx + intensity.len()
                    ],
                )
                .unwrap();
        }

        self.base.new_frame(
            data.metadata().frame_number(),
            (*data.metadata().timestamp().unwrap()).into(),
            frame_det_data_start_idx,
        )?;

        self.base.file.flush().unwrap();

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use flatbuffers::FlatBufferBuilder;
    use ndarray::{arr1, arr2};
    use std::{env, fs, path::PathBuf};
    use streaming_types::{
        dat1_digitizer_analog_trace_v1_generated::{
            finish_digitizer_analog_trace_message_buffer, root_as_digitizer_analog_trace_message,
            ChannelTrace, ChannelTraceArgs, DigitizerAnalogTraceMessage,
            DigitizerAnalogTraceMessageArgs,
        },
        frame_metadata_v1_generated::{FrameMetadataV1, FrameMetadataV1Args, GpsTime},
    };

    fn create_test_filename(name: &str) -> PathBuf {
        let mut path = env::temp_dir();
        path.push(format!("{}.h5", name));
        path
    }

    fn push_frame(
        file: &mut TraceFile,
        num_time_points: usize,
        frame_number: u32,
        time: GpsTime,
        channel_offset: u32,
        digitizer_id: u8,
    ) {
        let mut fbb = FlatBufferBuilder::new();

        let metadata = FrameMetadataV1Args {
            frame_number,
            period_number: 0,
            protons_per_pulse: 0,
            running: true,
            timestamp: Some(&time),
            veto_flags: 0,
        };
        let metadata = FrameMetadataV1::create(&mut fbb, &metadata);

        let mut voltage: Vec<Intensity> = vec![10; num_time_points];
        voltage[0] = digitizer_id as Intensity;
        voltage[1] = frame_number as Intensity;
        let voltage = Some(fbb.create_vector::<Intensity>(&voltage));
        let channel0 = ChannelTrace::create(
            &mut fbb,
            &ChannelTraceArgs {
                channel: channel_offset,
                voltage,
            },
        );

        let mut voltage: Vec<Intensity> = vec![11; num_time_points];
        voltage[0] = digitizer_id as Intensity;
        voltage[1] = frame_number as Intensity;
        let voltage = Some(fbb.create_vector::<Intensity>(&voltage));
        let channel1 = ChannelTrace::create(
            &mut fbb,
            &ChannelTraceArgs {
                channel: channel_offset + 1,
                voltage,
            },
        );

        let message = DigitizerAnalogTraceMessageArgs {
            digitizer_id,
            metadata: Some(metadata),
            sample_rate: 1_000_000_000,
            channels: Some(fbb.create_vector(&[channel0, channel1])),
        };
        let message = DigitizerAnalogTraceMessage::create(&mut fbb, &message);
        finish_digitizer_analog_trace_message_buffer(&mut fbb, message);

        let message = root_as_digitizer_analog_trace_message(fbb.finished_data()).unwrap();
        assert!(file.push(&message).is_ok());
    }

    #[test]
    fn test_basic() {
        let num_digitizers = 1;
        let num_time_points = 20;
        let num_channels = num_digitizers * CHANNELS_PER_DIGITIZER;
        let num_frames = 3;
        let num_measurements = num_frames * num_time_points;

        let filepath = create_test_filename("TraceFile_test_basic");
        let mut file = TraceFile::create(&filepath, num_digitizers).unwrap();
        let _ = fs::remove_file(filepath);

        push_frame(
            &mut file,
            num_time_points,
            0,
            GpsTime::new(22, 205, 10, 55, 30, 0, 0, 0),
            0,
            0,
        );

        push_frame(
            &mut file,
            num_time_points,
            1,
            GpsTime::new(22, 205, 10, 55, 30, 20, 0, 0),
            0,
            0,
        );

        push_frame(
            &mut file,
            num_time_points,
            2,
            GpsTime::new(22, 205, 10, 55, 30, 40, 0, 0),
            0,
            0,
        );

        let file = file.base.file;

        let sample_rate = file.dataset("sample_rate").unwrap();
        assert_eq!(sample_rate.read_scalar::<u64>().unwrap(), 1_000_000_000);

        let detector_data = file.dataset("detector_data").unwrap();
        assert_eq!(detector_data.shape(), vec![num_channels, num_measurements]);

        assert_eq!(
            detector_data
                .read_slice::<Intensity, _, _>(s![.., 0..3])
                .unwrap(),
            arr2(&[
                [0, 0, 10],
                [0, 0, 11],
                [0, 0, 0],
                [0, 0, 0],
                [0, 0, 0],
                [0, 0, 0],
                [0, 0, 0],
                [0, 0, 0],
            ])
        );

        assert_eq!(
            detector_data
                .read_slice::<Intensity, _, _>(s![.., num_time_points..num_time_points + 3])
                .unwrap(),
            arr2(&[
                [0, 1, 10],
                [0, 1, 11],
                [0, 0, 0],
                [0, 0, 0],
                [0, 0, 0],
                [0, 0, 0],
                [0, 0, 0],
                [0, 0, 0],
            ])
        );
    }

    #[test]
    fn test_multiple_digitizers() {
        let num_digitizers = 3;
        let num_time_points = 20;
        let num_channels = num_digitizers * CHANNELS_PER_DIGITIZER;
        let num_frames = 3;
        let num_measurements = num_frames * num_time_points;

        let filepath = create_test_filename("TraceFile_test_multiple_digitizers");
        let mut file = TraceFile::create(&filepath, num_digitizers).unwrap();
        let _ = fs::remove_file(filepath);

        push_frame(
            &mut file,
            num_time_points,
            0,
            GpsTime::new(22, 205, 10, 55, 30, 0, 0, 0),
            0,
            0,
        );

        push_frame(
            &mut file,
            num_time_points,
            1,
            GpsTime::new(22, 205, 10, 55, 30, 20, 0, 0),
            0,
            0,
        );

        push_frame(
            &mut file,
            num_time_points,
            2,
            GpsTime::new(22, 205, 10, 55, 30, 40, 0, 0),
            0,
            0,
        );

        push_frame(
            &mut file,
            num_time_points,
            0,
            GpsTime::new(22, 205, 10, 55, 30, 0, 0, 0),
            0,
            1,
        );

        push_frame(
            &mut file,
            num_time_points,
            1,
            GpsTime::new(22, 205, 10, 55, 30, 20, 0, 0),
            0,
            1,
        );

        push_frame(
            &mut file,
            num_time_points,
            2,
            GpsTime::new(22, 205, 10, 55, 30, 40, 0, 0),
            0,
            1,
        );

        push_frame(
            &mut file,
            num_time_points,
            0,
            GpsTime::new(22, 205, 10, 55, 30, 0, 0, 0),
            0,
            2,
        );

        push_frame(
            &mut file,
            num_time_points,
            1,
            GpsTime::new(22, 205, 10, 55, 30, 20, 0, 0),
            0,
            2,
        );

        push_frame(
            &mut file,
            num_time_points,
            2,
            GpsTime::new(22, 205, 10, 55, 30, 40, 0, 0),
            0,
            2,
        );

        let file = file.base.file;

        let frame_start_index = file.dataset("frame_start_index").unwrap();
        assert_eq!(frame_start_index.shape(), vec![num_frames]);

        assert_eq!(
            frame_start_index.read_1d::<usize>().unwrap(),
            arr1(&[0, num_time_points, num_time_points * 2])
        );

        let detector_data = file.dataset("detector_data").unwrap();
        assert_eq!(detector_data.shape(), vec![num_channels, num_measurements]);

        assert_eq!(
            detector_data
                .read_slice::<Intensity, _, _>(s![.., 0..3])
                .unwrap(),
            arr2(&[
                [0, 0, 10],
                [0, 0, 11],
                [0, 0, 0],
                [0, 0, 0],
                [0, 0, 0],
                [0, 0, 0],
                [0, 0, 0],
                [0, 0, 0],
                [1, 0, 10],
                [1, 0, 11],
                [0, 0, 0],
                [0, 0, 0],
                [0, 0, 0],
                [0, 0, 0],
                [0, 0, 0],
                [0, 0, 0],
                [2, 0, 10],
                [2, 0, 11],
                [0, 0, 0],
                [0, 0, 0],
                [0, 0, 0],
                [0, 0, 0],
                [0, 0, 0],
                [0, 0, 0],
            ])
        );

        assert_eq!(
            detector_data
                .read_slice::<Intensity, _, _>(s![.., num_time_points..num_time_points + 3])
                .unwrap(),
            arr2(&[
                [0, 1, 10],
                [0, 1, 11],
                [0, 0, 0],
                [0, 0, 0],
                [0, 0, 0],
                [0, 0, 0],
                [0, 0, 0],
                [0, 0, 0],
                [1, 1, 10],
                [1, 1, 11],
                [0, 0, 0],
                [0, 0, 0],
                [0, 0, 0],
                [0, 0, 0],
                [0, 0, 0],
                [0, 0, 0],
                [2, 1, 10],
                [2, 1, 11],
                [0, 0, 0],
                [0, 0, 0],
                [0, 0, 0],
                [0, 0, 0],
                [0, 0, 0],
                [0, 0, 0],
            ])
        );
    }

    #[test]
    fn test_multiple_digitizers_missing_data() {
        let num_digitizers = 3;
        let num_time_points = 20;
        let num_channels = num_digitizers * CHANNELS_PER_DIGITIZER;
        let num_frames = 3;
        let num_measurements = num_frames * num_time_points;

        let filepath = create_test_filename("TraceFile_test_multiple_digitizers_missing_data");
        let mut file = TraceFile::create(&filepath, num_digitizers).unwrap();
        let _ = fs::remove_file(filepath);

        push_frame(
            &mut file,
            num_time_points,
            0,
            GpsTime::new(22, 205, 10, 55, 30, 0, 0, 0),
            0,
            0,
        );

        push_frame(
            &mut file,
            num_time_points,
            1,
            GpsTime::new(22, 205, 10, 55, 30, 20, 0, 0),
            0,
            0,
        );

        push_frame(
            &mut file,
            num_time_points,
            2,
            GpsTime::new(22, 205, 10, 55, 30, 40, 0, 0),
            0,
            0,
        );

        push_frame(
            &mut file,
            num_time_points,
            0,
            GpsTime::new(22, 205, 10, 55, 30, 0, 0, 0),
            0,
            1,
        );

        // push_frame(
        //     &mut file,
        //     num_time_points,
        //     1,
        //     GpsTime::new(22, 205, 10, 55, 30, 20, 0, 0),
        //     0,
        //     1,
        // );

        push_frame(
            &mut file,
            num_time_points,
            2,
            GpsTime::new(22, 205, 10, 55, 30, 40, 0, 0),
            0,
            1,
        );

        // push_frame(
        //     &mut file,
        //     num_time_points,
        //     0,
        //     GpsTime::new(22, 205, 10, 55, 30, 0, 0, 0),
        //     0,
        //     2,
        // );

        push_frame(
            &mut file,
            num_time_points,
            1,
            GpsTime::new(22, 205, 10, 55, 30, 20, 0, 0),
            0,
            2,
        );

        push_frame(
            &mut file,
            num_time_points,
            2,
            GpsTime::new(22, 205, 10, 55, 30, 40, 0, 0),
            0,
            2,
        );

        let file = file.base.file;

        let frame_start_index = file.dataset("frame_start_index").unwrap();
        assert_eq!(frame_start_index.shape(), vec![num_frames]);

        assert_eq!(
            frame_start_index.read_1d::<usize>().unwrap(),
            arr1(&[0, num_time_points, num_time_points * 2])
        );

        let detector_data = file.dataset("detector_data").unwrap();
        assert_eq!(detector_data.shape(), vec![num_channels, num_measurements]);

        assert_eq!(
            detector_data
                .read_slice::<Intensity, _, _>(s![.., 0..3])
                .unwrap(),
            arr2(&[
                [0, 0, 10],
                [0, 0, 11],
                [0, 0, 0],
                [0, 0, 0],
                [0, 0, 0],
                [0, 0, 0],
                [0, 0, 0],
                [0, 0, 0],
                [1, 0, 10],
                [1, 0, 11],
                [0, 0, 0],
                [0, 0, 0],
                [0, 0, 0],
                [0, 0, 0],
                [0, 0, 0],
                [0, 0, 0],
                [0, 0, 0],
                [0, 0, 0],
                [0, 0, 0],
                [0, 0, 0],
                [0, 0, 0],
                [0, 0, 0],
                [0, 0, 0],
                [0, 0, 0],
            ])
        );

        assert_eq!(
            detector_data
                .read_slice::<Intensity, _, _>(s![.., num_time_points..num_time_points + 3])
                .unwrap(),
            arr2(&[
                [0, 1, 10],
                [0, 1, 11],
                [0, 0, 0],
                [0, 0, 0],
                [0, 0, 0],
                [0, 0, 0],
                [0, 0, 0],
                [0, 0, 0],
                [0, 0, 0],
                [0, 0, 0],
                [0, 0, 0],
                [0, 0, 0],
                [0, 0, 0],
                [0, 0, 0],
                [0, 0, 0],
                [0, 0, 0],
                [2, 1, 10],
                [2, 1, 11],
                [0, 0, 0],
                [0, 0, 0],
                [0, 0, 0],
                [0, 0, 0],
                [0, 0, 0],
                [0, 0, 0],
            ])
        );
    }
}

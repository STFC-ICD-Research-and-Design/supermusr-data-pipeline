use super::base::BaseFile;
use anyhow::{anyhow, Result};
use hdf5::Dataset;
use ndarray::{s, Array, Array0, Array1};
use ndarray_stats::QuantileExt;
use std::path::Path;
use supermusr_common::{channel_index, Intensity, SampleRate, CHANNELS_PER_DIGITIZER};
use supermusr_streaming_types::dat1_digitizer_analog_trace_v1_generated::DigitizerAnalogTraceMessage;

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
            (*data.metadata().timestamp().unwrap()).try_into().unwrap(),
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
            (*data.metadata().timestamp().unwrap()).try_into().unwrap(),
            frame_det_data_start_idx,
        )?;

        self.base.file.flush().unwrap();

        Ok(())
    }
}

#[cfg(test)]
mod tests;

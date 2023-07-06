use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use common::{Channel, DigitizerId, FrameNumber, Intensity};
use hdf5::{Extents, File};
use log::error;
use ndarray::{arr1, s, Array};
use std::path::{Path, PathBuf};
use streaming_types::dat1_digitizer_analog_trace_v1_generated::DigitizerAnalogTraceMessage;

/// Generate the filename for a HDF5 file for data from a trace message.
fn generate_filename(msg: DigitizerAnalogTraceMessage<'_>) -> Result<String> {
    let timestamp: DateTime<Utc> = (*msg
        .metadata()
        .timestamp()
        .ok_or(anyhow!("no timestamp in message"))?)
    .into();

    let digitizer_id = msg.digitizer_id();
    let frame_number = msg.metadata().frame_number();

    Ok(format!(
        "frame_{}_{digitizer_id}_{frame_number}.h5",
        timestamp.to_rfc3339()
    ))
}

/// Creates a HDF5 file containing data from a trace message.
pub(super) fn create(dir: &Path, msg: DigitizerAnalogTraceMessage<'_>) -> Result<PathBuf> {
    let filename = dir.join(generate_filename(msg)?);
    let file = File::create(&filename)?;

    // Obtain the frame/digitizer timetamp
    let frame_timestamp: DateTime<Utc> = (*msg
        .metadata()
        .timestamp()
        .ok_or(anyhow!("no timestamp in message"))?)
    .into();

    // Store the seconds component of the frame timetsamp in the HDF5 file
    let frame_timestamp_seconds = file
        .new_dataset::<u64>()
        .shape(Extents::Scalar)
        .create("metadata/frame_timestamp/seconds")?;
    frame_timestamp_seconds.write_scalar(&frame_timestamp.timestamp())?;

    // Store the nanoseconds component of the frame timetsamp in the HDF5 file
    let frame_timestamp_nanoseconds = file
        .new_dataset::<u32>()
        .shape(Extents::Scalar)
        .create("metadata/frame_timestamp/nanoseconds")?;
    frame_timestamp_nanoseconds.write_scalar(&frame_timestamp.timestamp_subsec_nanos())?;

    // Store the digitizer ID in the HDF5 file
    let digitizer_id = file
        .new_dataset::<DigitizerId>()
        .shape(Extents::Scalar)
        .create("metadata/digitizer_id")?;
    digitizer_id.write_scalar(&msg.digitizer_id())?;

    // Store the frame number in the HDF5 file
    let frame_number = file
        .new_dataset::<FrameNumber>()
        .shape(Extents::Scalar)
        .create("metadata/frame_number")?;
    frame_number.write_scalar(&msg.metadata().frame_number())?;

    // Store the sample rate the HDF5 file
    let sample_rate = file
        .new_dataset::<FrameNumber>()
        .shape(Extents::Scalar)
        .create("metadata/sample_rate")?;
    sample_rate.write_scalar(&msg.sample_rate())?;

    // Obtain the channel data
    let channels = msg
        .channels()
        .ok_or(anyhow!("no channel data in message"))?;
    let num_channels = channels.len();

    // Compute the list of channel numbers and store in the HDF5 file
    let channel_numbers_data: Vec<Channel> = msg
        .channels()
        .ok_or(anyhow!("no channel data in message"))?
        .iter()
        .map(|i| i.channel())
        .collect();
    let channel_numbers = file
        .new_dataset::<Channel>()
        .shape((num_channels,))
        .create("metadata/channel_numbers")?;
    channel_numbers.write(&arr1(&channel_numbers_data))?;

    // Calculate the maximum number of time points across each channel in the message.
    // In theory all channels should always have the same number of time points, this is just here
    // to reduce the chance of having to discard data.
    let num_data_points = channels
        .iter()
        .map(|c| match c.voltage() {
            None => 0,
            Some(intensity) => intensity.len(),
        })
        .max()
        .ok_or(anyhow!("cannot calculate maximum data length"))?;
    // Store the channel data in the HDF5 file
    let channel_data = file
        .new_dataset::<Intensity>()
        .shape((num_channels, num_data_points))
        .create("channel_data")?;
    for (idx, channel) in channels.iter().enumerate() {
        if let Some(intensity) = channel.voltage() {
            let intensity = intensity.iter().collect();
            channel_data.write_slice(&Array::from_vec(intensity), s![idx, ..])?;
        } else {
            error!("Missing intensities for channel {}", channel.channel());
        }
    }

    Ok(filename)
}

#[cfg(test)]
mod test {
    use super::*;
    use common::Intensity;
    use ndarray::{arr1, arr2, s};
    use std::env;
    use streaming_types::{
        dat1_digitizer_analog_trace_v1_generated::{
            finish_digitizer_analog_trace_message_buffer, root_as_digitizer_analog_trace_message,
            ChannelTrace, ChannelTraceArgs, DigitizerAnalogTraceMessage,
            DigitizerAnalogTraceMessageArgs,
        },
        flatbuffers::FlatBufferBuilder,
        frame_metadata_v1_generated::{FrameMetadataV1, FrameMetadataV1Args, GpsTime},
    };

    #[test]
    fn test_basic() {
        let digitizer_id = 2;
        let frame_number = 495;
        let num_time_points = 20;
        let timestamp = GpsTime::new(22, 205, 10, 55, 30, 0, 1, 5);

        let mut fbb = FlatBufferBuilder::new();

        let metadata = FrameMetadataV1Args {
            frame_number,
            period_number: 0,
            protons_per_pulse: 0,
            running: true,
            timestamp: Some(&timestamp),
            veto_flags: 0,
        };
        let metadata = FrameMetadataV1::create(&mut fbb, &metadata);

        let mut voltage: Vec<Intensity> = vec![10; num_time_points];
        voltage[0] = digitizer_id as Intensity;
        voltage[1] = frame_number as Intensity;
        voltage[2] = 0 as Intensity;
        let voltage = Some(fbb.create_vector::<Intensity>(&voltage));
        let channel0 = ChannelTrace::create(
            &mut fbb,
            &ChannelTraceArgs {
                channel: 0,
                voltage,
            },
        );

        let mut voltage: Vec<Intensity> = vec![11; num_time_points];
        voltage[0] = digitizer_id as Intensity;
        voltage[1] = frame_number as Intensity;
        voltage[2] = 1 as Intensity;
        let voltage = Some(fbb.create_vector::<Intensity>(&voltage));
        let channel1 = ChannelTrace::create(
            &mut fbb,
            &ChannelTraceArgs {
                channel: 1,
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

        let filename = create(&env::temp_dir(), message).unwrap();

        let file = File::open(filename).unwrap();

        assert_eq!(
            file.dataset("metadata/frame_timestamp/seconds")
                .unwrap()
                .read_scalar::<u64>()
                .unwrap(),
            1658660130
        );

        assert_eq!(
            file.dataset("metadata/frame_timestamp/nanoseconds")
                .unwrap()
                .read_scalar::<u32>()
                .unwrap(),
            1005
        );

        assert_eq!(
            file.dataset("metadata/digitizer_id")
                .unwrap()
                .read_scalar::<DigitizerId>()
                .unwrap(),
            2
        );

        assert_eq!(
            file.dataset("metadata/frame_number")
                .unwrap()
                .read_scalar::<FrameNumber>()
                .unwrap(),
            495
        );

        let num_channels = 2;
        let channel_numbers = file.dataset("metadata/channel_numbers").unwrap();
        assert_eq!(channel_numbers.shape(), vec![num_channels]);
        assert_eq!(
            channel_numbers.read::<Intensity, _>().unwrap(),
            arr1(&[0, 1])
        );

        let channel_data = file.dataset("channel_data").unwrap();
        assert_eq!(channel_data.shape(), vec![num_channels, num_time_points]);
        assert_eq!(
            channel_data
                .read_slice::<Intensity, _, _>(s![.., 0..3])
                .unwrap(),
            arr2(&[[2, 495, 0], [2, 495, 1],])
        );
    }
}

use super::*;
use ndarray::arr2;
use std::fs;
use supermusr_streaming_types::frame_metadata_v1_generated::GpsTime;

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

use chrono::{DateTime, Utc};
use hdf5::{Dataset, File};
use ndarray::{s, Array};
use std::path::Path;
use supermusr_common::FrameNumber;

pub(super) struct BaseFile {
    pub(super) file: File,

    pub(super) frame_timestamp_seconds: Dataset,
    pub(super) frame_timestamp_nanoseconds: Dataset,

    pub(super) frame_number: Dataset,

    pub(super) frame_start_index: Dataset,

    /// Number of the next "new" frame
    next_frame_number: FrameNumber,
}

impl BaseFile {
    pub(super) fn create(filename: &Path) -> anyhow::Result<Self> {
        let file = File::create(filename)?;

        let frame_timestamp_seconds = file
            .new_dataset::<u64>()
            .shape((0..,))
            .create("frame_timestamp/seconds")?;

        let frame_timestamp_nanoseconds = file
            .new_dataset::<u32>()
            .shape((0..,))
            .create("frame_timestamp/nanoseconds")?;

        let frame_number = file
            .new_dataset::<FrameNumber>()
            .shape((0..,))
            .create("frame_number")?;

        let frame_start_index = file
            .new_dataset::<u32>()
            .shape((0..,))
            .create("frame_start_index")?;

        Ok(BaseFile {
            file,
            frame_timestamp_seconds,
            frame_timestamp_nanoseconds,
            frame_number,
            frame_start_index,
            next_frame_number: 0,
        })
    }

    pub(super) fn find_frame_metadata_index(
        &self,
        frame_number: FrameNumber,
        timestamp: DateTime<Utc>,
    ) -> Option<usize> {
        let frame_number_index = self
            .frame_number
            .read_1d::<FrameNumber>()
            .expect("frame number dataset should be accessible")
            .iter()
            .position(|i| *i == frame_number);

        match frame_number_index {
            None => None,
            Some(frame_number_index) => {
                let timestamp_seconds: ndarray::Array0<u64> = self
                    .frame_timestamp_seconds
                    .read_slice::<u64, _, _>(s![frame_number_index])
                    .expect("frame timestamp seconds dataset should be accessible");
                let timestamp_seconds = timestamp_seconds
                    .first()
                    .expect("filtered timestamp array should not be empty");
                if *timestamp_seconds != timestamp.timestamp() as u64 {
                    panic!("frame number does not match timestamp seconds (this should not happen, or the frame number has reset while this program has been running)");
                }

                let timestamp_nanoseconds: ndarray::Array0<u64> = self
                    .frame_timestamp_nanoseconds
                    .read_slice::<u64, _, _>(s![frame_number_index])
                    .expect("frame timestamp seconds dataset should be accessible");
                let timestamp_nanoseconds = timestamp_nanoseconds
                    .first()
                    .expect("filtered timestamp array should not be empty");
                if *timestamp_nanoseconds != timestamp.timestamp_subsec_nanos() as u64 {
                    panic!("frame number does not match timestamp nanoseconds (this should not happen, or the frame number has reset while this program has been running)");
                }
                Some(frame_number_index)
            }
        }
    }

    pub(super) fn new_frame(
        &mut self,
        frame_number: FrameNumber,
        frame_time: DateTime<Utc>,
        frame_start: usize,
    ) -> anyhow::Result<()> {
        if frame_number < self.next_frame_number {
            return Ok(());
        }
        self.next_frame_number = frame_number + 1;

        let num_frames = self.frame_timestamp_seconds.shape()[0];

        // Record frame timestamp
        let seconds = Array::from_elem((1,), frame_time.timestamp());
        let nanoseconds = Array::from_elem((1,), frame_time.timestamp_subsec_nanos());

        self.frame_timestamp_seconds.resize((num_frames + 1,))?;
        self.frame_timestamp_nanoseconds.resize((num_frames + 1,))?;

        self.frame_timestamp_seconds
            .write_slice(&seconds, s![num_frames..num_frames + 1])?;
        self.frame_timestamp_nanoseconds
            .write_slice(&nanoseconds, s![num_frames..num_frames + 1])?;

        // Record frame number of new frame
        self.frame_number.resize((num_frames + 1,))?;

        let frame_number = Array::from_elem((1,), frame_number);
        self.frame_number
            .write_slice(&frame_number, s![num_frames..num_frames + 1])?;

        // Record start of frame index for new frame
        self.frame_start_index.resize((num_frames + 1,))?;

        let frame_start = Array::from_elem((1,), frame_start);
        self.frame_start_index
            .write_slice(&frame_start, s![num_frames..num_frames + 1])?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use std::{env, fs, path::PathBuf};

    fn create_test_filename(name: &str) -> PathBuf {
        let mut path = env::temp_dir();
        path.push(format!("{name}.h5"));
        path
    }

    #[test]
    fn test_basic() {
        let filepath = create_test_filename("basefile_test_basic");
        let mut file = BaseFile::create(&filepath).unwrap();
        let _ = fs::remove_file(filepath);

        file.new_frame(
            10,
            NaiveDate::from_ymd_opt(2022, 7, 4)
                .unwrap()
                .and_hms_nano_opt(10, 55, 30, 440)
                .unwrap()
                .and_local_timezone(Utc)
                .unwrap(),
            0,
        )
        .unwrap();

        file.new_frame(
            11,
            NaiveDate::from_ymd_opt(2022, 7, 4)
                .unwrap()
                .and_hms_nano_opt(10, 55, 30, 460)
                .unwrap()
                .and_local_timezone(Utc)
                .unwrap(),
            2,
        )
        .unwrap();

        file.new_frame(
            12,
            NaiveDate::from_ymd_opt(2022, 7, 4)
                .unwrap()
                .and_hms_nano_opt(10, 55, 30, 480)
                .unwrap()
                .and_local_timezone(Utc)
                .unwrap(),
            4,
        )
        .unwrap();

        let file = file.file;

        let timestamp_seconds = file.dataset("frame_timestamp/seconds").unwrap();
        assert_eq!(timestamp_seconds.shape(), vec![3]);
        assert_eq!(
            timestamp_seconds.read_slice::<u32, _, _>(s![..]).unwrap(),
            Array::from_vec(vec![1656932130; 3])
        );

        let timestamp_nanoseconds = file.dataset("frame_timestamp/nanoseconds").unwrap();
        assert_eq!(timestamp_nanoseconds.shape(), vec![3]);
        assert_eq!(
            timestamp_nanoseconds
                .read_slice::<u32, _, _>(s![..])
                .unwrap(),
            Array::from_vec(vec![440, 460, 480])
        );

        let frame_number = file.dataset("frame_number").unwrap();
        assert_eq!(frame_number.shape(), vec![3]);
        assert_eq!(
            frame_number.read_slice::<u32, _, _>(s![..]).unwrap(),
            Array::from_vec(vec![10, 11, 12])
        );

        let frame_start_index = file.dataset("frame_start_index").unwrap();
        assert_eq!(frame_start_index.shape(), vec![3]);
        assert_eq!(
            frame_start_index.read_slice::<u32, _, _>(s![..]).unwrap(),
            Array::from_vec(vec![0, 2, 4])
        );
    }

    #[test]
    fn test_multiple() {
        let filepath = create_test_filename("basefile_test_multiple");
        let mut file = BaseFile::create(&filepath).unwrap();
        let _ = fs::remove_file(filepath);

        file.new_frame(
            10,
            NaiveDate::from_ymd_opt(2022, 7, 4)
                .unwrap()
                .and_hms_nano_opt(10, 55, 30, 440)
                .unwrap()
                .and_local_timezone(Utc)
                .unwrap(),
            0,
        )
        .unwrap();

        file.new_frame(
            11,
            NaiveDate::from_ymd_opt(2022, 7, 4)
                .unwrap()
                .and_hms_nano_opt(10, 55, 30, 460)
                .unwrap()
                .and_local_timezone(Utc)
                .unwrap(),
            2,
        )
        .unwrap();

        file.new_frame(
            11,
            NaiveDate::from_ymd_opt(2022, 7, 4)
                .unwrap()
                .and_hms_nano_opt(10, 55, 30, 460)
                .unwrap()
                .and_local_timezone(Utc)
                .unwrap(),
            2,
        )
        .unwrap();

        file.new_frame(
            12,
            NaiveDate::from_ymd_opt(2022, 7, 4)
                .unwrap()
                .and_hms_nano_opt(10, 55, 30, 480)
                .unwrap()
                .and_local_timezone(Utc)
                .unwrap(),
            4,
        )
        .unwrap();

        let file = file.file;

        let frame_number = file.dataset("frame_number").unwrap();
        assert_eq!(frame_number.shape(), vec![3]);
        assert_eq!(
            frame_number.read_slice::<u32, _, _>(s![..]).unwrap(),
            Array::from_vec(vec![10, 11, 12])
        );

        let frame_start_index = file.dataset("frame_start_index").unwrap();
        assert_eq!(frame_start_index.shape(), vec![3]);
        assert_eq!(
            frame_start_index.read_slice::<u32, _, _>(s![..]).unwrap(),
            Array::from_vec(vec![0, 2, 4])
        );
    }

    #[test]
    fn test_multiple_missing_frames() {
        let filepath = create_test_filename("basefile_test_multiple_missing_frames");
        let mut file = BaseFile::create(&filepath).unwrap();
        let _ = fs::remove_file(filepath);

        file.new_frame(
            10,
            NaiveDate::from_ymd_opt(2022, 7, 4)
                .unwrap()
                .and_hms_nano_opt(10, 55, 30, 440)
                .unwrap()
                .and_local_timezone(Utc)
                .unwrap(),
            0,
        )
        .unwrap();

        file.new_frame(
            12,
            NaiveDate::from_ymd_opt(2022, 7, 4)
                .unwrap()
                .and_hms_nano_opt(10, 55, 30, 460)
                .unwrap()
                .and_local_timezone(Utc)
                .unwrap(),
            2,
        )
        .unwrap();

        file.new_frame(
            12,
            NaiveDate::from_ymd_opt(2022, 7, 4)
                .unwrap()
                .and_hms_nano_opt(10, 55, 30, 460)
                .unwrap()
                .and_local_timezone(Utc)
                .unwrap(),
            2,
        )
        .unwrap();

        file.new_frame(
            13,
            NaiveDate::from_ymd_opt(2022, 7, 4)
                .unwrap()
                .and_hms_nano_opt(10, 55, 30, 480)
                .unwrap()
                .and_local_timezone(Utc)
                .unwrap(),
            4,
        )
        .unwrap();

        let file = file.file;

        let frame_number = file.dataset("frame_number").unwrap();
        assert_eq!(frame_number.shape(), vec![3]);
        assert_eq!(
            frame_number.read_1d::<u32>().unwrap(),
            Array::from_vec(vec![10, 12, 13])
        );

        let frame_start_index = file.dataset("frame_start_index").unwrap();
        assert_eq!(frame_start_index.shape(), vec![3]);
        assert_eq!(
            frame_start_index.read_1d::<u32>().unwrap(),
            Array::from_vec(vec![0, 2, 4])
        );
    }

    #[test]
    fn test_find_frame_metadata_index() {
        let filepath = create_test_filename("basefile_test_find_frame_metadata_index");
        let mut file = BaseFile::create(&filepath).unwrap();
        let _ = fs::remove_file(filepath);

        file.new_frame(
            10,
            NaiveDate::from_ymd_opt(2022, 7, 4)
                .unwrap()
                .and_hms_nano_opt(10, 55, 30, 440)
                .unwrap()
                .and_local_timezone(Utc)
                .unwrap(),
            0,
        )
        .unwrap();

        file.new_frame(
            11,
            NaiveDate::from_ymd_opt(2022, 7, 4)
                .unwrap()
                .and_hms_nano_opt(10, 55, 30, 460)
                .unwrap()
                .and_local_timezone(Utc)
                .unwrap(),
            2,
        )
        .unwrap();

        file.new_frame(
            12,
            NaiveDate::from_ymd_opt(2022, 7, 4)
                .unwrap()
                .and_hms_nano_opt(10, 55, 30, 480)
                .unwrap()
                .and_local_timezone(Utc)
                .unwrap(),
            4,
        )
        .unwrap();

        // Frame found
        assert_eq!(
            Some(1),
            file.find_frame_metadata_index(
                11,
                NaiveDate::from_ymd_opt(2022, 7, 4)
                    .unwrap()
                    .and_hms_nano_opt(10, 55, 30, 460)
                    .unwrap()
                    .and_local_timezone(Utc)
                    .unwrap(),
            )
        );

        // Frame not found
        assert_eq!(
            None,
            file.find_frame_metadata_index(
                9,
                NaiveDate::from_ymd_opt(2022, 7, 4)
                    .unwrap()
                    .and_hms_nano_opt(10, 55, 30, 380)
                    .unwrap()
                    .and_local_timezone(Utc)
                    .unwrap(),
            )
        );

        // Partial metadata match
        assert!(std::panic::catch_unwind(|| {
            file.find_frame_metadata_index(
                11,
                NaiveDate::from_ymd_opt(2022, 7, 4)
                    .unwrap()
                    .and_hms_nano_opt(10, 55, 30, 360)
                    .unwrap()
                    .and_local_timezone(Utc)
                    .unwrap(),
            )
        })
        .is_err());
    }
}

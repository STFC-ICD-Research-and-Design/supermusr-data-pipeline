use anyhow::Result;
use chrono::{DateTime, Utc};
use common::FrameNumber;
use hdf5::{Dataset, File};
use ndarray::{s, Array};
use std::path::Path;

pub(super) struct BaseFile {
    pub(super) file: File,

    pub(super) frame_timestamp_seconds: Dataset,
    pub(super) frame_timestamp_nanoseconds: Dataset,

    pub(super) frame_start_index: Dataset,

    /// Number of the next "new" frame
    next_frame_number: FrameNumber,
}

impl BaseFile {
    pub(super) fn create(filename: &Path) -> Result<Self> {
        let file = File::create(filename)?;

        let frame_timestamp_seconds = file
            .new_dataset::<u64>()
            .shape((0..,))
            .create("frame_timestamp/seconds")?;

        let frame_timestamp_nanoseconds = file
            .new_dataset::<u32>()
            .shape((0..,))
            .create("frame_timestamp/nanoseconds")?;

        let frame_start_index = file
            .new_dataset::<u32>()
            .shape((0..,))
            .create("frame_start_index")?;

        Ok(BaseFile {
            file,
            frame_timestamp_seconds,
            frame_timestamp_nanoseconds,
            frame_start_index,
            next_frame_number: 0,
        })
    }

    pub(super) fn new_frame(
        &mut self,
        frame_number: FrameNumber,
        frame_time: DateTime<Utc>,
        frame_start: usize,
    ) -> Result<()> {
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
        path.push(format!("{}.h5", name));
        path
    }

    #[test]
    fn test_basic() {
        let filepath = create_test_filename("basefile_test_basic");
        let mut file = BaseFile::create(&filepath).unwrap();
        let _ = fs::remove_file(filepath);

        file.new_frame(
            0,
            NaiveDate::from_ymd(2022, 7, 4)
                .and_hms_nano(10, 55, 30, 440)
                .and_local_timezone(Utc)
                .unwrap(),
            0,
        )
        .unwrap();

        file.new_frame(
            1,
            NaiveDate::from_ymd(2022, 7, 4)
                .and_hms_nano(10, 55, 30, 460)
                .and_local_timezone(Utc)
                .unwrap(),
            2,
        )
        .unwrap();

        file.new_frame(
            2,
            NaiveDate::from_ymd(2022, 7, 4)
                .and_hms_nano(10, 55, 30, 480)
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
            0,
            NaiveDate::from_ymd(2022, 7, 4)
                .and_hms_nano(10, 55, 30, 440)
                .and_local_timezone(Utc)
                .unwrap(),
            0,
        )
        .unwrap();

        file.new_frame(
            1,
            NaiveDate::from_ymd(2022, 7, 4)
                .and_hms_nano(10, 55, 30, 460)
                .and_local_timezone(Utc)
                .unwrap(),
            2,
        )
        .unwrap();

        file.new_frame(
            1,
            NaiveDate::from_ymd(2022, 7, 4)
                .and_hms_nano(10, 55, 30, 460)
                .and_local_timezone(Utc)
                .unwrap(),
            2,
        )
        .unwrap();

        file.new_frame(
            2,
            NaiveDate::from_ymd(2022, 7, 4)
                .and_hms_nano(10, 55, 30, 480)
                .and_local_timezone(Utc)
                .unwrap(),
            4,
        )
        .unwrap();

        let file = file.file;

        let frame_start_index = file.dataset("frame_start_index").unwrap();
        assert_eq!(frame_start_index.shape(), vec![3]);
        assert_eq!(
            frame_start_index.read_slice::<u32, _, _>(s![..]).unwrap(),
            Array::from_vec(vec![0, 2, 4])
        );
    }
}

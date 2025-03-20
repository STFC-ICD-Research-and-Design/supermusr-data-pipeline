use std::path::{Path, PathBuf};
use tokio::time::Interval;

/// Creates the patterns for
fn get_path_glob_pattern(path: &Path) -> Result<String, &Path> {
    path.as_os_str()
        .to_str()
        .map(|path| format!("{path}/*.nxs"))
        .ok_or(path)
}

#[derive(Default, Debug)]
pub(crate) struct NexusSettings {
    local_path: PathBuf,
    local_path_completed: PathBuf,
    pub(crate) framelist_chunk_size: usize,
    pub(crate) eventlist_chunk_size: usize,
    pub(crate) periodlist_chunk_size: usize,
    pub(crate) runloglist_chunk_size: usize,
    pub(crate) seloglist_chunk_size: usize,
    pub(crate) alarmlist_chunk_size: usize,
    archive_path: Option<PathBuf>,
    archive_flush_interval_sec: u64,
}

impl NexusSettings {
    pub(crate) fn new(
        local_path: &Path,
        framelist_chunk_size: usize,
        eventlist_chunk_size: usize,
        archive_path: Option<&Path>,
        archive_flush_interval_sec: u64,
    ) -> Self {
        let local_path = local_path.to_path_buf();
        let mut local_path_completed = local_path.to_path_buf();
        local_path_completed.push("completed");
        Self {
            local_path,
            local_path_completed,
            framelist_chunk_size,
            eventlist_chunk_size,
            periodlist_chunk_size: 8,
            runloglist_chunk_size: 64,
            seloglist_chunk_size: 1024,
            alarmlist_chunk_size: 32,
            archive_path: archive_path.map(Path::to_owned),
            archive_flush_interval_sec,
        }
    }

    pub(crate) fn get_local_path(&self) -> &Path {
        &self.local_path
    }

    pub(crate) fn get_local_completed_path(&self) -> &Path {
        &self.local_path_completed
    }

    pub(crate) fn get_archive_path(&self) -> Option<&Path> {
        self.archive_path.as_deref()
    }

    pub(crate) fn get_local_temp_glob_pattern(&self) -> Result<String, &Path> {
        get_path_glob_pattern(&self.local_path)
    }

    pub(crate) fn get_local_completed_glob_pattern(&self) -> Result<String, &Path> {
        get_path_glob_pattern(&self.local_path_completed)
    }

    pub(crate) fn get_archive_flush_interval(&self) -> Interval {
        tokio::time::interval(tokio::time::Duration::from_secs(
            self.archive_flush_interval_sec,
        ))
    }
}

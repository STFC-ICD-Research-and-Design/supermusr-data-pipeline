//! This module defines types used to configure `NexusEngine`
//! and the modules of `nexus_structure`.
use std::path::{Path, PathBuf};
use tokio::time::Interval;

/// Creates the glob patterns for matching all NeXus files in a directory.
/// # Parameters
/// - path: The path of the directory to match in.
/// # Return
/// The glob string, i.e. of the form "\[completed directory\]/*.nxs".
/// # Error Modes
/// - Emits `Err(path)` if it cannot be converted to a string.
fn get_path_glob_pattern(path: &Path) -> Result<String, &Path> {
    path.as_os_str()
        .to_str()
        .map(|path| format!("{path}/*.nxs"))
        .ok_or(path)
}

/// Type alias to tie the `RunLog`'s chunk size to an associated type [crate::nexus::NexusSchematic::Settings].
pub(crate) type RunLogChunkSize = usize;

/// Type alias to tie the `SELog`'s chunk size to an associated type [crate::nexus::NexusSchematic::Settings].
pub(crate) type SELogChunkSize = usize;

/// Type alias to tie the `AlarmLog`'s chunk size to an associated type [crate::nexus::NexusSchematic::Settings].
pub(crate) type AlarmChunkSize = usize;

/// Type alias to tie the chunk sizes of fields which increment by frame, to an associated type [crate::nexus::NexusSchematic::Settings].
pub(crate) type FrameChunkSize = usize;

/// Type alias to tie the chunk sizes of fields which increment by muon event, to an associated type [crate::nexus::NexusSchematic::Settings].
pub(crate) type EventChunkSize = usize;

/// Type alias to tie the `Period`'s chunk size to an associated type [crate::nexus::NexusSchematic::Settings].
pub(crate) type PeriodChunkSize = usize;

/// Contains chunk sizes to use in constructing one-dimentional hdf5 datasets.
#[derive(Default, Debug)]
pub(crate) struct ChunkSizeSettings {
    /// Chunk size for fields in `EventData` which increment each frame.
    pub(crate) frame: FrameChunkSize,
    /// Chunk size for fields in `EventData` which increment each muon event.
    pub(crate) event: EventChunkSize,
    /// Chunk size for fields which increment each new period.
    pub(crate) period: PeriodChunkSize,
    /// Chunk size for runlog fields.
    pub(crate) runlog: RunLogChunkSize,
    /// Chunk size for selog fields.
    pub(crate) selog: SELogChunkSize,
    /// Chunk size for alarm fields.
    pub(crate) alarm: AlarmChunkSize,
}

impl ChunkSizeSettings {
    /// Creates a new [ChunkSizeSettings]. The caller specifies the frame and event chunk size,
    /// and the others are currently hard-coded.
    pub(crate) fn new(frame: usize, event: usize) -> Self {
        Self {
            frame,
            event,
            period: 8,
            runlog: 64,
            selog: 1024,
            alarm: 32,
        }
    }
}

/// Contains all settings which persist across all runs.
#[derive(Default, Debug)]
pub(crate) struct NexusSettings {
    /// Path where NeXus files reside when they are receiving data.
    local_path: PathBuf,
    /// Path to directory which NeXus files are moved immeditately upon completion.
    local_path_completed: PathBuf,
    /// The hdf5 chunk sizes to use.
    chunk_sizes: ChunkSizeSettings,
    /// Optional path to directory which completed NeXus files are moved periodically. This can be a remote directory.
    archive_path: Option<PathBuf>,
    /// Interval (in seconds) in which the NeXus files in `local_path_completed` are moved to `archive_path` (if set).
    archive_flush_interval_sec: u64,
}

impl NexusSettings {
    /// Creates a new [NexusSettings].
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
            chunk_sizes: ChunkSizeSettings::new(framelist_chunk_size, eventlist_chunk_size),
            archive_path: archive_path.map(Path::to_owned),
            archive_flush_interval_sec,
        }
    }

    /// Return the path to the local temporary directory.
    pub(crate) fn get_local_path(&self) -> &Path {
        &self.local_path
    }

    /// Return the path to the local "completed" directory.
    pub(crate) fn get_local_completed_path(&self) -> &Path {
        &self.local_path_completed
    }

    /// Return the optional archive path.
    pub(crate) fn get_archive_path(&self) -> Option<&Path> {
        self.archive_path.as_deref()
    }

    /// Creates a glob pattern for matching with files in the local "temporary" directory.
    /// # Return
    /// The glob string, i.e. of the form "\[local directory\]/*.nxs".
    /// # Error Modes
    /// - Propagates errors from [get_path_glob_pattern] if they occur.
    pub(crate) fn get_local_temp_glob_pattern(&self) -> Result<String, &Path> {
        get_path_glob_pattern(&self.local_path)
    }

    /// Creates a glob pattern for matching with files in the local "completed" directory.
    /// # Return
    /// The glob string, i.e. of the form "\[completed directory\]/*.nxs".
    /// # Error Modes
    /// - Propagates errors from [get_path_glob_pattern] if they occur.
    pub(crate) fn get_local_completed_glob_pattern(&self) -> Result<String, &Path> {
        get_path_glob_pattern(&self.local_path_completed)
    }

    /// Creates an [Interval] object which ticks with the period specified in [archive_flush_interval_sec].
    /// 
    /// [archive_flush_interval_sec]: NexusSettings::archive_flush_interval_sec
    pub(crate) fn get_archive_flush_interval(&self) -> Interval {
        tokio::time::interval(tokio::time::Duration::from_secs(
            self.archive_flush_interval_sec,
        ))
    }

    /// Returns the sizes of the hdf5 chunks to use.
    pub(crate) fn get_chunk_sizes(&self) -> &ChunkSizeSettings {
        &self.chunk_sizes
    }
}

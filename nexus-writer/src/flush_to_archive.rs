//! Defines async function which moves completed NeXus files to remote storage.
use crate::{
    error::{ErrorCodeLocation, NexusWriterError, NexusWriterResult},
    NexusSettings,
};
use std::path::{Path, PathBuf};
use tokio::{
    signal::unix::{signal, SignalKind},
    task::JoinHandle,
    time::Interval,
};
use tracing::{debug, info, warn};

/// Moves a single file to the archive
/// # Parameters
/// - from_path: The file's existing path.
/// - to_path: The file's target path.
/// # Error Modes
/// - Propagates [std::fs::copy] errors if they occur.
/// - Propagates [std::fs::remove_file] errors if they occur.
#[tracing::instrument(skip_all, level = "info", fields(
    from_path = from_path.to_string_lossy().to_string(),
    to_path
))]
fn move_file_to_archive(from_path: &Path, archive_path: &Path) -> NexusWriterResult<()> {
    let mut to_path = archive_path.to_path_buf();
    if let Some(file_name) = from_path.file_name() {
        to_path.push(file_name);
        tracing::Span::current().record("to_path", to_path.to_string_lossy().to_string());
    }

    match std::fs::copy(from_path, to_path) {
        Ok(bytes) => info!("File Move Succesful. {bytes} byte(s) moved."),
        Err(e) => {
            warn!("File Move Error {e}");
            return Err(e.into());
        }
    };
    if let Err(e) = std::fs::remove_file(from_path) {
        warn!("Error removing temporary file: {e}");
        return Err(e.into());
    }
    Ok(())
}

/// Flushes all files in the local completed directory to the archive
/// # Parameters
/// - glob_pattern: A glob pattern which should match NeXus files in the appropriate directory.
/// - archive_path: The archive's path.
/// # Error Modes
/// - Propagates [glob] errors if they occur.
/// - Propagates [move_file_to_archive] errors if they occur.
///
/// [glob]: glob::glob()
#[tracing::instrument(level = "debug", fields(
    glob_pattern = glob_pattern,
    archive_path = archive_path.to_string_lossy().to_string()
))]
pub(crate) async fn flush_to_archive(
    glob_pattern: &str,
    archive_path: &Path,
) -> NexusWriterResult<()> {
    for file_path in glob::glob(glob_pattern)? {
        move_file_to_archive(file_path?.as_path(), archive_path)?;
    }
    Ok(())
}

/// Runs infinitely, and periodically moves any files in the local "completed" directory
/// to the archive, for instance, on a network storage drive.
///
/// Calling this function returns a Future, which should be passed to a async task,
/// as in function [create_archive_flush_task]. The general form of this is:
/// ```rust
/// let join_handle = tokio::spawn(archive_flush_task(...))?;
/// ```
/// # Parameters
/// - glob_pattern: A glob pattern which should match NeXus files in the appropriate directory.
/// - archive_path: The archive's path.
/// - interval: the interval at which the [flush_to_archive] function should be called.
/// # Error Modes
/// - Propagates [tokio::signal] errors if they occur.
/// - Propagates [flush_to_archive] errors if they occur.
#[tracing::instrument(skip_all, level = "info", fields(
    glob_pattern = glob_pattern,
    archive_path = archive_path.to_string_lossy().to_string()
))]
async fn archive_flush_task(
    glob_pattern: String,
    archive_path: PathBuf,
    mut interval: Interval,
) -> NexusWriterResult<()> {
    // Is used to await any sigint signals
    let mut sigint = signal(SignalKind::interrupt())?;

    debug!("Finding files matched to {glob_pattern}");
    loop {
        tokio::select! {
            _ = interval.tick() => flush_to_archive(&glob_pattern, &archive_path).await?,
            _ = sigint.recv() => return Ok(())
        }
    }
}

/// When the user specifies an `archive_path` in [NexusSettings], then a new thread and setup the task.
/// # Parameters
/// - nexus_settings: contains path to `archive_path` if set.
/// # Return
/// If the user specified an archive path, creates the archive flush task
/// and returns the [JoinHandle], otherwise returns [None].
/// # Error Modes
/// - Emits [CannotConvertPath] if [get_local_completed_glob_pattern] fails.
///
/// [CannotConvertPath]: NexusWriterError::CannotConvertPath
/// [get_local_completed_glob_pattern]: NexusSettings::get_local_completed_glob_pattern()
#[tracing::instrument(skip_all, level = "info")]
pub(crate) fn create_archive_flush_task(
    nexus_settings: &NexusSettings,
) -> NexusWriterResult<Option<JoinHandle<NexusWriterResult<()>>>> {
    let local_completed_glob_pattern =
        nexus_settings
            .get_local_completed_glob_pattern()
            .map_err(|path| NexusWriterError::CannotConvertPath {
                path: path.to_path_buf(),
                location: ErrorCodeLocation::FlushToArchive,
            })?;
    Ok(nexus_settings.get_archive_path().map(|archive_path| {
        tokio::spawn(archive_flush_task(
            local_completed_glob_pattern,
            archive_path.to_path_buf(),
            nexus_settings.get_archive_flush_interval(),
        ))
    }))
}

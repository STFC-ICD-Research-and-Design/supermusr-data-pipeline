use crate::{NexusSettings, NexusWriterResult};
use std::path::{Path, PathBuf};
use tokio::{
    signal::unix::{signal, SignalKind},
    task::JoinHandle,
    time::Interval,
};
use tracing::{debug, info, info_span, warn};

#[tracing::instrument(skip_all, level = "debug")]
fn move_file_to_archive(from_path: &Path, archive_path: &Path) -> NexusWriterResult<()> {
    let mut to_path = archive_path.to_path_buf();
    if let Some(file_name) = from_path.file_name() {
        to_path.push(file_name);
    }

    info_span!(
        "Move to Remote Archive",
        from_path = from_path.to_string_lossy().to_string(),
        to_path = to_path.to_string_lossy().to_string()
    )
    .in_scope(|| {
        match std::fs::copy(from_path, to_path) {
            Ok(bytes) => info!("File Move Succesful. {bytes} byte(s) moved."),
            Err(e) => {
                warn!("File Move Error {e}");
                return Err(e);
            }
        };
        if let Err(e) = std::fs::remove_file(from_path) {
            warn!("Error removing temporary file: {e}");
            Err(e)
        } else {
            Ok(())
        }
    })?;
    Ok(())
}

/// If an additional archive location is set by the user,
/// then completed runs placed in the vector `self.run_move_cache`
/// have their nexus files asynchonously moved to that location.
/// Afterwhich the runs are dropped.
#[tracing::instrument(level = "debug", fields(glob_pattern=glob_pattern,archive_path=archive_path.to_string_lossy().to_string()))]
pub(crate) async fn flush_to_archive(
    glob_pattern: &str,
    archive_path: &Path,
) -> NexusWriterResult<()> {
    for file_path in glob::glob(glob_pattern)? {
        move_file_to_archive(file_path?.as_path(), archive_path)?;
    }
    Ok(())
}

#[tracing::instrument(skip_all, level = "info", fields(glob_pattern=glob_pattern,archive_path=archive_path.to_string_lossy().to_string()))]
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

#[tracing::instrument(skip_all, level = "info")]
pub(crate) fn create_archive_flush_task(
    nexus_settings: &NexusSettings,
) -> NexusWriterResult<Option<JoinHandle<NexusWriterResult<()>>>> {
    let local_completed_glob_pattern = nexus_settings.get_local_completed_glob_pattern()?;
    Ok(nexus_settings.get_archive_path().map(|archive_path| {
        tokio::spawn(archive_flush_task(
            local_completed_glob_pattern,
            archive_path.to_path_buf(),
            nexus_settings.get_archive_flush_interval(),
        ))
    }))
}

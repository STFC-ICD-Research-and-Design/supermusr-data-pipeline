use std::{
    fs::File,
    path::{Path, PathBuf},
};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use supermusr_streaming_types::{
    ecs_6s4t_run_stop_generated::RunStop, ecs_pl72_run_start_generated::RunStart,
};

use super::hdf5_file::RunFile;

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub(crate) struct RunStopParameters {
    pub(crate) collect_until: DateTime<Utc>,
    pub(crate) last_modified: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct RunParameters {
    pub(crate) collect_from: DateTime<Utc>,
    pub(crate) run_stop_parameters: Option<RunStopParameters>,
    pub(crate) num_periods: u32,
    pub(crate) run_name: String,
    pub(crate) run_number: u32,
    pub(crate) instrument_name: String,
}

impl RunParameters {
    #[tracing::instrument(skip_all, level = "trace", err(level = "warn"))]
    pub(crate) fn new(data: RunStart<'_>, run_number: u32) -> anyhow::Result<Self> {
        Ok(Self {
            collect_from: DateTime::<Utc>::from_timestamp_millis(data.start_time().try_into()?)
                .ok_or(anyhow::anyhow!(
                    "Cannot create start_time from {0}",
                    &data.start_time()
                ))?,
            run_stop_parameters: None,
            num_periods: data.n_periods(),
            run_name: data
                .run_name()
                .ok_or(anyhow::anyhow!("Run Name not found"))?
                .to_owned(),
            run_number,
            instrument_name: data
                .instrument_name()
                .ok_or(anyhow::anyhow!("Instrument Name not found"))?
                .to_owned(),
        })
    }

    #[tracing::instrument(skip_all, level = "trace", err(level = "warn"))]
    pub(crate) fn set_stop_if_valid(&mut self, data: RunStop<'_>) -> anyhow::Result<()> {
        if self.run_stop_parameters.is_some() {
            Err(anyhow::anyhow!("Stop Command before Start Command"))
        } else {
            let stop_time =
                DateTime::<Utc>::from_timestamp_millis(data.stop_time().try_into()?).ok_or(
                    anyhow::anyhow!("Cannot create end_time from {0}", data.stop_time()),
                )?;
            if self.collect_from < stop_time {
                self.run_stop_parameters = Some(RunStopParameters {
                    collect_until: stop_time,
                    last_modified: Utc::now(),
                });
                Ok(())
            } else {
                Err(anyhow::anyhow!(
                    "Stop Time earlier than current Start Time."
                ))
            }
        }
    }

    /// Returns true if timestamp is strictly after collect_from and,
    /// if run_stop_parameters exist then, if timestamp is strictly
    /// before params.collect_until.
    #[tracing::instrument(skip_all, level = "trace")]
    pub(crate) fn is_message_timestamp_valid(&self, timestamp: &DateTime<Utc>) -> bool {
        if self.collect_from < *timestamp {
            self.run_stop_parameters
                .as_ref()
                .map(|params| *timestamp < params.collect_until)
                .unwrap_or(true)
        } else {
            false
        }
    }

    #[tracing::instrument(skip_all, level = "trace")]
    pub(crate) fn update_last_modified(&mut self) {
        if let Some(params) = &mut self.run_stop_parameters {
            params.last_modified = Utc::now();
        }
    }

    pub(crate) fn get_hdf5_path_buf(path: &Path, run_name: &str) -> PathBuf {
        let mut path = path.to_owned();
        path.push(run_name);
        path.set_extension("nxs");
        path
    }

    pub(crate) fn get_partial_path_buf(path: &Path, run_name: &str) -> PathBuf {
        let mut path = path.to_owned();
        path.push(run_name);
        path.set_extension("partial_run");
        path
    }

    pub(crate) fn save_partial_run(&self, path: &Path) -> anyhow::Result<()> {
        let path_buf = Self::get_partial_path_buf(path, &self.run_name);
        let file = File::create(path_buf.as_path())?;
        serde_json::to_writer(file, &self)?;
        Ok(())
    }

    pub(crate) fn detect_partial_run(path: &Path, filename: &str) -> anyhow::Result<Option<Self>> {
        let run_file = RunFile::open_runfile(path, filename)?;
        let path_buf = Self::get_partial_path_buf(path, filename);
        if path_buf.as_path().exists() {
            let file = File::open(path_buf.as_path())?;
            let run_parameters: RunParameters = serde_json::from_reader(file)?;
            std::fs::remove_file(path_buf.as_path())?;
            Ok(Some(run_parameters))
        } else {
            Ok(None)
        }
    }
}

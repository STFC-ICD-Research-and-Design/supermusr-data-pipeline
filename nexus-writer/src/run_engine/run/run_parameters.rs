use crate::{
    error::{ErrorCodeLocation, FlatBufferMissingError, NexusWriterError, NexusWriterResult},
    run_engine::NexusDateTime,
};
use chrono::Utc;
use std::path::{Path, PathBuf};
use supermusr_streaming_types::{
    ecs_6s4t_run_stop_generated::RunStop, ecs_pl72_run_start_generated::RunStart,
};

#[derive(Clone, Default, Debug)]
pub(crate) struct NexusConfiguration {
    /// Data pipeline configuration to be written to the `/raw_data_1/program_name/configuration`
    /// attribute of the NeXus file.
    pub(crate) configuration: String,
}

impl NexusConfiguration {
    pub(crate) fn new(configuration: Option<String>) -> Self {
        Self {
            configuration: configuration.unwrap_or_default(),
        }
    }
}

#[derive(Default, Debug, Clone)]
pub(crate) struct RunStopParameters {
    pub(crate) collect_until: NexusDateTime,
    pub(crate) last_modified: NexusDateTime,
}

#[derive(Debug, Clone)]
pub(crate) struct RunParameters {
    pub(crate) collect_from: NexusDateTime,
    pub(crate) run_stop_parameters: Option<RunStopParameters>,
    pub(crate) run_name: String,
    pub(crate) periods: Vec<u64>,
}

impl RunParameters {
    #[tracing::instrument(skip_all, level = "trace", err(level = "warn"))]
    pub(crate) fn new(data: RunStart<'_>) -> NexusWriterResult<Self> {
        let run_name = data
            .run_name()
            .ok_or(NexusWriterError::FlatBufferMissing(
                FlatBufferMissingError::RunName,
                ErrorCodeLocation::NewRunParamemters,
            ))?
            .to_owned();
        Ok(Self {
            collect_from: NexusDateTime::from_timestamp_millis(data.start_time().try_into()?)
                .ok_or(NexusWriterError::IntOutOfRangeForDateTime {
                    int: data.start_time(),
                    location: ErrorCodeLocation::NewRunParamemters,
                })?,
            run_stop_parameters: None,
            run_name,
            periods: Default::default(),
        })
    }

    #[tracing::instrument(skip_all, level = "trace", err(level = "warn"))]
    pub(crate) fn set_stop_if_valid(&mut self, data: &RunStop<'_>) -> NexusWriterResult<()> {
        if self.run_stop_parameters.is_some() {
            Err(NexusWriterError::StopCommandBeforeStartCommand(
                ErrorCodeLocation::SetStopIfValid,
            ))
        } else {
            let stop_time = NexusDateTime::from_timestamp_millis(data.stop_time().try_into()?)
                .ok_or(NexusWriterError::IntOutOfRangeForDateTime {
                    int: data.stop_time(),
                    location: ErrorCodeLocation::SetStopIfValid,
                })?;
            if self.collect_from < stop_time {
                self.run_stop_parameters = Some(RunStopParameters {
                    collect_until: stop_time,
                    last_modified: Utc::now(),
                });
                Ok(())
            } else {
                Err(NexusWriterError::StopTimeEarlierThanStartTime {
                    start: self.collect_from,
                    stop: stop_time,
                    location: ErrorCodeLocation::SetStopIfValid,
                })
            }
        }
    }

    #[tracing::instrument(skip_all, level = "trace", err(level = "warn"))]
    pub(crate) fn set_aborted_run(&mut self, stop_time: u64) -> NexusWriterResult<()> {
        let collect_until = NexusDateTime::from_timestamp_millis(stop_time.try_into()?).ok_or(
            NexusWriterError::IntOutOfRangeForDateTime {
                int: stop_time,
                location: ErrorCodeLocation::SetAbortedRun,
            },
        )?;
        if self.run_stop_parameters.is_some() {
            return Err(NexusWriterError::RunStopAlreadySet(
                ErrorCodeLocation::SetAbortedRun,
            ));
        }
        {
            self.run_stop_parameters = Some(RunStopParameters {
                collect_until,
                last_modified: Utc::now(),
            });
        }
        Ok(())
    }

    /// Returns true if timestamp is strictly after collect_from and,
    /// if run_stop_parameters exist then, if timestamp is strictly
    /// before params.collect_until.
    #[tracing::instrument(skip_all, level = "trace")]
    pub(crate) fn is_message_timestamp_within_range(&self, timestamp: &NexusDateTime) -> bool {
        if self.collect_from < *timestamp {
            self.is_message_timestamp_not_after_end(timestamp)
        } else {
            false
        }
    }

    /// if run_stop_parameters exist then, return true if timestamp is
    /// strictly before params.collect_until, otherwise returns true.
    #[tracing::instrument(skip_all, level = "trace")]
    pub(crate) fn is_message_timestamp_not_after_end(&self, timestamp: &NexusDateTime) -> bool {
        self.run_stop_parameters
            .as_ref()
            .map(|params| *timestamp < params.collect_until)
            .unwrap_or(true)
    }

    #[tracing::instrument(skip_all, level = "trace")]
    pub(crate) fn update_last_modified(&mut self) {
        if let Some(params) = &mut self.run_stop_parameters {
            params.last_modified = Utc::now();
        }
    }

    pub(crate) fn get_hdf5_filename(path: &Path, run_name: &str) -> PathBuf {
        let mut path = path.to_owned();
        path.push(run_name);
        path.set_extension("nxs");
        path
    }
}

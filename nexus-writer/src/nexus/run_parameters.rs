use chrono::{DateTime, Utc};
use supermusr_streaming_types::{
    ecs_6s4t_run_stop_generated::RunStop, ecs_pl72_run_start_generated::RunStart,
};

const TRACING_CLASS: &str = "NexusWriter::RunParameters";

#[derive(Default, Debug)]
pub(crate) struct RunStopParameters {
    pub(crate) collect_until: DateTime<Utc>,
    pub(crate) last_modified: DateTime<Utc>,
}

#[derive(Debug)]
pub(crate) struct RunParameters {
    pub(crate) collect_from: DateTime<Utc>,
    pub(crate) run_stop_parameters: Option<RunStopParameters>,
    pub(crate) num_periods: u32,
    pub(crate) run_name: String,
    pub(crate) run_number: u32,
    pub(crate) instrument_name: String,
}

impl RunParameters {
    #[tracing::instrument(fields(class = TRACING_CLASS))]
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

    #[tracing::instrument(fields(class = TRACING_CLASS))]
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
    #[tracing::instrument(fields(class = TRACING_CLASS))]
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

    #[tracing::instrument(fields(class = TRACING_CLASS))]
    pub(crate) fn update_last_modified(&mut self) {
        if let Some(params) = &mut self.run_stop_parameters {
            params.last_modified = Utc::now();
        }
    }
}

use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use supermusr_streaming_types::{
    ecs_6s4t_run_stop_generated::RunStop, ecs_pl72_run_start_generated::RunStart,
};

#[derive(Default, Debug)]
pub(crate) struct RunStopParameters {
    pub(crate) collect_until: u64,
    pub(crate) time_completed: DateTime<Utc>,
}

#[derive(Debug)]
pub(crate) struct RunParameters {
    pub(crate) collect_from: u64,
    pub(crate) run_stop_parameters: Option<RunStopParameters>,
    pub(crate) num_periods: u32,
    pub(crate) run_name: String,
    pub(crate) run_number: u32,
    pub(crate) instrument_name: String,
}

impl RunParameters {
    pub(crate) fn new(data: RunStart<'_>, run_number: u32) -> Result<Self> {
        Ok(Self {
            collect_from: data.start_time(),
            run_stop_parameters: None,
            num_periods: data.n_periods(),
            run_name: data
                .run_name()
                .ok_or(anyhow!("Run Name not found"))?
                .to_owned(),
            run_number,
            instrument_name: data
                .instrument_name()
                .ok_or(anyhow!("Instrument Name not found"))?
                .to_owned(),
        })
    }

    pub(crate) fn set_stop_if_valid(&mut self, data: RunStop<'_>) -> Result<()> {
        if self.run_stop_parameters.is_some() {
            Err(anyhow!("Stop Command before Start Command"))
        } else if self.collect_from < data.stop_time() {
            self.run_stop_parameters = Some(RunStopParameters {
                collect_until: data.stop_time(),
                time_completed: Utc::now(),
            });
            Ok(())
        } else {
            Err(anyhow!("Stop Time earlier than current Start Time."))
        }
    }

    pub(crate) fn is_message_timestamp_valid(&self, timestamp: &DateTime<Utc>) -> Result<bool> {
        let millis: u64 = timestamp.timestamp_millis().try_into()?;
        Ok(if self.collect_from < millis {
            self.run_stop_parameters
                .as_ref()
                .map(|params| millis < params.collect_until)
                .unwrap_or(true)
        } else {
            false
        })
    }

    pub(crate) fn update_time_completed(&mut self) {
        if let Some(params) = &mut self.run_stop_parameters {
            params.time_completed = Utc::now();
        }
    }
}

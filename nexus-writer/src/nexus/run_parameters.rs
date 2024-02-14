use crate::nexus::hdf5_writer::{
    add_new_field_to, add_new_group_to, add_new_slice_field_to, add_new_string_field_to, Hdf5Writer,
};
use crate::nexus::nexus_class as NX;
use anyhow::{anyhow, Result};
use chrono::{DateTime, Duration, Utc};
use hdf5::Group;
use supermusr_streaming_types::{
    ecs_6s4t_run_stop_generated::RunStop, ecs_pl72_run_start_generated::RunStart,
};
pub(crate) const DATETIME_FORMAT: &str = "%Y-%m-%dT%H:%M:%S%z";

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
        let milis: u64 = timestamp.timestamp_millis().try_into()?;
        Ok(
            if let Some(RunStopParameters {
                collect_until,
                time_completed: _,
            }) = self.run_stop_parameters
            {
                (self.collect_from..collect_until).contains(&milis)
            } else {
                false
            },
        )
    }

    pub(crate) fn is_ready_to_write(&self, now: &DateTime<Utc>, delay: &Duration) -> bool {
        self.run_stop_parameters
            .as_ref()
            .map(|run_stop_parameters| *now - run_stop_parameters.time_completed > *delay)
            .unwrap_or(false)
    }
}

impl Hdf5Writer for RunParameters {
    fn write_hdf5(&self, parent: &Group) -> Result<()> {
        add_new_field_to(parent, "IDF_version", 2)?;
        add_new_string_field_to(parent, "definition", "muonTD")?;
        add_new_field_to(parent, "run_number", self.run_number)?;
        add_new_string_field_to(parent, "experiment_identifier", "")?;

        let start_time = DateTime::<Utc>::from_timestamp_millis(self.collect_from as i64)
            .ok_or(anyhow!("Cannot create start_time from {0}",self.collect_from))?
            .format(DATETIME_FORMAT)
            .to_string();
        add_new_string_field_to(parent, "start_time", start_time.as_str())?;

        let end_ms = self.run_stop_parameters
            .as_ref()
            .ok_or(anyhow!("File end time not found."))?
            .collect_until as i64;
        let end_time = DateTime::<Utc>::from_timestamp_millis(end_ms)
            .ok_or(anyhow!("Cannot create end_time from {end_ms}"))?
            .format(DATETIME_FORMAT)
            .to_string();
        add_new_string_field_to(parent, "end_time", end_time.as_str())?;

        add_new_string_field_to(parent, "name", self.instrument_name.as_str())?;
        add_new_string_field_to(parent, "title", "")?;
        self.write_instrument(parent)?;
        self.write_periods(parent)?;
        Ok(())
    }
}

impl RunParameters {
    fn write_instrument(&self, parent: &Group) -> Result<()> {
        let instrument = add_new_group_to(parent, "instrument", NX::INSTRUMENT)?;
        add_new_string_field_to(&instrument, "name", self.instrument_name.as_str())?;
        {
            let source = add_new_group_to(&instrument, "source", NX::SOURCE)?;
            add_new_string_field_to(&source, "name", "MuSR")?;
            add_new_string_field_to(&source, "type", "")?;
            add_new_string_field_to(&source, "probe", "")?;
        }
        {
            let _detector = add_new_group_to(&instrument, "detector", NX::DETECTOR)?;
        }
        Ok(())
    }

    fn write_periods(&self, parent: &Group) -> Result<()> {
        let periods = add_new_group_to(parent, "periods", NX::PERIOD)?;
        add_new_field_to(&periods, "number", self.num_periods)?;
        add_new_slice_field_to::<u32>(&periods, "type", &vec![1; self.num_periods as usize])?;
        Ok(())
    }
}

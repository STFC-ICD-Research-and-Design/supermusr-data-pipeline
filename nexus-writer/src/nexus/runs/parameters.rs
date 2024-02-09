use crate::nexus::nexus_class as NX;
use crate::{
    hdf5_writer::Hdf5Writer,
    nexus::hdf5_writer::{
        add_new_field_to, add_new_group_to, add_new_slice_field_to, add_new_string_field_to,
        set_attribute_list_to, set_group_nx_class,
    },
};
use anyhow::{anyhow, Result};
use chrono::{DateTime, Duration, Utc};
use hdf5::Group;
use supermusr_streaming_types::{
    ecs_6s4t_run_stop_generated::RunStop, ecs_pl72_run_start_generated::RunStart,
};
const DATETIME_FORMAT: &str = "%Y-%m-%dT%H:%M:%S%z";

#[derive(Debug)]
pub(crate) struct RunParameters {
    pub(crate) collect_from: u64,
    pub(crate) collect_until: Option<u64>,
    pub(crate) num_periods: u32,
    pub(crate) run_name: String,
    pub(crate) run_number: u32,
    pub(crate) instrument_name: String,
}

impl RunParameters {
    pub(crate) fn new(data: RunStart<'_>, run_number: u32) -> Result<Self> {
        Ok(Self {
            collect_from: data.start_time(),
            collect_until: None,
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
        if self.collect_until.is_some() {
            Err(anyhow!("Stop Command before Start Command"))
        } else {
            if self.collect_from < data.stop_time() {
                self.collect_until = Some(data.stop_time());
                Ok(())
            } else {
                Err(anyhow!("Stop Time earlier than current Start Time."))
            }
        }
    }

    pub(crate) fn is_message_timestamp_valid(&self, timestamp: &DateTime<Utc>) -> Result<bool> {
        let milis : u64 = timestamp.timestamp_millis().try_into()?;
        Ok(
            if let Some(until) = self.collect_until {
                (self.collect_from.. until).contains(&milis)
            } else {
                false
            }
        )
    }
}

impl Hdf5Writer for RunParameters {
    fn write_hdf5(&self, parent: &Group) -> Result<()> {
        add_new_field_to(&parent, "IDF_version", 2)?;
        add_new_string_field_to(&parent, "definition", "muonTD")?;
        add_new_field_to(&parent, "run_number", self.run_number)?;
        add_new_string_field_to(&parent, "experiment_identifier", "")?;
        let start_time = (DateTime::<Utc>::UNIX_EPOCH
            + Duration::milliseconds(self.collect_from as i64))
        .format(DATETIME_FORMAT)
        .to_string();
        add_new_string_field_to(&parent, "start_time", start_time.as_str())?;
        let end_time = (DateTime::<Utc>::UNIX_EPOCH
            + Duration::milliseconds(
                self.collect_until
                    .ok_or(anyhow!("File end time not found."))? as i64,
            ))
        .format(DATETIME_FORMAT)
        .to_string();
        add_new_string_field_to(&parent, "end_time", end_time.as_str())?;
        add_new_string_field_to(&parent, "name", self.instrument_name.as_str())?;
        self.write_instrument(&parent)?;
        self.write_periods(&parent)?;
        Ok(())
    }
}

impl RunParameters {
    fn write_instrument(&self, parent: &Group) -> Result<()> {
        let instrument = add_new_group_to(&parent, "instrument", NX::INSTRUMENT)?;
        add_new_string_field_to(&instrument, "name", self.instrument_name.as_str())?;
        {
            let source = add_new_group_to(&instrument, "source", NX::SOURCE)?;
            add_new_string_field_to(&source, "name", "")?;
            add_new_string_field_to(&source, "type", "")?;
            add_new_string_field_to(&source, "probe", "")?;
        }
        {
            let _detector = add_new_group_to(&instrument, "detector", NX::DETECTOR)?;
        }
        Ok(())
    }

    fn write_periods(&self, parent: &Group) -> Result<()> {
        let periods = add_new_group_to(&parent, "periods", NX::PERIOD)?;
        add_new_field_to(&periods, "number", self.num_periods)?;
        add_new_slice_field_to::<u32>(&periods, "type", &vec![1; self.num_periods as usize])?;
        Ok(())
    }
}

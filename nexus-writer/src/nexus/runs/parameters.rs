use crate::nexus::writer::{
    add_new_field_to, add_new_group_to, add_new_slice_field_to, add_new_string_field_to, set_group_nx_class, set_attribute_list_to
};
use anyhow::{anyhow, Result};
use chrono::{Duration,DateTime, Utc};
use hdf5::Group;
use supermusr_streaming_types::{
    ecs_6s4t_run_stop_generated::RunStop, ecs_pl72_run_start_generated::RunStart,
};
use crate::nexus::nexus_class as NX;
const DATETIME_FORMAT : &str = "%Y-%m-%dT%H:%M:%S%z";

#[derive(Debug)]
pub(crate) struct RunParameters {
    pub(crate) collect_from: u64,
    pub(crate) collect_until: Option<u64>,
    pub(crate) num_periods: u32,
    pub(crate) run_name: String,
    pub(crate) instrument_name: String,
}

impl RunParameters {
    pub(crate) fn new(data: RunStart<'_>) -> Result<Self> {
        Ok(Self {
            collect_from: data.start_time(),
            collect_until: None,
            num_periods: data.n_periods(),
            run_name: data
                .run_name()
                .ok_or(anyhow!("Run Name not found"))?
                .to_owned(),
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

    pub(crate) fn is_message_timestamp_valid(&self, timestamp: &DateTime<Utc>) -> bool {
        let milis = timestamp.timestamp_millis();
        assert!(milis > 0);
        if let Some(until) = self.collect_until {
            // maybe use (self.collect_from.. until).contains(milis as u64)
            self.collect_from < (milis as u64) && (milis as u64) < until
        } else {
            false
        }
    }

    pub(crate) fn write_header(&self, parent: &Group, run_number: usize) -> Result<Group> {
        set_group_nx_class(parent, NX::ROOT)?;

        set_attribute_list_to(parent, &[
            ("HDF5_version", "1.14.3"), // Can this be taken directly from the nix package?
            ("NeXus_version", ""),      // Where does this come from?
            ("file_name", &parent.filename()),  //  This should be absolutized at some point
            ("file_time", Utc::now().to_string().as_str())  //  This should be formatted, the nanoseconds are overkill.
        ])?;

        let entry = add_new_group_to(parent, "raw_data_1", NX::ENTRY)?;

        add_new_field_to(&entry, "IDF_version", 2)?;
        add_new_string_field_to(&entry, "definition", "muonTD")?;
        add_new_field_to(&entry, "run_number", run_number)?;
        add_new_string_field_to(&entry, "experiment_identifier", "")?;
        let start_time = (DateTime::<Utc>::UNIX_EPOCH
            + Duration::milliseconds(self.collect_from as i64))
            .format(DATETIME_FORMAT)
            .to_string();
        add_new_string_field_to(&entry, "start_time", start_time.as_str())?;
        let end_time = (DateTime::<Utc>::UNIX_EPOCH
            + Duration::milliseconds(
                self.collect_until
                    .ok_or(anyhow!("File end time not found."))? as i64,
            ))
            .format(DATETIME_FORMAT)
            .to_string();
        add_new_string_field_to(&entry, "end_time", end_time.as_str())?;
        add_new_string_field_to(&entry, "name", self.instrument_name.as_str())?;
        self.write_instrument(&entry)?;
        self.write_periods(&entry)?;

        add_new_group_to(&entry, "detector_1", NX::EVENT_DATA)
    }

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
        add_new_slice_field_to::<u32>(&periods, "type", &vec![1;self.num_periods as usize])?;
        Ok(())
    }
}

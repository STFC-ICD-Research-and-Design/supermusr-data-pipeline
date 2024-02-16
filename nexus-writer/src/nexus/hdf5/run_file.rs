use super::{
    add_attribute_to, add_new_group_to, create_resizable_dataset, set_group_nx_class, set_slice_to,
    set_string_to, EventRun,
};
use crate::{
    nexus::{
        nexus_class as NX,
        RunParameters,
        DATETIME_FORMAT,
    },
    GenericEventMessage
};
use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use hdf5::{types::VarLenUnicode, Dataset, File};
use std::{fs::create_dir_all, path::Path};
use tracing::debug;

#[derive(Debug)]
pub(crate) struct RunFile {
    file: File,

    idf_version: Dataset,
    definition: Dataset,
    run_number: Dataset,
    experiment_identifier: Dataset,

    start_time: Dataset,
    end_time: Dataset,
    name: Dataset,
    title: Dataset,

    instrument_name: Dataset,

    source_name: Dataset,
    source_type: Dataset,
    source_probe: Dataset,

    period_number: Dataset,
    period_type: Dataset,

    lists: EventRun,
}

impl RunFile {
    pub(crate) fn new(filename: &Path, run_name: &str) -> Result<Self> {
        create_dir_all(filename)?;
        let filename = {
            let mut filename = filename.to_owned();
            filename.push(run_name);
            filename.set_extension("nxs");
            filename
        };
        debug!("File save begin. File: {0}.", filename.display());

        let file = File::create(filename)?;
        set_group_nx_class(&file, NX::ROOT)?;

        add_attribute_to(&file, "HDF5_version", "1.14.3")?; // Can this be taken directly from the nix package;
        add_attribute_to(&file, "NeXus_version", "")?; // Where does this come from?
        add_attribute_to(&file, "file_name", &file.filename())?; //  This should be absolutized at some point
        add_attribute_to(&file, "file_time", Utc::now().to_string().as_str())?; //  This should be formatted, the nanoseconds are overkill.

        let entry = add_new_group_to(&file, "raw_data_1", NX::ENTRY)?;

        let idf_version = entry.new_dataset::<u32>().create("IDF_version")?;
        let definition = entry.new_dataset::<VarLenUnicode>().create("definition")?;
        let run_number = entry.new_dataset::<u32>().create("run_number")?;
        let experiment_identifier = entry
            .new_dataset::<VarLenUnicode>()
            .create("experiment_identifier")?;

        let start_time = entry.new_dataset::<VarLenUnicode>().create("start_time")?;
        let end_time = entry.new_dataset::<VarLenUnicode>().create("end_time")?;

        let name = entry.new_dataset::<VarLenUnicode>().create("name")?;
        let title = entry.new_dataset::<VarLenUnicode>().create("title")?;

        let instrument = add_new_group_to(&entry, "instrument", NX::INSTRUMENT)?;
        let instrument_name = instrument.new_dataset::<VarLenUnicode>().create("name")?;

        let periods = add_new_group_to(&entry, "periods", NX::PERIOD)?;
        let period_number = periods.new_dataset::<u32>().create("number")?;
        let period_type = create_resizable_dataset::<u32>(&periods, "type", 0, 32)?;

        let source = add_new_group_to(&instrument, "source", NX::SOURCE)?;
        let source_name = source.new_dataset::<VarLenUnicode>().create("name")?;
        let source_type = source.new_dataset::<VarLenUnicode>().create("type")?;
        let source_probe = source.new_dataset::<VarLenUnicode>().create("probe")?;

        let _detector = add_new_group_to(&instrument, "detector", NX::DETECTOR)?;

        let lists = EventRun::new(&entry)?;

        Ok(Self {
            file,
            idf_version,
            start_time,
            end_time,
            name,
            title,
            instrument_name,
            period_number,
            period_type,
            lists,
            source_name,
            source_type,
            source_probe,
            definition,
            run_number,
            experiment_identifier,
        })
    }

    pub(crate) fn open(filename: &Path, run_name: &str) -> Result<Self> {
        let filename = {
            let mut filename = filename.to_owned();
            filename.push(run_name);
            filename.set_extension("nxs");
            filename
        };
        debug!("File open begin. File: {0}.", filename.display());

        let file = File::open_rw(filename)?;

        let entry = file.group("raw_data_1")?;

        let idf_version = entry.dataset("IDF_version")?;
        let definition = entry.dataset("definition")?;
        let run_number = entry.dataset("run_number")?;
        let experiment_identifier = entry.dataset("experiment_identifier")?;

        let start_time = entry.dataset("start_time")?;
        let end_time = entry.dataset("end_time")?;

        let name = entry.dataset("name")?;
        let title = entry.dataset("title")?;

        let periods = entry.group("periods")?;
        let period_number = periods.dataset("number")?;
        let period_type = periods.dataset("type")?;

        let instrument = entry.group("instrument")?;
        let instrument_name = instrument.dataset("name")?;

        let source = instrument.group("source")?;
        let source_name = source.dataset("name")?;
        let source_type = source.dataset("type")?;
        let source_probe = source.dataset("probe")?;

        let _detector = instrument.group("detector")?;

        let lists = EventRun::open(&entry)?;

        Ok(Self {
            file,
            idf_version,
            start_time,
            end_time,
            name,
            title,
            instrument_name,
            period_number,
            period_type,
            lists,
            source_name,
            source_type,
            source_probe,
            definition,
            run_number,
            experiment_identifier,
        })
    }

    pub(crate) fn init(&mut self, parameters: &RunParameters) -> Result<()> {
        self.idf_version.write_scalar(&2)?;
        self.run_number.write_scalar(&parameters.run_number)?;

        set_string_to(&self.definition, "muonTD")?;
        set_string_to(&self.experiment_identifier, "")?;

        let start_time = DateTime::<Utc>::from_timestamp_millis(parameters.collect_from as i64)
            .ok_or(anyhow!(
                "Cannot create start_time from {0}",
                &parameters.collect_from
            ))?
            .format(DATETIME_FORMAT)
            .to_string();

        set_string_to(&self.start_time, &start_time)?;

        set_string_to(&self.name, &parameters.run_name)?;
        set_string_to(&self.title, "")?;

        set_string_to(&self.instrument_name, &parameters.instrument_name)?;

        self.period_number.write_scalar(&parameters.num_periods)?;
        set_slice_to(&self.period_type, &vec![1; parameters.num_periods as usize])?;

        set_string_to(&self.source_name, "MuSR")?;
        set_string_to(&self.source_type, "")?;
        set_string_to(&self.source_probe, "")?;
        Ok(())
    }

    pub(crate) fn set_end_time(&mut self, end_ms: i64) -> Result<()> {
        let end_time = DateTime::<Utc>::from_timestamp_millis(end_ms)
            .ok_or(anyhow!("Cannot create end_time from {end_ms}"))?
            .format(DATETIME_FORMAT)
            .to_string();

        set_string_to(&self.end_time, &end_time)?;
        Ok(())
    }

    pub(crate) fn ensure_end_time_is_set(
        &mut self,
        parameters: &RunParameters,
        message: &GenericEventMessage,
    ) -> Result<()> {
        let end_ms = {
            if let Some(run_stop_parameters) = &parameters.run_stop_parameters {
                run_stop_parameters.collect_until as i64
            } else {
                let ns = message
                    .time
                    .and_then(|time| time.iter().last())
                    .ok_or(anyhow!("Event time missing."))?;

                message.timestamp.timestamp_millis() + ns.div_ceil(1_000_000) as i64
            }
        };
        self.set_end_time(end_ms)
    }

    pub(crate) fn push_message(
        &mut self,
        parameters: &RunParameters,
        message: &GenericEventMessage,
    ) -> Result<()> {
        self.lists.push_message(message)?;
        self.ensure_end_time_is_set(parameters, message)?;
        Ok(())
    }
    
    pub(crate) fn close(self) -> Result<()> {
        self.file.close()?;
        Ok(())
    }
}

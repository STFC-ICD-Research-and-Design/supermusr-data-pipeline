use super::{
    add_attribute_to, add_new_group_to, create_resizable_dataset, set_group_nx_class, set_slice_to,
    set_string_to, EventRun,
};
use crate::nexus::{
    hdf5_file::run_file_components::{RunLog, SeLog},
    nexus_class as NX, NexusSettings, RunParameters, DATETIME_FORMAT,
};
use chrono::{DateTime, Duration, Utc};
use hdf5::{types::VarLenUnicode, Dataset, File};
use std::{fs::create_dir_all, path::Path};
use supermusr_streaming_types::{
    aev2_frame_assembled_event_v2_generated::FrameAssembledEventListMessage,
    ecs_al00_alarm_generated::Alarm, ecs_f144_logdata_generated::f144_LogData,
    ecs_se00_data_generated::se00_SampleEnvironmentData,
};
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

    logs: RunLog,

    source_name: Dataset,
    source_type: Dataset,
    source_probe: Dataset,

    period_number: Dataset,
    period_type: Dataset,

    selogs: SeLog,

    lists: EventRun,
}

impl RunFile {
    #[tracing::instrument(skip_all, err(level = "warn"))]
    pub(crate) fn new_runfile(
        filename: &Path,
        run_name: &str,
        nexus_settings: &NexusSettings,
    ) -> anyhow::Result<Self> {
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

        let logs = RunLog::new_runlog(&entry)?;

        let periods = add_new_group_to(&entry, "periods", NX::PERIOD)?;
        let period_number = periods.new_dataset::<u32>().create("number")?;
        let period_type = create_resizable_dataset::<u32>(
            &periods,
            "type",
            0,
            nexus_settings.periodlist_chunk_size,
        )?;

        let selogs = SeLog::new_selog(&entry)?;

        let source = add_new_group_to(&instrument, "source", NX::SOURCE)?;
        let source_name = source.new_dataset::<VarLenUnicode>().create("name")?;
        let source_type = source.new_dataset::<VarLenUnicode>().create("type")?;
        let source_probe = source.new_dataset::<VarLenUnicode>().create("probe")?;

        let _detector = add_new_group_to(&instrument, "detector", NX::DETECTOR)?;

        let lists = EventRun::new_event_runfile(&entry, nexus_settings)?;

        Ok(Self {
            file,
            idf_version,
            start_time,
            end_time,
            name,
            title,
            instrument_name,
            logs,
            period_number,
            period_type,
            selogs,
            lists,
            source_name,
            source_type,
            source_probe,
            definition,
            run_number,
            experiment_identifier,
        })
    }

    #[tracing::instrument(skip_all, err(level = "warn"))]
    pub(crate) fn open_runfile(filename: &Path, run_name: &str) -> anyhow::Result<Self> {
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

        let selogs = SeLog::open_selog(&entry)?;

        let instrument = entry.group("instrument")?;
        let instrument_name = instrument.dataset("name")?;

        let logs = RunLog::open_runlog(&entry)?;

        let source = instrument.group("source")?;
        let source_name = source.dataset("name")?;
        let source_type = source.dataset("type")?;
        let source_probe = source.dataset("probe")?;

        let _detector = instrument.group("detector")?;

        let lists = EventRun::open_event_runfile(&entry)?;

        Ok(Self {
            file,
            idf_version,
            start_time,
            end_time,
            name,
            title,
            instrument_name,
            logs,
            period_number,
            period_type,
            selogs,
            lists,
            source_name,
            source_type,
            source_probe,
            definition,
            run_number,
            experiment_identifier,
        })
    }

    #[tracing::instrument(skip_all, level = "trace", err(level = "warn"))]
    pub(crate) fn init(&mut self, parameters: &RunParameters) -> anyhow::Result<()> {
        self.idf_version.write_scalar(&2)?;
        self.run_number.write_scalar(&parameters.run_number)?;

        set_string_to(&self.definition, "muonTD")?;
        set_string_to(&self.experiment_identifier, "")?;

        let start_time = parameters.collect_from.format(DATETIME_FORMAT).to_string();

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

    #[tracing::instrument(skip_all, level = "trace", err(level = "warn"))]
    pub(crate) fn set_end_time(&mut self, end_time: &DateTime<Utc>) -> anyhow::Result<()> {
        let end_time = end_time.format(DATETIME_FORMAT).to_string();

        set_string_to(&self.end_time, &end_time)?;
        Ok(())
    }

    #[tracing::instrument(skip_all, level = "trace", err(level = "warn"))]
    pub(crate) fn ensure_end_time_is_set(
        &mut self,
        parameters: &RunParameters,
        message: &FrameAssembledEventListMessage,
    ) -> anyhow::Result<()> {
        let end_time = {
            if let Some(run_stop_parameters) = &parameters.run_stop_parameters {
                run_stop_parameters.collect_until
            } else {
                let time = message
                    .time()
                    .ok_or(anyhow::anyhow!("Event time missing."))?;

                let ms = if time.is_empty() {
                    0
                } else {
                    time.get(time.len() - 1).div_ceil(1_000_000).into()
                };

                let duration = Duration::try_milliseconds(ms)
                    .ok_or(anyhow::anyhow!("Invalid duration {ms}ms."))?;

                let timestamp: DateTime<Utc> = (*message
                    .metadata()
                    .timestamp()
                    .ok_or(anyhow::anyhow!("Message timestamp missing."))?)
                .try_into()?;

                timestamp
                    .checked_add_signed(duration)
                    .ok_or(anyhow::anyhow!("Unable to add {duration} to {timestamp}"))?
            }
        };
        self.set_end_time(&end_time)
    }

    #[tracing::instrument(skip_all, level = "trace", err(level = "warn"))]
    pub(crate) fn push_logdata_to_runfile(
        &mut self,
        logdata: &f144_LogData,
        nexus_settings: &NexusSettings,
    ) -> anyhow::Result<()> {
        self.logs.push_logdata_to_runlog(logdata, nexus_settings)
    }

    #[tracing::instrument(skip_all, level = "trace", err(level = "warn"))]
    pub(crate) fn push_alarm_to_runfile(&mut self, alarm: Alarm) -> anyhow::Result<()> {
        self.selogs.push_alarm_to_selog(alarm)
    }

    #[tracing::instrument(skip_all, level = "trace", err(level = "warn"))]
    pub(crate) fn push_selogdata(
        &mut self,
        selogdata: se00_SampleEnvironmentData,
        nexus_settings: &NexusSettings,
    ) -> anyhow::Result<()> {
        self.selogs
            .push_selogdata_to_selog(&selogdata, nexus_settings)
    }

    #[tracing::instrument(skip_all, level = "trace", err(level = "warn"))]
    pub(crate) fn push_message_to_runfile(
        &mut self,
        parameters: &RunParameters,
        message: &FrameAssembledEventListMessage,
    ) -> anyhow::Result<()> {
        self.lists.push_message_to_event_runfile(message)?;
        self.ensure_end_time_is_set(parameters, message)?;
        Ok(())
    }

    #[tracing::instrument(skip_all, level = "trace", err(level = "warn"))]
    pub(crate) fn close(self) -> anyhow::Result<()> {
        self.file.close()?;
        Ok(())
    }
}

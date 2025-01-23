use super::{
    hdf5_writer::{DatasetExt, GroupExt, HasAttributesExt}, EventRun
};
use crate::nexus::{
    hdf5_file::run_file_components::{RunLog, SeLog},
    nexus_class as NX,
    run_parameters::RunStopParameters,
    NexusConfiguration, NexusSettings, RunParameters, DATETIME_FORMAT,
};
use chrono::{DateTime, Utc};
use hdf5::{types::VarLenUnicode, Dataset, File, H5Type};
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
    contents: RunFileContents,
}

#[derive(Debug)]
struct RunFileContents {
    idf_version: Dataset,
    definition: Dataset,
    program_name: Dataset,
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

impl RunFileContents {
    #[tracing::instrument(skip_all, err(level = "warn"))]
    pub(crate) fn populate_new_runfile(
        file: &File,
        nexus_settings: &NexusSettings,
    ) -> anyhow::Result<Self> {
        file.set_nx_class(NX::ROOT)?;

        file.add_attribute_to("HDF5_version", "1.14.3")?; // Can this be taken directly from the nix package;
        file.add_attribute_to("NeXus_version", "")?; // Where does this come from?
        file.add_attribute_to("file_name", &file.filename())?; //  This should be absolutized at some point
        file.add_attribute_to("file_time", Utc::now().to_string().as_str())?; //  This should be formatted, the nanoseconds are overkill.

        let entry = file.add_new_group_to("raw_data_1", NX::ENTRY)?;

        let idf_version = entry.new_dataset::<u32>().create("IDF_version")?;
        let definition = entry.new_dataset::<VarLenUnicode>().create("definition")?;
        let program_name = entry
            .new_dataset::<VarLenUnicode>()
            .create("program_name")?;

        let run_number = entry.new_dataset::<u32>().create("run_number")?;
        let experiment_identifier = entry
            .new_dataset::<VarLenUnicode>()
            .create("experiment_identifier")?;

        let start_time = entry.new_dataset::<VarLenUnicode>().create("start_time")?;
        let end_time = entry.new_dataset::<VarLenUnicode>().create("end_time")?;

        let name = entry.new_dataset::<VarLenUnicode>().create("name")?;
        let title = entry.new_dataset::<VarLenUnicode>().create("title")?;

        let instrument = entry.add_new_group_to("instrument", NX::INSTRUMENT)?;
        let instrument_name = instrument.new_dataset::<VarLenUnicode>().create("name")?;

        let logs = RunLog::new_runlog(&entry)?;

        let periods = entry.add_new_group_to("periods", NX::PERIOD)?;
        let period_number = periods.new_dataset::<u32>().create("number")?;
        let period_type = periods.create_resizable_dataset::<u32>(
            "type",
            0,
            nexus_settings.periodlist_chunk_size,
        )?;

        let selogs = SeLog::new_selog(&entry)?;

        let source = instrument.add_new_group_to("source", NX::SOURCE)?;
        let source_name = source.new_dataset::<VarLenUnicode>().create("name")?;
        let source_type = source.new_dataset::<VarLenUnicode>().create("type")?;
        let source_probe = source.new_dataset::<VarLenUnicode>().create("probe")?;

        let _detector = instrument.add_new_group_to("detector", NX::DETECTOR)?;

        let lists = EventRun::new_event_runfile(&entry, nexus_settings)?;

        Ok(Self {
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
            program_name,
            experiment_identifier,
        })
    }

    fn populate_open_runfile(file: &File) -> anyhow::Result<Self> {
        let entry = file.group("raw_data_1")?;

        let idf_version = entry.dataset("IDF_version")?;
        let definition = entry.dataset("definition")?;
        let run_number = entry.dataset("run_number")?;
        let program_name = entry.dataset("program_name")?;
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
            program_name,
            experiment_identifier,
        })
    }
}

impl RunFile {
    #[tracing::instrument(skip_all, err(level = "warn"))]
    pub(crate) fn new_runfile(
        path: &Path,
        run_name: &str,
        nexus_settings: &NexusSettings,
    ) -> anyhow::Result<Self> {
        create_dir_all(path)?;
        let filename = RunParameters::get_hdf5_filename(path, run_name);
        debug!("File save begin. File: {0}.", filename.display());

        let file = File::create(filename)?;
        match RunFileContents::populate_new_runfile(&file, nexus_settings) {
            Ok(contents) => Ok(Self { file, contents }),
            Err(e) => {
                file.close()?;
                Err(e)
            }
        }
    }

    #[tracing::instrument(skip_all, err(level = "warn"))]
    pub(crate) fn open_runfile(local_path: &Path, run_name: &str) -> anyhow::Result<Self> {
        let filename = RunParameters::get_hdf5_filename(local_path, run_name);
        debug!("File open begin. File: {0}.", filename.display());

        let file = File::open_rw(filename)?;
        match RunFileContents::populate_open_runfile(&file) {
            Ok(contents) => Ok(Self { file, contents }),
            Err(e) => {
                file.close()?;
                Err(e)
            }
        }
    }

    #[tracing::instrument(skip_all, level = "trace", err(level = "warn"))]
    pub(crate) fn init(
        &mut self,
        parameters: &RunParameters,
        nexus_configuration: &NexusConfiguration,
    ) -> anyhow::Result<()> {
        self.contents.idf_version.write_scalar(&2)?;
        self.contents
            .run_number
            .write_scalar(&parameters.run_number)?;

        self.contents.definition.set_string_to("muonTD")?;
        self.contents.experiment_identifier.set_string_to("")?;

        self.contents.program_name.set_string_to(
            "SuperMuSR Data Pipeline Nexus Writer",
        )?;
        self.contents.program_name.add_attribute_to("version", "1.0")?;
        self.contents.program_name.add_attribute_to(
            "configuration",
            &nexus_configuration.configuration,
        )?;

        let start_time = parameters.collect_from.format(DATETIME_FORMAT).to_string();

        self.contents.start_time.set_string_to(&start_time)?;
        self.contents.end_time.set_string_to("")?;

        self.contents.name.set_string_to(&parameters.run_name)?;
        self.contents.title.set_string_to("")?;

        self.contents.instrument_name.set_string_to(&parameters.instrument_name)?;

        self.contents
            .period_number
            .set_scalar_to(&parameters.num_periods)?;
        self.contents.period_type.set_slice_to(
            &vec![1; parameters.num_periods as usize],
        )?;

        self.contents.source_name.set_string_to("MuSR")?;
        self.contents.source_type.set_string_to("")?;
        self.contents.source_probe.set_string_to("")?;

        self.contents.lists.init(&parameters.collect_from)?;
        Ok(())
    }

    #[tracing::instrument(skip_all, level = "trace", err(level = "warn"))]
    pub(crate) fn set_end_time(&mut self, end_time: &DateTime<Utc>) -> anyhow::Result<()> {
        let end_time = end_time.format(DATETIME_FORMAT).to_string();

        self.contents.end_time.set_string_to(&end_time)?;
        Ok(())
    }

    #[tracing::instrument(skip_all, level = "trace", err(level = "warn"))]
    pub(crate) fn push_logdata_to_runfile(
        &mut self,
        logdata: &f144_LogData,
        nexus_settings: &NexusSettings,
    ) -> anyhow::Result<()> {
        self.contents
            .logs
            .push_logdata_to_runlog(logdata, nexus_settings)
    }

    #[tracing::instrument(skip_all, level = "trace", err(level = "warn"))]
    pub(crate) fn push_alarm_to_runfile(&mut self, alarm: Alarm) -> anyhow::Result<()> {
        self.contents.selogs.push_alarm_to_selog(alarm)
    }

    #[tracing::instrument(skip_all, level = "trace", err(level = "warn"))]
    pub(crate) fn push_selogdata(
        &mut self,
        selogdata: se00_SampleEnvironmentData,
        nexus_settings: &NexusSettings,
    ) -> anyhow::Result<()> {
        self.contents
            .selogs
            .push_selogdata_to_selog(&selogdata, nexus_settings)
    }

    #[tracing::instrument(skip_all, level = "trace", err(level = "warn"))]
    pub(crate) fn push_message_to_runfile(
        &mut self,
        message: &FrameAssembledEventListMessage,
        nexus_settings: &NexusSettings,
    ) -> anyhow::Result<()> {
        self.contents.lists.push_message_to_event_runfile(message)?;

        if !message.complete() {
            let time_zero = self.contents.lists.get_time_zero(message)?;

            self.contents.logs.push_incomplete_frame_log(
                time_zero,
                message
                    .digitizers_present()
                    .unwrap_or_default()
                    .iter()
                    .collect(),
                nexus_settings,
            )?;
        }
        Ok(())
    }

    fn try_read_scalar<T: H5Type>(dataset: &Dataset) -> anyhow::Result<T> {
        if dataset.storage_size() != 0 {
            if dataset.is_scalar() {
                Ok(dataset.read_scalar::<T>()?)
            } else {
                anyhow::bail!("{} is not a scalar", dataset.name())
            }
        } else {
            anyhow::bail!("{} is not allocated", dataset.name())
        }
    }

    #[tracing::instrument(skip_all, level = "trace", err(level = "warn"))]
    pub(crate) fn extract_run_parameters(&self) -> anyhow::Result<RunParameters> {
        let collect_from: DateTime<Utc> =
            Self::try_read_scalar::<VarLenUnicode>(&self.contents.start_time)?.parse()?;
        let run_name = Self::try_read_scalar::<VarLenUnicode>(&self.contents.name)?.into();
        let run_number = Self::try_read_scalar::<u32>(&self.contents.run_number)?;
        let num_periods = Self::try_read_scalar::<u32>(&self.contents.period_number)?;
        let instrument_name =
            Self::try_read_scalar::<VarLenUnicode>(&self.contents.instrument_name)?.into();
        let run_stop_parameters = Self::try_read_scalar::<VarLenUnicode>(&self.contents.end_time)?
            .parse()
            .map(|collect_until| RunStopParameters {
                collect_until,
                last_modified: Utc::now(),
            })
            .ok();
        Ok(RunParameters {
            collect_from,
            run_stop_parameters,
            num_periods,
            run_name,
            run_number,
            instrument_name,
        })
    }

    #[tracing::instrument(skip_all, level = "trace", err(level = "warn"))]
    pub(crate) fn set_aborted_run_warning(
        &mut self,
        stop_time: i32,
        nexus_settings: &NexusSettings,
    ) -> anyhow::Result<()> {
        self.contents
            .logs
            .set_aborted_run_warning(stop_time, nexus_settings)?;
        Ok(())
    }

    #[tracing::instrument(skip_all, level = "trace", err(level = "warn"))]
    pub(crate) fn close(self) -> anyhow::Result<()> {
        self.file.close()?;
        Ok(())
    }
}

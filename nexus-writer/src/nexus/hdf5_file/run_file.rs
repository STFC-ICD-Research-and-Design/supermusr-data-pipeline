use super::{
    error::{ConvertResult, NexusHDF5Result},
    hdf5_writer::{DatasetExt, GroupExt, HasAttributesExt},
    EventRun,
};
use crate::{
    nexus::{
        hdf5_file::run_file_components::{RunLog, SeLog},
        nexus_class as NX,
        run_parameters::RunStopParameters,
        NexusConfiguration, NexusDateTime, NexusSettings, RunParameters, DATETIME_FORMAT,
    },
    message_handlers::SampleEnvironmentLog,
};
use chrono::Utc;
use hdf5::{types::VarLenUnicode, Dataset, File};
use std::{fs::create_dir_all, path::Path};
use supermusr_streaming_types::{
    aev2_frame_assembled_event_v2_generated::FrameAssembledEventListMessage,
    ecs_al00_alarm_generated::Alarm, ecs_f144_logdata_generated::f144_LogData,
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
    ) -> NexusHDF5Result<Self> {
        file.set_nx_class(NX::ROOT)?;

        file.add_attribute_to("HDF5_version", "1.14.3")?; // Can this be taken directly from the nix package;
        file.add_attribute_to("NeXus_version", "")?; // Where does this come from?
        file.add_attribute_to("file_name", &file.filename())?; //  This should be absolutized at some point
        file.add_attribute_to("file_time", Utc::now().to_string().as_str())?; //  This should be formatted, the nanoseconds are overkill.

        let entry = file.add_new_group_to("raw_data_1", NX::ENTRY)?;

        let idf_version = entry.create_scalar_dataset::<u32>("IDF_version")?;
        let definition = entry.create_scalar_dataset::<VarLenUnicode>("definition")?;
        let program_name = entry.create_scalar_dataset::<VarLenUnicode>("program_name")?;

        let run_number = entry.create_scalar_dataset::<u32>("run_number")?;
        let experiment_identifier =
            entry.create_scalar_dataset::<VarLenUnicode>("experiment_identifier")?;

        let start_time = entry.create_scalar_dataset::<VarLenUnicode>("start_time")?;
        let end_time = entry.create_scalar_dataset::<VarLenUnicode>("end_time")?;

        let name = entry.create_scalar_dataset::<VarLenUnicode>("name")?;
        let title = entry.create_scalar_dataset::<VarLenUnicode>("title")?;

        let instrument = entry.add_new_group_to("instrument", NX::INSTRUMENT)?;
        let instrument_name = instrument.create_scalar_dataset::<VarLenUnicode>("name")?;

        let logs = RunLog::new_runlog(&entry)?;

        let periods = entry.add_new_group_to("periods", NX::PERIOD)?;
        let period_number = periods.create_scalar_dataset::<u32>("number")?;
        let period_type = periods
            .create_resizable_empty_dataset::<u32>("type", nexus_settings.periodlist_chunk_size)?;

        let selogs = SeLog::new_selog(&entry)?;

        let source = instrument.add_new_group_to("source", NX::SOURCE)?;
        let source_name = source.create_scalar_dataset::<VarLenUnicode>("name")?;
        let source_type = source.create_scalar_dataset::<VarLenUnicode>("type")?;
        let source_probe = source.create_scalar_dataset::<VarLenUnicode>("probe")?;

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

    fn populate_open_runfile(file: &File) -> NexusHDF5Result<Self> {
        let entry = file.get_group("raw_data_1")?;

        let idf_version = entry.get_dataset("IDF_version")?;
        let definition = entry.get_dataset("definition")?;
        let run_number = entry.get_dataset("run_number")?;
        let program_name = entry.get_dataset("program_name")?;
        let experiment_identifier = entry.get_dataset("experiment_identifier")?;

        let start_time = entry.get_dataset("start_time")?;
        let end_time = entry.get_dataset("end_time")?;

        let name = entry.get_dataset("name")?;
        let title = entry.get_dataset("title")?;

        let periods = entry.get_group("periods")?;
        let period_number = periods.get_dataset("number")?;
        let period_type = periods.get_dataset("type")?;

        let selogs = SeLog::open_selog(&entry)?;

        let instrument = entry.get_group("instrument")?;
        let instrument_name = instrument.get_dataset("name")?;

        let logs = RunLog::open_runlog(&entry)?;

        let source = instrument.get_group("source")?;
        let source_name = source.get_dataset("name")?;
        let source_type = source.get_dataset("type")?;
        let source_probe = source.get_dataset("probe")?;

        let _detector = instrument.get_group("detector")?;

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
    ) -> NexusHDF5Result<Self> {
        create_dir_all(path).err_file()?;
        let filename = RunParameters::get_hdf5_filename(path, run_name);
        debug!("File save begin. File: {0}.", filename.display());

        let file = File::create(filename).err_file()?;
        match RunFileContents::populate_new_runfile(&file, nexus_settings) {
            Ok(contents) => Ok(Self { file, contents }),
            Err(e) => {
                file.close().err_file()?;
                Err(e)
            }
        }
    }

    #[tracing::instrument(skip_all, err(level = "warn"))]
    pub(crate) fn open_runfile(local_path: &Path, run_name: &str) -> NexusHDF5Result<Self> {
        let filename = RunParameters::get_hdf5_filename(local_path, run_name);
        debug!("File open begin. File: {0}.", filename.display());

        let file = File::open_rw(filename).err_file()?;
        match RunFileContents::populate_open_runfile(&file) {
            Ok(contents) => Ok(Self { file, contents }),
            Err(e) => {
                file.close().err_file()?;
                Err(e)
            }
        }
    }

    #[tracing::instrument(skip_all, level = "trace", err(level = "warn"))]
    pub(crate) fn init(
        &mut self,
        parameters: &RunParameters,
        nexus_configuration: &NexusConfiguration,
    ) -> NexusHDF5Result<()> {
        self.contents.idf_version.set_scalar_to(&2)?;
        self.contents
            .run_number
            .set_scalar_to(&parameters.run_number)?;

        self.contents.definition.set_string_to("muonTD")?;
        self.contents.experiment_identifier.set_string_to("")?;

        self.contents
            .program_name
            .set_string_to("SuperMuSR Data Pipeline Nexus Writer")?;
        self.contents
            .program_name
            .add_attribute_to("version", "1.0")?;
        self.contents
            .program_name
            .add_attribute_to("configuration", &nexus_configuration.configuration)?;

        let start_time = parameters.collect_from.format(DATETIME_FORMAT).to_string();

        self.contents.start_time.set_string_to(&start_time)?;
        self.contents.end_time.set_string_to("")?;

        self.contents.name.set_string_to(&parameters.run_name)?;
        self.contents.title.set_string_to("")?;

        self.contents
            .instrument_name
            .set_string_to(&parameters.instrument_name)?;

        self.contents
            .period_number
            .set_scalar_to(&parameters.num_periods)?;
        self.contents
            .period_type
            .set_slice_to(&vec![1; parameters.num_periods as usize])?;

        self.contents.source_name.set_string_to("MuSR")?;
        self.contents.source_type.set_string_to("")?;
        self.contents.source_probe.set_string_to("")?;

        self.contents.lists.init(&parameters.collect_from)?;
        Ok(())
    }

    #[tracing::instrument(skip_all, level = "trace", err(level = "warn"))]
    pub(crate) fn set_end_time(&mut self, end_time: &NexusDateTime) -> NexusHDF5Result<()> {
        let end_time = end_time.format(DATETIME_FORMAT).to_string();

        self.contents.end_time.set_string_to(&end_time)?;
        Ok(())
    }

    #[tracing::instrument(skip_all, level = "trace", err(level = "warn"))]
    pub(crate) fn push_logdata_to_runfile(
        &mut self,
        logdata: &f144_LogData,
        origin_time: &NexusDateTime,
        nexus_settings: &NexusSettings,
    ) -> NexusHDF5Result<()> {
        self.contents
            .logs
            .push_logdata_to_runlog(logdata, origin_time, nexus_settings)
    }

    #[tracing::instrument(skip_all, level = "trace", err(level = "warn"))]
    pub(crate) fn push_alarm_to_runfile(
        &mut self,
        alarm: Alarm,
        origin_time: &NexusDateTime,
        nexus_settings: &NexusSettings,
    ) -> NexusHDF5Result<()> {
        self.contents
            .selogs
            .push_alarm_to_selog(alarm, origin_time, nexus_settings)
    }

    #[tracing::instrument(skip_all, level = "trace", err(level = "warn"))]
    pub(crate) fn push_selogdata(
        &mut self,
        selogdata: SampleEnvironmentLog,
        origin_time: &NexusDateTime,
        nexus_settings: &NexusSettings,
    ) -> NexusHDF5Result<()> {
        match selogdata {
            SampleEnvironmentLog::LogData(f144_log_data) => self
                .contents
                .selogs
                .push_logdata_to_selog(&f144_log_data, origin_time, nexus_settings),
            SampleEnvironmentLog::SampleEnvironmentData(se00_sample_environment_data) => {
                self.contents.selogs.push_selogdata_to_selog(
                    &se00_sample_environment_data,
                    origin_time,
                    nexus_settings,
                )
            }
        }
    }

    #[tracing::instrument(skip_all, level = "trace", err(level = "warn"))]
    pub(crate) fn push_frame_eventlist_message_to_runfile(
        &mut self,
        message: &FrameAssembledEventListMessage,
    ) -> NexusHDF5Result<()> {
        self.contents
            .lists
            .push_frame_eventlist_message_to_runfile(message)
    }

    #[tracing::instrument(skip_all, level = "trace", err(level = "warn"))]
    pub(crate) fn extract_run_parameters(&self) -> NexusHDF5Result<RunParameters> {
        let collect_from = self.contents.start_time.get_datetime_from()?;
        let run_name = self.contents.name.get_string_from()?;
        let run_number = self.contents.run_number.get_scalar_from()?;
        let num_periods = self.contents.period_number.get_scalar_from()?;
        let instrument_name = self.contents.instrument_name.get_string_from()?;
        let run_stop_parameters = self
            .contents
            .end_time
            .get_datetime_from()
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
    pub(crate) fn push_incomplete_frame_warning(
        &mut self,
        message: &FrameAssembledEventListMessage,
        nexus_settings: &NexusSettings,
    ) -> NexusHDF5Result<()> {
        let time_zero = self.contents.lists.get_time_zero(message).err_file()?;
        let origin = self
            .contents
            .lists
            .get_offset()
            .expect("This should never fail.");

        self.contents.logs.push_incomplete_frame_log(
            time_zero,
            message
                .digitizers_present()
                .unwrap_or_default()
                .iter()
                .collect(),
            origin,
            nexus_settings,
        )
    }

    #[tracing::instrument(skip_all, level = "trace", err(level = "warn"))]
    pub(crate) fn push_run_resumed_warning(
        &mut self,
        current_time: &NexusDateTime,
        origin_time: &NexusDateTime,
        nexus_settings: &NexusSettings,
    ) -> NexusHDF5Result<()> {
        self.contents
            .logs
            .push_run_resumed_warning(current_time, origin_time, nexus_settings)?;
        Ok(())
    }

    #[tracing::instrument(skip_all, level = "trace", err(level = "warn"))]
    pub(crate) fn push_aborted_run_warning(
        &mut self,
        stop_time_ms: i64,
        origin_time: &NexusDateTime,
        nexus_settings: &NexusSettings,
    ) -> NexusHDF5Result<()> {
        self.contents
            .logs
            .push_aborted_run_warning(stop_time_ms, origin_time, nexus_settings)?;
        Ok(())
    }

    #[tracing::instrument(skip_all, level = "trace", err(level = "warn"))]
    pub(crate) fn close(self) -> NexusHDF5Result<()> {
        self.file.close().err_file()?;
        Ok(())
    }
}

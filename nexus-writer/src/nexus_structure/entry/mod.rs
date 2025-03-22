use chrono::Utc;
use event_data::EventData;
use hdf5::{types::VarLenUnicode, Dataset, Group};
use instrument::Instrument;
use period::Period;
use runlog::RunLog;
use selog::SELog;

use crate::{
    hdf5_handlers::{DatasetExt, GroupExt, HasAttributesExt, NexusHDF5Result},
    run_engine::{
        run_messages::{
            InitialiseNewNexusRun, InitialiseNewNexusStructure, PushAbortRunWarning, PushAlarm,
            PushFrameEventList, PushIncompleteFrameWarning, PushRunLogData, PushRunResumeWarning,
            PushRunStart, PushRunStop, PushSampleEnvironmentLog, SetEndTime,
        },
        RunParameters, RunStopParameters, DATETIME_FORMAT,
    },
    NexusSettings,
};

use super::{NexusGroup, NexusMessageHandler, NexusSchematic};

mod event_data;
mod instrument;
mod period;
mod runlog;
mod selog;

pub(crate) struct Entry {
    idf_version: Dataset,
    definition: Dataset,
    program_name: Dataset,
    run_number: Dataset,
    experiment_identifier: Dataset,

    start_time: Dataset,
    end_time: Dataset,
    name: Dataset,
    title: Dataset,

    run_logs: NexusGroup<RunLog>,

    instrument: NexusGroup<Instrument>,
    period: NexusGroup<Period>,

    selogs: NexusGroup<SELog>,

    detector_1: NexusGroup<EventData>,
}

impl Entry {
    pub(super) fn extract_run_parameters(&self) -> NexusHDF5Result<RunParameters> {
        let collect_from = self.start_time.get_datetime_from()?;
        let run_name = self.name.get_string_from()?;
        let run_number = self.run_number.get_scalar_from()?;
        let num_periods = self.period.extract(Period::get_number_of_periods)?;
        let instrument_name = self.instrument.extract(Instrument::get_instrument_name)?;
        let run_stop_parameters = self
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
}

impl NexusSchematic for Entry {
    const CLASS: &str = "NXentry";
    type Settings = NexusSettings;

    fn build_group_structure(group: &Group, settings: &NexusSettings) -> NexusHDF5Result<Self> {
        Ok(Self {
            idf_version: group.create_constant_scalar_dataset::<i32>("IDF_version", &2)?,
            definition: group.create_constant_string_dataset("definition", "")?,
            program_name: group.create_scalar_dataset::<VarLenUnicode>("program_name")?,
            run_number: group.create_scalar_dataset::<u32>("run_number")?,
            experiment_identifier: group
                .create_scalar_dataset::<VarLenUnicode>("experiment_identifier")?,
            start_time: group.create_scalar_dataset::<VarLenUnicode>("start_time")?,
            end_time: group.create_scalar_dataset::<VarLenUnicode>("end_time")?,
            name: group.create_constant_string_dataset("name", "")?,
            title: group.create_constant_string_dataset("title", "")?,
            instrument: Instrument::build_new_group(group, "instrument", &())?,
            run_logs: RunLog::build_new_group(group, "runlogs", settings.get_chunk_sizes())?,
            period: Period::build_new_group(group, "period", settings.get_chunk_sizes())?,
            selogs: SELog::build_new_group(group, "selogs", settings.get_chunk_sizes())?,
            detector_1: EventData::build_new_group(
                &group,
                "detector_1",
                settings.get_chunk_sizes(),
            )?,
        })
    }

    fn populate_group_structure(group: &Group) -> NexusHDF5Result<Self> {
        todo!()
    }

    fn close_group() -> NexusHDF5Result<()> {
        todo!()
    }
}

impl NexusMessageHandler<InitialiseNewNexusStructure<'_>> for Entry {
    fn handle_message(
        &mut self,
        InitialiseNewNexusStructure(parameters, nexus_configuration): &InitialiseNewNexusStructure<
            '_,
        >,
    ) -> NexusHDF5Result<()> {
        self.run_number.set_scalar_to(&parameters.run_number)?;

        self.definition.set_string_to("muonTD")?;
        self.experiment_identifier.set_string_to("")?;

        self.program_name
            .set_string_to("SuperMuSR Data Pipeline Nexus Writer")?;
        self.program_name.add_attribute_to("version", "1.0")?;
        self.program_name
            .add_attribute_to("configuration", &nexus_configuration.configuration)?;

        let start_time = parameters.collect_from.format(DATETIME_FORMAT).to_string();

        self.start_time.set_string_to(&start_time)?;
        self.end_time.set_string_to("")?;

        self.name.set_string_to(&parameters.run_name)?;
        self.title.set_string_to("")?;

        self.period
            .handle_message(&InitialiseNewNexusRun(parameters))?;
        self.instrument
            .handle_message(&InitialiseNewNexusRun(parameters))?;
        self.detector_1
            .handle_message(&InitialiseNewNexusRun(parameters))?;
        Ok(())
    }
}

impl NexusMessageHandler<PushRunStart<'_>> for Entry {
    fn handle_message(&mut self, message: &PushRunStart<'_>) -> NexusHDF5Result<()> {
        todo!()
    }
}

impl NexusMessageHandler<PushFrameEventList<'_>> for Entry {
    fn handle_message(&mut self, message: &PushFrameEventList<'_>) -> NexusHDF5Result<()> {
        self.detector_1.handle_message(message)
    }
}

impl NexusMessageHandler<PushRunLogData<'_>> for Entry {
    fn handle_message(&mut self, message: &PushRunLogData<'_>) -> NexusHDF5Result<()> {
        self.run_logs.handle_message(message)
    }
}

impl NexusMessageHandler<PushSampleEnvironmentLog<'_>> for Entry {
    fn handle_message(&mut self, message: &PushSampleEnvironmentLog<'_>) -> NexusHDF5Result<()> {
        self.selogs.handle_message(message)
    }
}

impl NexusMessageHandler<PushAlarm<'_>> for Entry {
    fn handle_message(&mut self, message: &PushAlarm<'_>) -> NexusHDF5Result<()> {
        self.selogs.handle_message(message)
    }
}

impl NexusMessageHandler<PushRunResumeWarning<'_>> for Entry {
    fn handle_message(&mut self, message: &PushRunResumeWarning<'_>) -> NexusHDF5Result<()> {
        self.run_logs.handle_message(message)
    }
}

impl NexusMessageHandler<PushIncompleteFrameWarning<'_>> for Entry {
    fn handle_message(&mut self, message: &PushIncompleteFrameWarning<'_>) -> NexusHDF5Result<()> {
        self.run_logs.handle_message(message)
    }
}

impl NexusMessageHandler<PushAbortRunWarning<'_>> for Entry {
    fn handle_message(&mut self, message: &PushAbortRunWarning<'_>) -> NexusHDF5Result<()> {
        self.run_logs.handle_message(message)
    }
}

impl NexusMessageHandler<PushRunStop<'_>> for Entry {
    fn handle_message(&mut self, message: &PushRunStop<'_>) -> NexusHDF5Result<()> {
        todo!()
    }
}

impl NexusMessageHandler<SetEndTime<'_>> for Entry {
    fn handle_message(&mut self, message: &SetEndTime<'_>) -> NexusHDF5Result<()> {
        todo!()
    }
}

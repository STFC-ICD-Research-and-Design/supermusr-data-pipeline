use chrono::Utc;
use event_data::EventData;
use hdf5::{Dataset, Group};
use instrument::Instrument;
use period::Period;
use runlog::RunLog;
use sample::Sample;
use selog::SELog;
use tracing::warn;

use crate::{
    hdf5_handlers::{DatasetExt, GroupExt, HasAttributesExt, NexusHDF5Result},
    nexus::{NexusClass, DATETIME_FORMAT},
    run_engine::{
        run_messages::{
            InitialiseNewNexusRun, InitialiseNewNexusStructure, PushAlarm, PushFrameEventList,
            PushInternallyGeneratedLogWarning, PushRunLog, PushRunStart, PushSampleEnvironmentLog,
            SetEndTime, UpdatePeriodList,
        },
        RunParameters, RunStopParameters,
    },
    NexusSettings,
};

use super::{NexusGroup, NexusMessageHandler, NexusSchematic};

mod event_data;
mod instrument;
mod period;
mod runlog;
mod sample;
mod selog;

pub(crate) struct Entry {
    _idf_version: Dataset,
    _definition: Dataset,
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
    sample: NexusGroup<Sample>,

    selogs: NexusGroup<SELog>,

    detector_1: NexusGroup<EventData>,
}

impl Entry {
    pub(super) fn extract_run_parameters(&self) -> NexusHDF5Result<RunParameters> {
        let collect_from = self.start_time.get_datetime()?;
        let run_name = self.name.get_string()?;
        let run_stop_parameters = self
            .end_time
            .get_datetime()
            .map(|collect_until| RunStopParameters {
                collect_until,
                last_modified: Utc::now(),
            })
            .ok();

        Ok(RunParameters {
            collect_from,
            run_stop_parameters,
            run_name,
            periods: self.period.extract(Period::extract_periods)?,
        })
    }
}

/// Names of datasets/attribute and subgroups in the Entry struct
mod labels {
    pub(super) const IDF_VERSION: &str = "IDF_version";
    pub(super) const DEFINITION: &str = "definition";
    pub(super) const PROGRAM_NAME: &str = "program_name";
    pub(super) const PROGRAM_NAME_VERSION: &str = "version";
    pub(super) const PROGRAM_NAME_CONFIGURATION: &str = "configuration";
    pub(super) const RUN_NUMBER: &str = "run_number";
    pub(super) const EXPERIMENT_IDENTIFIER: &str = "experiment_identifier";
    pub(super) const START_TIME: &str = "start_time";
    pub(super) const END_TIME: &str = "end_time";
    pub(super) const NAME: &str = "name";
    pub(super) const TITLE: &str = "title";
    pub(super) const INSTRUMENT: &str = "instrument";
    pub(super) const RUNLOGS: &str = "runlogs";
    pub(super) const PERIOD: &str = "period";
    pub(super) const SELOGS: &str = "selogs";
    pub(super) const SAMPLE: &str = "sample";
    pub(super) const DETECTOR_1: &str = "detector_1";
}

// Values of Nexus Constant
const IDF_VERSION: i32 = 2;
const DEFINITION: &str = "muonTD";
const PROGRAM_NAME: &str = "SuperMuSR Data Pipeline Nexus Writer";
const PROGRAM_NAME_VERSION: &str = "1.0";

impl NexusSchematic for Entry {
    const CLASS: NexusClass = NexusClass::Entry;
    type Settings = NexusSettings;

    fn build_group_structure(group: &Group, settings: &NexusSettings) -> NexusHDF5Result<Self> {
        Ok(Self {
            _idf_version: group
                .create_constant_scalar_dataset::<i32>(labels::IDF_VERSION, &IDF_VERSION)?,
            _definition: group.create_constant_string_dataset(labels::DEFINITION, DEFINITION)?,
            program_name: group
                .create_constant_string_dataset(labels::PROGRAM_NAME, PROGRAM_NAME)?
                .with_attribute(labels::PROGRAM_NAME_VERSION, PROGRAM_NAME_VERSION)?,
            run_number: group.create_scalar_dataset::<u32>(labels::RUN_NUMBER)?,
            experiment_identifier: group.create_string_dataset(labels::EXPERIMENT_IDENTIFIER)?,
            start_time: group.create_string_dataset(labels::START_TIME)?,
            end_time: group.create_string_dataset(labels::END_TIME)?,
            name: group.create_constant_string_dataset(labels::NAME, "")?,
            title: group.create_constant_string_dataset(labels::TITLE, "")?,
            instrument: Instrument::build_new_group(group, labels::INSTRUMENT, &())?,
            run_logs: RunLog::build_new_group(group, labels::RUNLOGS, &())?,
            period: Period::build_new_group(group, labels::PERIOD, settings.get_chunk_sizes())?,
            selogs: SELog::build_new_group(group, labels::SELOGS, settings.get_chunk_sizes())?,
            sample: Sample::build_new_group(group, labels::SAMPLE, settings.get_chunk_sizes())?,
            detector_1: EventData::build_new_group(
                group,
                "detector_1",
                &(
                    settings.get_chunk_sizes().event,
                    settings.get_chunk_sizes().frame,
                ),
            )?,
        })
    }

    fn populate_group_structure(group: &Group) -> NexusHDF5Result<Self> {
        let _idf_version = group.get_dataset(labels::IDF_VERSION)?;
        let _definition = group.get_dataset(labels::DEFINITION)?;
        let run_number = group.get_dataset(labels::RUN_NUMBER)?;
        let program_name = group.get_dataset(labels::PROGRAM_NAME)?;
        let experiment_identifier = group.get_dataset(labels::EXPERIMENT_IDENTIFIER)?;

        let start_time = group.get_dataset(labels::START_TIME)?;
        let end_time = group.get_dataset(labels::END_TIME)?;

        let name = group.get_dataset(labels::NAME)?;
        let title = group.get_dataset(labels::TITLE)?;

        let instrument = Instrument::open_group(group, labels::INSTRUMENT)?;
        let period = Period::open_group(group, labels::PERIOD)?;
        let sample = Sample::open_group(group, labels::SAMPLE)?;

        let run_logs = RunLog::open_group(group, labels::RUNLOGS)?;
        let selogs = SELog::open_group(group, labels::SELOGS)?;

        let detector_1 = EventData::open_group(group, labels::DETECTOR_1)?;

        Ok(Self {
            _idf_version,
            start_time,
            end_time,
            name,
            title,
            selogs,
            _definition,
            run_number,
            program_name,
            experiment_identifier,
            run_logs,
            sample,
            instrument,
            period,
            detector_1,
        })
    }
}

/// Helper function to extract the run number from the run name
fn extract_run_number(run_name: &str) -> NexusHDF5Result<u32> {
    // Get Run Number by filtering out any non-integer ascii characters
    let string = run_name
        .chars()
        .filter(char::is_ascii_digit)
        .collect::<String>();

    // If there were no integer characters then return 0
    if string.is_empty() {
        warn!(
            "'Run Number' cannot be determined, defaulting to {}",
            u32::default()
        );
        Ok(u32::default())
    } else {
        Ok(string.parse::<u32>()?)
    }
}

/// Initialise nexus file
impl NexusMessageHandler<InitialiseNewNexusStructure<'_>> for Entry {
    fn handle_message(&mut self, message: &InitialiseNewNexusStructure<'_>) -> NexusHDF5Result<()> {
        let InitialiseNewNexusStructure {
            parameters,
            configuration,
        } = message;

        self.run_number
            .set_scalar(&extract_run_number(&parameters.run_name)?)?;

        self.experiment_identifier.set_string("")?;

        self.program_name.add_attribute(
            labels::PROGRAM_NAME_CONFIGURATION,
            &configuration.configuration,
        )?;

        let start_time = parameters.collect_from.format(DATETIME_FORMAT).to_string();

        self.start_time.set_string(&start_time)?;
        self.end_time.set_string("")?;

        self.name.set_string(&parameters.run_name)?;
        self.title.set_string("")?;

        self.detector_1
            .handle_message(&InitialiseNewNexusRun { parameters })?;
        Ok(())
    }
}

// Direct `PushRunStart` to the group(s) that need it
impl NexusMessageHandler<PushRunStart<'_>> for Entry {
    fn handle_message(&mut self, message: &PushRunStart<'_>) -> NexusHDF5Result<()> {
        self.instrument.handle_message(message)
    }
}

// Direct `PushFrameEventList` to the group(s) that need it
impl NexusMessageHandler<PushFrameEventList<'_>> for Entry {
    fn handle_message(&mut self, message: &PushFrameEventList<'_>) -> NexusHDF5Result<()> {
        self.detector_1.handle_message(message)
    }
}

// Direct `UpdatePeriodList` to the group(s) that need it
impl NexusMessageHandler<UpdatePeriodList<'_>> for Entry {
    fn handle_message(&mut self, message: &UpdatePeriodList<'_>) -> NexusHDF5Result<()> {
        self.period.handle_message(message)
    }
}

// Direct `PushRunLog` to the group(s) that need it
impl NexusMessageHandler<PushRunLog<'_>> for Entry {
    fn handle_message(&mut self, message: &PushRunLog<'_>) -> NexusHDF5Result<()> {
        self.run_logs.handle_message(message)
    }
}

// Direct `PushSampleEnvironmentLog` to the group(s) that need it
impl NexusMessageHandler<PushSampleEnvironmentLog<'_>> for Entry {
    fn handle_message(&mut self, message: &PushSampleEnvironmentLog<'_>) -> NexusHDF5Result<()> {
        self.selogs.handle_message(message)
    }
}

// Direct `PushAlarm` to the group(s) that need it
impl NexusMessageHandler<PushAlarm<'_>> for Entry {
    fn handle_message(&mut self, message: &PushAlarm<'_>) -> NexusHDF5Result<()> {
        self.selogs.handle_message(message)
    }
}

// Direct `PushInternallyGeneratedLogWarning` to the group(s) that need it
impl NexusMessageHandler<PushInternallyGeneratedLogWarning<'_>> for Entry {
    fn handle_message(
        &mut self,
        message: &PushInternallyGeneratedLogWarning<'_>,
    ) -> NexusHDF5Result<()> {
        self.run_logs.handle_message(message)
    }
}

// Set `end_time` field
impl NexusMessageHandler<SetEndTime<'_>> for Entry {
    fn handle_message(&mut self, message: &SetEndTime<'_>) -> NexusHDF5Result<()> {
        let end_time = message.end_time.format(DATETIME_FORMAT).to_string();

        self.end_time.set_string(&end_time)
    }
}

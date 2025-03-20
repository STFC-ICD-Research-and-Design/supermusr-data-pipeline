use event_data::EventData;
use hdf5::{types::VarLenUnicode, Attribute, Dataset, Group, Location};
use instrument::Instrument;
use period::Period;
use runlog::RunLog;
use selog::SELog;

use crate::{hdf5_handlers::{
    ConvertResult, DatasetExt, GroupExt, HasAttributesExt, NexusHDF5Result,
}, NexusSettings};

use super::{NexusGroup, NexusSchematic};

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
            instrument: Instrument::build_new_group(&group, "instrument", &())?,
            run_logs: RunLog::build_new_group(&group, "runlogs", settings.get_chunk_sizes())?,
            period: Period::build_new_group(&group, "period", settings.get_chunk_sizes())?,
            selogs: SELog::build_new_group(&group, "selogs", settings.get_chunk_sizes())?,
            detector_1: EventData::build_new_group(&group, "detector_1", settings.get_chunk_sizes())?,
        })
    }

    fn populate_group_structure(group: &Group) -> NexusHDF5Result<Self> {
        todo!()
    }

    fn close_group() -> NexusHDF5Result<()> {
        todo!()
    }
}

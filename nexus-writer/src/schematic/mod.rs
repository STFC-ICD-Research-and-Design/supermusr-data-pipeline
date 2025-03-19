use hdf5::{types::VarLenUnicode, Attribute, Dataset, Group, Location};
use period::Period;
use runlog::RunLog;
use selog::SELog;
use event_data::EventData;
use instrument::Instrument;

use crate::{hdf5_handlers::{NexusGroup, NexusSchematic}, nexus::{DatasetExt, GroupExt, HasAttributesExt, NexusWriterError}, NexusWriterResult};

mod instrument;
mod event_data;
mod runlog;
mod selog;
mod period;

struct Root {
    hdf5_version: Attribute,
    nexus_version: Attribute,
    file_name: Attribute,
    file_time: Attribute,
    //file.add_attribute_to("HDF5_version", "1.14.3")?; // Can this be taken directly from the nix package;
    //file.add_attribute_to("NeXus_version", "")?; // Where does this come from?
    //file.add_attribute_to("file_name", &file.filename())?; //  This should be absolutized at some point
    //file.add_attribute_to("file_time", Utc::now().to_string().as_str())?; //  This should be formatted, the nanoseconds are overkill.
    raw_data_1: NexusGroup<Entry>
}

impl NexusSchematic for Root {
    const CLASS: &str = "NX_root";
    
    fn build_group_structure(group: &Group) -> NexusWriterResult<Self> {
        Ok(Self {
            hdf5_version: group.add_attribute_to("HDF5_version", "")?,
            nexus_version: group.add_attribute_to("NeXuS_version", "")?,
            file_name: group.add_attribute_to("file_name", "")?,
            file_time: group.add_attribute_to("file_time", "")?,
            raw_data_1: Entry::build_new_group(&group, "root")?
        })
    }

    fn populate_group_structure(group: &Group) -> NexusWriterResult<Self> {
        Ok(Self {
            hdf5_version: group.get_attribute("HDF5_version")?,
            nexus_version: group.get_attribute("NeXuS_version")?,
            file_name: group.get_attribute("file_name")?,
            file_time: group.get_attribute("file_time")?,
            raw_data_1: Entry::open_group(&group, "raw_data_1")?
        })
    }
    
    fn close_group() -> NexusWriterResult<()> {
        Ok(())
    }
}

struct Entry {
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

    fn build_group_structure(group: &Group) -> NexusWriterResult<Self> {
        Ok(Self{
            idf_version: group.create_constant_scalar_dataset::<i32>("IDF_version", &2)?,
            definition: group.create_constant_string_dataset("definition", "")?,
            program_name: group.create_scalar_dataset::<VarLenUnicode>("program_name")?,
            run_number: group.create_scalar_dataset::<u32>("run_number")?,
            experiment_identifier: group.create_scalar_dataset::<VarLenUnicode>("experiment_identifier")?,
            start_time: group.create_scalar_dataset::<VarLenUnicode>("start_time")?,
            end_time: group.create_scalar_dataset::<VarLenUnicode>("end_time")?,
            name: group.create_constant_string_dataset("name", "")?,
            title: group.create_constant_string_dataset("title", "")?,
            instrument: Instrument::build_new_group(&group, "instrument")?,
            run_logs: RunLog::build_new_group(&group, "runlogs")?,
            period: Period::build_new_group(&group, "period")?,
            selogs: SELog::build_new_group(&group, "selogs")?,
            detector_1: EventData::build_new_group(&group, "detector_1")?
        })
    }

    fn populate_group_structure(group: &Group) -> NexusWriterResult<Self> {
        todo!()
    }
    
    fn close_group() -> NexusWriterResult<()> {
        todo!()
    }
}
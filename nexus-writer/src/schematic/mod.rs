use hdf5::{types::VarLenUnicode, Attribute, Dataset, Group, Location};
use period::Period;
use runlog::RunLog;
use selog::SELog;
use event_data::EventData;
use instrument::Instrument;

use crate::{NexusWriterResult, nexus::{HasAttributesExt, DatasetExt, GroupExt}};

mod event_data;
mod runlog;
mod selog;
mod period;
mod instrument;

pub(crate) trait NexusSchematic : Sized {
    const CLASS: &str;

    fn create_and_setup_group(parent: &Group, name: &str) -> NexusHDF5Result<Group> {
        parent.add_new_group_to(name, Self::class)
    }

    fn build_new_group(parent: &Group) -> NexusWriterResult<NexusGroup<Self>>;
    fn open_group(parent: &Group) -> NexusWriterResult<NexusGroup<Self>>;
    fn close_group() -> NexusWriterResult<()>;
}

pub(crate) struct NexusGroup<S : NexusSchematic> {
    group: Group,
    schematic: S,
}

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
    
    fn build_new_group(parent: &Group) -> NexusWriterResult<NexusGroup<Self>> {
        let root = Self::create_and_setup_group(parent, "root")?;

        Ok(NexusGroup::<Self>{
            group: root.clone(),
            schematic: Self {
                hdf5_version: root.add_attribute_to("HDF5_version", "")?,
                nexus_version: root.add_attribute_to("NeXuS_version", "")?,
                file_name: root.add_attribute_to("file_name", "")?,
                file_time: root.add_attribute_to("file_time", "")?,
                raw_data_1: Entry::build_new_group(&root)?
            }
        })
    }

    fn open_group(parent: &Group) -> NexusWriterResult<NexusGroup<Self>> {
        let root = parent.get_group("root")?;

        Ok(NexusGroup::<Self>{
            group: root.clone(),
            schematic: Self {
                hdf5_version: root.get_attribute("HDF5_version")?,
                nexus_version: root.get_attribute("NeXuS_version")?,
                file_name: root.get_attribute("file_name")?,
                file_time: root.get_attribute("file_time")?,
                raw_data_1: Entry::open_group(&root)?
            }
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

    instrument: Instrument,

    run_logs: NexusGroup<RunLog>,

    source: NexusGroup<Source>,
    period: NexusGroup<Period>,

    selogs: NexusGroup<SELog>,

    detector_1: NexusGroup<EventData>,
}


impl NexusSchematic for Entry {
    const CLASS: &str = "NXentry";

    fn build_new_group(parent: &Group) -> NexusWriterResult<NexusGroup<Self>> {
        let group = Self::create_and_setup_group(parent, "raw_data_1")?;

        let schematic = Self {
            idf_version: group.create_constant_scalar_dataset::<i32>("IDF_version", &2)?,
            definition: group.create_constant_string_dataset("definition", "")?,
            program_name: group.create_scalar_dataset::<VarLenUnicode>("program_name")?,
            run_number: group.create_scalar_dataset::<u32>("run_number")?,
            experiment_identifier: group.create_scalar_dataset::<VarLenUnicode>("experiment_identifier")?,
            start_time: group.create_scalar_dataset::<VarLenUnicode>("start_time")?,
            end_time: group.create_scalar_dataset::<VarLenUnicode>("end_time")?,
            name: group.create_constant_string_dataset("name", "")?,
            title: group.create_constant_string_dataset("title", "")?,
            instrument: Instrument::build_new_group(&group)?,
            run_logs: RunLog::build_new_group(&group)?,
            source: Source::build_new_group(&group)?,
            period: Period::build_new_group(&group)?,
            selogs: SELog::build_new_group(&group)?,
            detector_1: EventData::build_new_group(&group)?
        };

        Ok(NexusGroup::<Self> {
            group,
            schematic
        })
    }

    fn open_group(parent: &Group) -> NexusWriterResult<NexusGroup<Self>> {
        todo!()
    }
    
    fn close_group() -> NexusWriterResult<()> {
        todo!()
    }
}
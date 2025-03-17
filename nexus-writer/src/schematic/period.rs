use super::NexusSchematic;

pub(crate) struct Period {
    
}

impl NexusSchematic for Period {
    const CLASS: &str = "NXperiod";

    fn build_new_group(parent: &hdf5::Group) -> crate::NexusWriterResult<super::NexusGroup<Self>> {
        todo!()
    }

    fn open_group(parent: &hdf5::Group) -> crate::NexusWriterResult<super::NexusGroup<Self>> {
        todo!()
    }

    fn close_group() -> crate::NexusWriterResult<()> {
        todo!()
    }
}
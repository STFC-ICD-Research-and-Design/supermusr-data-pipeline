use super::NexusSchematic;

pub(crate) struct RunLog {
    
}

impl NexusSchematic for RunLog {
    const CLASS: &str = "NXrunlog";

    fn build_new_group(this: hdf5::Group) -> crate::NexusWriterResult<super::NexusGroup<Self>> {
        todo!()
    }

    fn open_group(parent: hdf5::Group) -> crate::NexusWriterResult<super::NexusGroup<Self>> {
        todo!()
    }

    fn close_group(parent: hdf5::Group) -> crate::NexusWriterResult<()> {
        todo!()
    }
}
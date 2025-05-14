//! Defines [Period] group structure which contains data specifying the periods used in the run.
use crate::{
    hdf5_handlers::{AttributeExt, DatasetExt, GroupExt, HasAttributesExt, NexusHDF5Result},
    nexus::NexusClass,
    nexus_structure::{NexusMessageHandler, NexusSchematic},
    run_engine::{run_messages::UpdatePeriodList, PeriodChunkSize},
};
use hdf5::{Dataset, Group};

/// Field names for [Period].
mod labels {
    pub(super) const NUMBER: &str = "number";
    pub(super) const PERIOD_TYPE: &str = "type";
    pub(super) const LABELS: &str = "labels";
    pub(super) const LABELS_SEPARATOR: &str = "separator";
}

// Values of Nexus Constant
/// The character used to separate the period labels.
const LABELS_SEPARATOR: &str = ",";

/// Names of datasets/attribute and subgroups in the Entry struct
pub(crate) struct Period {
    number: Dataset,
    peroid_type: Dataset,
    labels: Dataset,
}

impl Period {
    /// As periods are stored directly in the [RunParameters] object, this method extracts
    /// a vector of periods from an existing NeXus file.
    /// # Return
    /// A vector of periods.
    /// # Error Modes
    /// - Propagates errors from [Dataset::get_attribute()].
    /// - Propagates [ParseIntError] errors.
    ///
    /// [ParseIntError]: std::num::ParseIntError
    /// [RunParameters]: crate::run_engine::RunParameters
    pub(super) fn extract_periods(&self) -> NexusHDF5Result<Vec<u64>> {
        let separator = self
            .labels
            .get_attribute(labels::LABELS_SEPARATOR)?
            .get_string()?;
        self.labels
            .get_string()?
            .split(&separator)
            .map(str::parse)
            .collect::<Result<_, _>>()
            .map_err(Into::into)
    }
}

impl NexusSchematic for Period {
    /// The nexus class of this group.
    const CLASS: NexusClass = NexusClass::Period;

    /// This group structure only needs the appropriate chunk size.
    type Settings = PeriodChunkSize;

    fn build_group_structure(group: &Group, settings: &Self::Settings) -> NexusHDF5Result<Self> {
        Ok(Self {
            number: group.create_scalar_dataset::<u32>(labels::NUMBER)?,
            peroid_type: group
                .create_resizable_empty_dataset::<u32>(labels::PERIOD_TYPE, *settings)?,
            labels: group
                .create_string_dataset(labels::LABELS)?
                .with_constant_string_attribute(labels::LABELS_SEPARATOR, LABELS_SEPARATOR)?,
        })
    }

    fn populate_group_structure(group: &Group) -> NexusHDF5Result<Self> {
        Ok(Self {
            number: group.get_dataset(labels::NUMBER)?,
            peroid_type: group.get_dataset(labels::PERIOD_TYPE)?,
            labels: group.get_dataset(labels::LABELS)?,
        })
    }
}

impl NexusMessageHandler<UpdatePeriodList<'_>> for Period {
    /// Causes the periods dataset to be rewritten from the provided period list.
    /// # Return
    /// A vector of periods.
    /// # Error Modes
    /// - Propagates errors from [Dataset::set_scalar()].
    /// - Propagates errors from [Dataset::set_slice()].
    /// - Propagates errors from [Dataset::get_attribute()].
    /// - Propagates errors from [Attribute::get_string()].
    /// - Propagates errors from [Dataset::set_string()].
    /// - Propagates [ParseIntError] errors.
    ///
    /// [ParseIntError]: std::num::ParseIntError
    /// [Attribute::get_string()]: crate::hdf5_handlers::AttributeExt::get_string()
    fn handle_message(
        &mut self,
        UpdatePeriodList { periods }: &UpdatePeriodList<'_>,
    ) -> NexusHDF5Result<()> {
        self.number.set_scalar(&periods.len())?;
        let mut peroid_type = Vec::new();
        peroid_type.resize(periods.len(), 1);
        self.peroid_type.set_slice(&peroid_type)?;
        let separator = self
            .labels
            .get_attribute(labels::LABELS_SEPARATOR)?
            .get_string()?;
        let labels = periods
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(&separator);
        self.labels.set_string(&labels)
    }
}

use super::partial::PartialFrame;
use crate::data::{Accumulate, DigitiserData};
#[cfg(test)]
use supermusr_common::DigitizerId;
use supermusr_streaming_types::FrameMetadata;

pub(crate) struct AggregatedFrame<D> {
    pub(crate) metadata: FrameMetadata,
    #[cfg(test)]
    pub(crate) digitiser_ids: Vec<DigitizerId>,
    pub(crate) digitiser_data: D,
}

impl<D> From<PartialFrame<D>> for AggregatedFrame<D>
where
    DigitiserData<D>: Accumulate<D>,
{
    fn from(mut partial: PartialFrame<D>) -> Self {
        Self {
            metadata: partial.metadata.clone(),
            #[cfg(test)]
            digitiser_ids: partial.digitiser_ids(),
            digitiser_data: <DigitiserData<D> as Accumulate<D>>::accumulate(
                &mut partial.digitiser_data,
            ),
        }
    }
}

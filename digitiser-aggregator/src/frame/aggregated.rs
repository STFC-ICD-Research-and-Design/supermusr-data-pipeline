use super::partial::{PartialFrame, PartialFrameLike};
use crate::data::{Accumulate, DigitiserData};
use std::fmt::Debug;
#[cfg(test)]
use supermusr_common::DigitizerId;
use supermusr_streaming_types::FrameMetadata;

pub(crate) trait AggregatedFrameLike<D, P: PartialFrameLike<D>>:
    Debug + AsRef<AggregatedFrame<D>> + AsMut<AggregatedFrame<D>> + From<P>
{
}

impl<D> AsRef<Self> for AggregatedFrame<D> {
    fn as_ref(&self) -> &Self {
        self
    }
}
impl<D> AsMut<Self> for AggregatedFrame<D> {
    fn as_mut(&mut self) -> &mut Self {
        self
    }
}

impl<D: Debug> AggregatedFrameLike<D, PartialFrame<D>> for AggregatedFrame<D> where
    Vec<(u8, D)>: Accumulate<D>
{
}

#[derive(Debug)]
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

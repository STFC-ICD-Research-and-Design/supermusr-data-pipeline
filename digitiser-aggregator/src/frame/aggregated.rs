use super::partial::PartialFrame;
use crate::data::{Accumulate, DigitiserData};
use supermusr_common::{
    DigitizerId,
    spanned::{SpanOnce, Spanned, SpannedMut},
};
use supermusr_streaming_types::FrameMetadata;

pub(crate) struct AggregatedFrame<D> {
    span: SpanOnce,
    pub(crate) metadata: FrameMetadata,
    pub(crate) complete: bool,
    pub(crate) digitiser_ids: Vec<DigitizerId>,
    pub(crate) digitiser_data: D,
}

#[cfg(test)]
impl<D> AggregatedFrame<D> {
    pub(crate) fn new(
        metadata: FrameMetadata,
        complete: bool,
        digitiser_ids: Vec<DigitizerId>,
        digitiser_data: D,
    ) -> Self {
        Self {
            span: Default::default(),
            metadata,
            complete,
            digitiser_ids,
            digitiser_data,
        }
    }
}

impl<D> From<PartialFrame<D>> for AggregatedFrame<D>
where
    DigitiserData<D>: Accumulate<D>,
{
    fn from(mut partial: PartialFrame<D>) -> Self {
        Self {
            span: partial
                .span_mut()
                .take()
                .expect("partial frame should have a span"),
            metadata: partial.metadata.clone(),
            complete: partial.is_complete(),
            digitiser_ids: partial.digitiser_ids(),
            digitiser_data: <DigitiserData<D> as Accumulate<D>>::accumulate(
                &mut partial.digitiser_data,
            ),
        }
    }
}

impl<D> Spanned for AggregatedFrame<D> {
    fn span(&self) -> &SpanOnce {
        &self.span
    }
}

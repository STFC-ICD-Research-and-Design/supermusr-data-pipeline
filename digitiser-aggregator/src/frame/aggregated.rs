use super::partial::PartialFrame;
use crate::data::{Accumulate, DigitiserData};
use std::fmt::Debug;
use supermusr_common::spanned::{SpanOnce, Spanned};
#[cfg(test)]
use supermusr_common::DigitizerId;
use supermusr_streaming_types::FrameMetadata;
use tracing::Span;

pub(crate) struct AggregatedFrame<D> {
    span: SpanOnce,
    pub(crate) metadata: FrameMetadata,
    #[cfg(test)]
    pub(crate) digitiser_ids: Vec<DigitizerId>,
    pub(crate) digitiser_data: D,
}

#[cfg(test)]
impl<D> AggregatedFrame<D> {
    pub(crate) fn new(
        metadata: FrameMetadata,
        #[cfg(test)]
        digitiser_ids: Vec<DigitizerId>,
        digitiser_data: D,
    ) -> Self {
        Self {
            span: Default::default(),
            metadata,
            #[cfg(test)]digitiser_ids,
            digitiser_data
        }
    }
}

impl<D> From<PartialFrame<D>> for AggregatedFrame<D>
where
    DigitiserData<D>: Accumulate<D>,
{
    fn from(mut partial: PartialFrame<D>) -> Self {
        Self {
            span: partial.inherit_span(),
            metadata: partial.metadata.clone(),
            #[cfg(test)]
            digitiser_ids: partial.digitiser_ids(),
            digitiser_data: <DigitiserData<D> as Accumulate<D>>::accumulate(
                &mut partial.digitiser_data,
            ),
        }
    }
}

impl<D> Spanned for AggregatedFrame<D> {
    fn init_span(&mut self, span: Span) {
        self.span = match self.span {
            SpanOnce::Waiting => SpanOnce::Spanned(span),
            _ => panic!(),
        };
    }

    fn get_span(&self) -> &Span {
        match &self.span {
            SpanOnce::Spanned(span) => span,
            _ => panic!(),
        }
    }

    fn inherit_span(&mut self) -> SpanOnce {
        let span = match &mut self.span {
            SpanOnce::Spanned(span) => span.clone(),
            _ => panic!(),
        };
        self.span = SpanOnce::Spent;
        SpanOnce::Spanned(span)
    }
}
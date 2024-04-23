use std::{fmt::Debug, time::Duration};
use supermusr_common::tracer::Spanned;
use supermusr_streaming_types::FrameMetadata;
use tracing::{trace_span, Span};

use crate::{
    data::Accumulate,
    frame::{AggregatedFrame, AggregatedFrameLike, FrameCache, PartialFrame, PartialFrameLike},
};

pub(crate) type SpannedPartialFrame<D> = Spanned<PartialFrame<D>>;

impl<D: Debug> PartialFrameLike<D> for SpannedPartialFrame<D> {
    fn new(ttl: Duration, metadata: FrameMetadata) -> Self {
        let span = trace_span!("Frame");
        Spanned {
            span,
            value: PartialFrame::<D>::new(ttl, metadata),
        }
    }
}

pub(crate) struct SpannedAggregatedFrame<D> {
    pub(crate) span: Span,
    pub(crate) value: AggregatedFrame<D>,
}

impl<D: Debug> Debug for SpannedAggregatedFrame<D> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.value.fmt(f)
    }
}

impl<D> AsRef<AggregatedFrame<D>> for SpannedAggregatedFrame<D> {
    fn as_ref(&self) -> &AggregatedFrame<D> {
        &self.value
    }
}

impl<D> AsMut<AggregatedFrame<D>> for SpannedAggregatedFrame<D> {
    fn as_mut(&mut self) -> &mut AggregatedFrame<D> {
        &mut self.value
    }
}

impl<D: Debug> AggregatedFrameLike<D, SpannedPartialFrame<D>> for SpannedAggregatedFrame<D> where
    Vec<(u8, D)>: Accumulate<D>
{
}

impl<D: Debug> From<SpannedPartialFrame<D>> for SpannedAggregatedFrame<D>
where
    Vec<(u8, D)>: Accumulate<D>,
{
    fn from(pf: SpannedPartialFrame<D>) -> Self {
        SpannedAggregatedFrame {
            span: pf.span,
            value: pf.value.into(),
        }
    }
}

pub(crate) type SpannedFrameCache<D> =
    FrameCache<D, SpannedPartialFrame<D>, SpannedAggregatedFrame<D>>;

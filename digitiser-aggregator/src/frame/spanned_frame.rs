use std::{fmt::Debug, ops::{Deref, DerefMut}, time::Duration};
use supermusr_common::{spanned::{SpanAgnostic, SpanWrappable, SpanWrapper}, Spannify};
use supermusr_streaming_types::FrameMetadata;
use tracing::{trace_span, Span};

use crate::{
    data::Accumulate,
    frame::{AggregatedFrame, FrameCache, PartialFrame},
};

impl<D : Debug> SpanWrappable for PartialFrame<D> {}
impl<D : Debug> SpanWrappable for AggregatedFrame<D> {}

// Partial Frame
pub(crate) trait PartialFrameLike<D : Debug> : SpanAgnostic<PartialFrame<D>> {
    fn new(ttl: Duration, metadata: FrameMetadata) -> Self;
}

Spannify!(pub(crate), SpannedPartialFrame<D>,PartialFrame<D>);

pub(crate) struct SpannedPartialFrame<D> {
    span: Span,
    value: PartialFrame<D>
}

impl<D: Debug> Deref for SpannedPartialFrame<D> {
    type Target = PartialFrame<D>;

    fn deref(&self) -> &PartialFrame<D> {
        &self.value
    }
}

impl<D: Debug> DerefMut for SpannedPartialFrame<D> {
    fn deref_mut(&mut self) -> &mut PartialFrame<D> {
        &mut self.value
    }
}

impl<D: Debug> SpanAgnostic<PartialFrame<D>> for SpannedPartialFrame<D> {}

impl<D: Debug> From<PartialFrame<D>> for SpannedPartialFrame<D> {
    fn from(value : PartialFrame<D>) -> Self {
        let span = trace_span!("Frame");
        SpannedPartialFrame { span, value }
    }
}
/*
impl<D: Debug> Into<PartialFrame<D>> for SpannedPartialFrame<D> {
    fn into(self) -> PartialFrame<D> {
        self.value
    }
}
 */
impl<D: Debug> SpanWrapper<PartialFrame<D>> for SpannedPartialFrame<D> {
    fn span(&self) -> &Span {
        &self.span
    }
}


impl<D: Debug> PartialFrameLike<D> for SpannedPartialFrame<D> {
    fn new(ttl: Duration, metadata: FrameMetadata) -> Self {
        let span = trace_span!("Frame");
        let value = PartialFrame::<D>::new(ttl, metadata);
        SpannedPartialFrame { span, value }
    }
}



impl<D: Debug> Deref for PartialFrame<D> {
    type Target = PartialFrame<D>;

    fn deref(&self) -> &PartialFrame<D> {
        self
    }
}

impl<D: Debug> DerefMut for PartialFrame<D> {
    fn deref_mut(&mut self) -> &mut PartialFrame<D> {
        self
    }
}

impl<D: Debug> SpanAgnostic<PartialFrame<D>> for PartialFrame<D> {}

impl<D: Debug> PartialFrameLike<D> for PartialFrame<D> {
    fn new(ttl: Duration, metadata: FrameMetadata) -> Self {
        PartialFrame::<D>::new(ttl,metadata)
    }
}


// Aggregated Frame



pub(crate) trait AggregatedFrameLike<D : Debug, P: PartialFrameLike<D>> : SpanAgnostic<AggregatedFrame<D>> + From<P> {}

pub(crate) struct SpannedAggregatedFrame<D> {
    pub(crate) span: Span,
    pub(crate) value: AggregatedFrame<D>,
}

impl<D: Debug> Debug for SpannedAggregatedFrame<D> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self.value, f)
    }
}

impl<D: Debug> Deref for SpannedAggregatedFrame<D> {
    type Target = AggregatedFrame<D>;

    fn deref(&self) -> &AggregatedFrame<D> {
        &self.value
    }
}

impl<D: Debug> DerefMut for SpannedAggregatedFrame<D> {
    fn deref_mut(&mut self) -> &mut AggregatedFrame<D> {
        &mut self.value
    }
}

impl<D: Debug> SpanAgnostic<AggregatedFrame<D>> for SpannedAggregatedFrame<D> {}


impl<D: Debug> Deref for AggregatedFrame<D> {
    type Target = AggregatedFrame<D>;

    fn deref(&self) -> &AggregatedFrame<D> {
        self
    }
}

impl<D: Debug> DerefMut for AggregatedFrame<D> {
    fn deref_mut(&mut self) -> &mut AggregatedFrame<D> {
        self
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
            value: Into::<_>::into(pf.value),
        }
    }
}

pub(crate) type SpannedFrameCache<D> =
    FrameCache<D, SpannedPartialFrame<D>, SpannedAggregatedFrame<D>>;

use std::{fmt::Debug, ops::{Deref, DerefMut}, time::Duration};
use supermusr_common::spanned::{SpanOnce, Spanned};
use supermusr_streaming_types::FrameMetadata;
use tracing::{trace_span, Span};

use crate::{
    data::Accumulate,
    frame::{AggregatedFrame, FrameCache, PartialFrame},
};
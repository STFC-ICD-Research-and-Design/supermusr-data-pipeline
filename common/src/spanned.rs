use std::{
    fmt::Debug,
    ops::{Deref, DerefMut},
};
use thiserror::Error;
use tracing::Span;

#[derive(Error, Debug)]
pub enum SpanOnceError {
    #[error("Attempt to initialise twice")]
    AlreadyInit,
    #[error("Attempt to initialise a spent span")]
    SpentInit,
    #[error("Attempt to read an uninitialised span")]
    UninitialisedRead,
    #[error("Attempt to read a spent span")]
    SpentRead,
    #[error("Attempt to inherit from an uninitialised span")]
    UninitialisedInherit,
    #[error("Attempt to inherit from a spent span")]
    SpentInherit,
}

/// A wrapper for use by types implementing the Spanned and SpannedMut trait.
/// This type can only be set once, read immutably, and inherited by a new
/// uninitialised SpanOnce.
#[derive(Default)]
pub enum SpanOnce {
    #[default]
    Waiting,
    Spanned(Span),
    Spent,
}

impl SpanOnce {
    pub fn init(&mut self, span: Span) -> Result<(), SpanOnceError> {
        *self = match self {
            SpanOnce::Waiting => SpanOnce::Spanned(span),
            SpanOnce::Spanned(_) => return Err(SpanOnceError::AlreadyInit),
            SpanOnce::Spent => return Err(SpanOnceError::SpentInit),
        };
        Ok(())
    }

    pub fn get(&self) -> Result<&Span, SpanOnceError> {
        match &self {
            SpanOnce::Spanned(span) => Ok(span),
            SpanOnce::Waiting => Err(SpanOnceError::UninitialisedRead),
            SpanOnce::Spent => Err(SpanOnceError::SpentRead),
        }
    }

    pub fn inherit(&mut self) -> Result<SpanOnce, SpanOnceError> {
        let span = match &self {
            SpanOnce::Spanned(span) => span.clone(),
            SpanOnce::Waiting => return Err(SpanOnceError::UninitialisedInherit),
            SpanOnce::Spent => return Err(SpanOnceError::SpentInherit),
        };
        *self = SpanOnce::Spent;
        Ok(SpanOnce::Spanned(span))
    }
}

/// Types which have a span: SpanOnce field implement one or both of
/// Spanned or SpannedMut. Their purpose is to return references
/// to the SpanOnce field. For instance:
/// ```rust
/// # struct SpanOnce;
/// # struct Foo { span: SpanOnce }
/// # pub trait Spanned { fn span(&self) -> &SpanOnce; }
/// # pub trait SpannedMut: Spanned { fn span_mut(&mut self) -> &mut SpanOnce; }
///
/// impl Spanned for Foo {
///     fn span(&self) -> &SpanOnce {
///         &self.span
///     }
/// }
///
/// impl SpannedMut for Foo {
///     fn span_mut(&mut self) -> &mut SpanOnce {
///         &mut self.span
///     }
/// }
/// ```
pub trait Spanned {
    fn span(&self) -> &SpanOnce;
}

pub trait SpannedMut: Spanned {
    fn span_mut(&mut self) -> &mut SpanOnce;
}

/// This generic type wraps a type and associates a SpanOnce with it.
/// This is an alternative to implementing Spanned or SpannedMut for the type.
/// Note this does not currently implement SpannedMut but could be easily
/// extended in the future.
pub struct SpanWrapper<T> {
    span: SpanOnce,
    value: T,
}

impl<T> Spanned for SpanWrapper<T> {
    fn span(&self) -> &SpanOnce {
        &self.span
    }
}

impl<T: Default> SpanWrapper<T> {
    pub fn default_with_span(span: Span) -> Self {
        Self {
            span: SpanOnce::Spanned(span),
            value: Default::default(),
        }
    }
}

impl<T> SpanWrapper<T> {
    pub fn new(span: Span, value: T) -> Self {
        Self {
            span: SpanOnce::Spanned(span),
            value,
        }
    }

    pub fn new_with_current(value: T) -> Self {
        Self {
            span: SpanOnce::Spanned(tracing::Span::current()),
            value,
        }
    }
}

impl<T> Deref for SpanWrapper<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.value
    }
}

impl<T> DerefMut for SpanWrapper<T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.value
    }
}

impl<T: Debug> Debug for SpanWrapper<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.value.fmt(f)
    }
}

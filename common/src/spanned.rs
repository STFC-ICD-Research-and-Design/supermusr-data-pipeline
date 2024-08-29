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
    #[error("Attempt to take from a spent span")]
    SpentTake,
}

/// A wrapper for use by types implementing the Spanned and SpannedMut trait.
/// This type can only be set once, read immutably, and inherited by a new
/// uninitialised SpanOnce.
#[derive(Default, PartialEq)]
pub enum SpanOnce {
    #[default]
    Waiting,
    Spanned(Span),
    Spent,
}

impl SpanOnce {
    pub fn is_waiting(&self) -> bool {
        matches!(self, Self::Waiting)
    }

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

    pub fn take(&mut self) -> Result<SpanOnce, SpanOnceError> {
        let span = match &self {
            SpanOnce::Spanned(span) => span.clone(),
            SpanOnce::Waiting => return Ok(SpanOnce::Waiting),
            SpanOnce::Spent => return Err(SpanOnceError::SpentTake),
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

/// Types which have a span: SpanOnce field may implement this trait which
/// is intended to encapsulate span-aggregating behaviour.
pub trait SpannedAggregator: SpannedMut {
    fn span_init(&mut self) -> Result<(), SpanOnceError>;

    fn link_current_span<F: Fn() -> Span>(
        &self,
        aggregated_span_fn: F,
    ) -> Result<(), SpanOnceError>;

    fn end_span(&self) -> Result<(), SpanOnceError>;
}

/// Types which contain a collection of Spanned types may implement these traits which
/// provide methods for finding the associated spans of the Spanned objects
pub trait FindSpan<T: SpannedAggregator + 'static> {
    type Key: PartialEq;

    fn find_span(&self, _key: Self::Key) -> Option<&impl SpannedAggregator> {
        Option::<T>::None.as_ref()
    }
}

pub trait FindSpanMut<T: SpannedAggregator + 'static>: FindSpan<T> {
    fn find_span_mut(
        &mut self,
        _key: <Self as FindSpan<T>>::Key,
    ) -> Option<&mut impl SpannedAggregator> {
        Option::<&mut T>::None
    }
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

#[cfg(test)]
mod test {
    use super::*;
    use tracing;

    /// Tests that SpanOnce can be initialised
    #[test]
    fn test_init() {
        let mut span = SpanOnce::default();
        assert!(span.init(tracing::Span::current()).is_ok());
    }

    /// Tests that SpanOnce cannot be initialised twice
    #[test]
    fn test_init_twice_fail() {
        let mut span = SpanOnce::default();
        span.init(tracing::Span::current()).unwrap();
        let result = span.init(tracing::Span::current());
        assert!(matches!(result, Err(SpanOnceError::AlreadyInit)));
    }

    /// Tests that SpanOnce can be read once initialised
    #[test]
    fn test_read() {
        let mut span = SpanOnce::default();
        span.init(tracing::Span::current()).unwrap();
        assert!(span.get().is_ok());
    }

    /// Tests that SpanOnce cannot be read if not initialised
    #[test]
    fn test_uninit_read_fail() {
        let span = SpanOnce::default();
        let result = span.get();
        assert!(matches!(result, Err(SpanOnceError::UninitialisedRead)));
    }

    /// Tests that SpanOnce can be taken if initialised
    #[test]
    fn test_take() {
        let mut span = SpanOnce::default();
        span.init(tracing::Span::current()).unwrap();
        assert!(span.take().is_ok());
    }

    /// Tests that SpanOnce can be taken if not initialised
    #[test]
    fn test_uninit_take_fail() {
        let mut span = SpanOnce::default();
        let result = span.take();
        assert!(result.is_ok());
    }

    /// Tests that SpanOnce can be taken if initialised and after being read from
    #[test]
    fn test_take_after_read() {
        let mut span = SpanOnce::default();
        span.init(tracing::Span::current()).unwrap();
        span.get().unwrap();
        assert!(span.take().is_ok());
    }

    /// Tests that SpanOnce cannot be initialised if it has been taken
    #[test]
    fn test_init_after_take_fail() {
        let mut span = SpanOnce::default();
        span.init(tracing::Span::current()).unwrap();
        span.take().unwrap();
        let result = span.init(tracing::Span::current());
        assert!(matches!(result, Err(SpanOnceError::SpentInit)));
    }

    /// Tests that SpanOnce cannot be read from if it has been taken
    #[test]
    fn test_read_after_inherit_fail() {
        let mut span = SpanOnce::default();
        span.init(tracing::Span::current()).unwrap();
        span.take().unwrap();
        let result = span.get();
        assert!(matches!(result, Err(SpanOnceError::SpentRead)));
    }

    /// Tests that SpanOnce cannot be taken twice
    #[test]
    fn test_inherit_twice_fail() {
        let mut span = SpanOnce::default();
        span.init(tracing::Span::current()).unwrap();
        span.take().unwrap();
        let result = span.take();
        assert!(matches!(result, Err(SpanOnceError::SpentTake)));
    }
}

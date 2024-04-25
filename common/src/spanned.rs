use std::{fmt::Debug, ops::{Deref, DerefMut}};
use tracing::Span;

#[derive(Default)]
pub enum SpanOnce {
    #[default]
    Waiting,
    Spanned(Span),
    Spent,
}

impl SpanOnce {
    pub fn init(&mut self, span: Span) {
        *self = match self {
            SpanOnce::Waiting => SpanOnce::Spanned(span),
            _ => panic!(),
        }
    }
    pub fn get(&self) -> &Span {
        match &self {
            SpanOnce::Spanned(span) => span,
            _ => panic!(),
        }
    }
    pub fn inherit(&mut self) -> SpanOnce {
        let span = match &self {
            SpanOnce::Spanned(span) => span.clone(),
            _ => panic!(),
        };
        *self = SpanOnce::Spent;
        SpanOnce::Spanned(span)
    }
}
pub trait Spanned {
    fn span(&self) -> &SpanOnce;
}
pub trait SpannedMut : Spanned {
    fn span_mut(&mut self) -> &mut SpanOnce;
}

/// This is a wrapper for a type which can be bundled with a span.
/// Given type Foo, define trait FooLike in the following fashion:
/// ```rust
/// # #[derive(Debug)] struct Foo;
/// # impl AsMut<Foo> for Foo { fn as_mut(&mut self) -> &mut Foo { self } }
/// # impl AsRef<Foo> for Foo { fn as_ref(&self) -> &Foo { self } }
/// trait FooLike : std::fmt::Debug + AsRef<Foo> + AsMut<Foo> {
///     fn new(/* ... */) -> Self where Self: Sized;
/// }
/// // and implement for both Foo and Spanned<Foo>, that is:
/// impl FooLike for Foo {
///     fn new(/* ... */) -> Foo {
///         # unreachable!()
///         /* ... */
///     }
/// }
/// // and
/// # use supermusr_common::tracer::Spanned;
/// impl FooLike for Spanned<Foo> {
///     fn new(/* ... */) -> Spanned<Foo> {
///         # unreachable!()
///         /* ... */
///     }
/// }
/// ```
/// Now any function or struct that uses Foo, can use a generic that implements FooType instead.
/// For instance
/// ```rust
/// # struct Foo; impl Foo { fn some_foo(&self) {} }
/// struct Bar {
///     foo : Foo,
/// }
/// impl Bar {
///     fn do_some_foo(&self) {
///         self.foo.some_foo()
///     }
/// }
/// ```
/// becomes:
/// ```rust
/// # #[derive(Debug)] struct Foo; impl Foo { fn some_foo(&self) {} }
/// # impl AsMut<Foo> for Foo { fn as_mut(&mut self) -> &mut Foo { self } }
/// # impl AsRef<Foo> for Foo { fn as_ref(&self) -> &Foo { self } }
/// trait FooLike : std::fmt::Debug + AsRef<Foo> + AsMut<Foo> {
///     fn new(/* ... */) -> Self where Self: Sized;
/// }
/// struct Bar<F : FooLike> {
///     foo : F,
/// }
/// impl<F : FooLike> Bar<F> {
///     fn do_some_foo(&self) {
///         self.foo.as_ref().some_foo()
///     }
/// }
/// ```
/// So now Foo and Spanned<Foo> can be switched out easily,
/// by using either `Bar<Foo>` or `Bar<Spanned<Foo>>`.
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
        Self { span: SpanOnce::Spanned(span), value }
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
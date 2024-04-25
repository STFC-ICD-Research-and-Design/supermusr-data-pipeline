use std::{fmt::Debug, ops::{Deref, DerefMut}};
use tracing::Span;

#[macro_export]
macro_rules! Spannify {
    ($v: vis, $spanned_t:ident, $t:ident) => {
        $v struct $spanned_t {
            span : Span,
            value : $t
        }

        impl<D: Debug> Deref for $spanned_t {
            type Target = $t;
        
            fn deref(&self) -> &Self::Target {
                &self.value
            }
        }
        
        impl<D: Debug> DerefMut for $spanned_t {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.value
            }
        }
    };
}

pub trait SpanAgnostic<T : SpanWrappable> : Deref<Target = T> + DerefMut {}

pub trait SpanWrapper<T : SpanWrappable> : SpanAgnostic<T> {
    fn span(&self) -> &Span;
}

pub trait SpanWrappable : Debug {

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
pub struct Spanned<T> {
    pub span: Span,
    pub value: T,
}

impl<T: Default> Spanned<T> {
    pub fn default_with_span(span: Span) -> Self {
        Self {
            span,
            value: Default::default(),
        }
    }
}

impl<T> Spanned<T> {
    pub fn new(span: Span, value: T) -> Self {
        Self { span, value }
    }

    pub fn new_with_current(value: T) -> Self {
        Self {
            span: tracing::Span::current(),
            value,
        }
    }
}

impl<T: Debug> Debug for Spanned<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.value.fmt(f)
    }
}

impl<T> AsRef<T> for Spanned<T> {
    fn as_ref(&self) -> &T {
        &self.value
    }
}

impl<T> AsMut<T> for Spanned<T> {
    fn as_mut(&mut self) -> &mut T {
        &mut self.value
    }
}

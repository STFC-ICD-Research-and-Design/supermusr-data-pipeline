use opentelemetry::{
    propagation::{Extractor, Injector},
    trace::{TraceContextExt, TraceError},
};
use opentelemetry_otlp::WithExportConfig;
use rdkafka::message::{BorrowedHeaders, Headers, OwnedHeaders};
use std::fmt::Debug;
use tracing::Span;
use tracing_opentelemetry::{self, OpenTelemetrySpanExt};
use tracing_subscriber::layer::SubscriberExt;

/// Create this object to initialise the Open Telemetry Tracer
pub struct OtelTracer;

impl OtelTracer {
    pub fn new(endpoint: &str, service_name: &str) -> Result<Self, TraceError> {
        let otlp_exporter = opentelemetry_otlp::new_exporter()
            .tonic()
            .with_endpoint(endpoint);

        let otlp_resource = opentelemetry_sdk::Resource::new(vec![opentelemetry::KeyValue::new(
            "service.name",
            service_name.to_owned(),
        )]);

        let otlp_config = opentelemetry_sdk::trace::Config::default().with_resource(otlp_resource);

        let tracer = opentelemetry_otlp::new_pipeline()
            .tracing()
            .with_trace_config(otlp_config)
            .with_exporter(otlp_exporter)
            .install_batch(opentelemetry_sdk::runtime::Tokio)?;

        opentelemetry::global::set_text_map_propagator(
            opentelemetry_sdk::propagation::TraceContextPropagator::new(),
        );
        let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);
        let subscriber = tracing_subscriber::Registry::default().with(telemetry);
        tracing::subscriber::set_global_default(subscriber).unwrap();
        Ok(Self)
    }

    /// Extracts the open telementry context from the given kafka headers and sets the given span's parent to it
    pub fn extract_context_from_kafka_to_span(headers: &BorrowedHeaders, span: &Span) {
        span.set_parent(opentelemetry::global::get_text_map_propagator(
            |propagator| propagator.extract(&HeaderExtractor(headers)),
        ));
    }

    /// Injects the open telemetry context into the given kafka headers
    pub fn inject_context_from_span_into_kafka(parent_span: &Span, headers: &mut OwnedHeaders) {
        opentelemetry::global::get_text_map_propagator(|propagator| {
            propagator.inject_context(&parent_span.context(), &mut HeaderInjector(headers))
        });
    }

    /// Creates a link from span to other_span
    pub fn link_span_to_span(span: &Span, other_span: &Span) {
        span.add_link(other_span.context().span().span_context().clone());
    }

    /// Sets a span's parent to other_span
    pub fn set_span_parent_to(span: &Span, parent_span: &Span) {
        span.set_parent(parent_span.context());
    }
}

impl Drop for OtelTracer {
    fn drop(&mut self) {
        opentelemetry::global::shutdown_tracer_provider()
    }
}


struct HeaderInjector<'a>(pub &'a mut OwnedHeaders);

impl<'a> Injector for HeaderInjector<'a> {
    fn set(&mut self, key: &str, value: String) {
        let mut new = OwnedHeaders::new().insert(rdkafka::message::Header {
            key,
            value: Some(&value),
        });

        for header in self.0.iter() {
            let s = String::from_utf8(header.value.unwrap().to_vec()).unwrap();
            new = new.insert(rdkafka::message::Header {
                key: header.key,
                value: Some(&s),
            });
        }

        self.0.clone_from(&new);
    }
}

struct HeaderExtractor<'a>(pub &'a BorrowedHeaders);

impl<'a> Extractor for HeaderExtractor<'a> {
    fn get(&self, key: &str) -> Option<&str> {
        for i in 0..self.0.count() {
            if let Ok(val) = self.0.get_as::<str>(i) {
                if val.key == key {
                    return val.value;
                }
            }
        }
        None
    }

    fn keys(&self) -> Vec<&str> {
        self.0.iter().map(|kv| kv.key).collect::<Vec<_>>()
    }
}




/// This is a wrapper for a type which can be bundled with a span.
/// Given type Foo, define trait FooLike in the following fashion:
/// ```rust
/// trait FooLike : Debug + AsRef<Foo> + AsMut<Foo> {
///     fn new(/* ... */) -> Self where Self: Sized;
/// }
/// ```
/// and implement for both Foo and Spanned<Foo>, that is:
/// /// ```rust
/// impl FooLike for Foo {
///     fn new(/* ... */) -> Foo {
///         /* ... */
///     }
/// }
/// ```
/// and
/// ```rust
/// impl FooLike for Spanned<Foo> {
///     fn new(/* ... */) -> Spanned<Foo> {
///         /* ... */
///     }
/// }
/// ```
/// Now any function or struct that uses Foo, can use a generic that implements FooType instead.
/// For instance
/// ```rust
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
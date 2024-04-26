use opentelemetry::{
    propagation::{Extractor, Injector},
    trace::TraceError,
};
use opentelemetry_otlp::WithExportConfig;
use rdkafka::{
    message::{BorrowedHeaders, Headers, OwnedHeaders},
    producer::FutureRecord,
};
use std::fmt::Debug;
use tracing::{debug, level_filters::LevelFilter, Span};
use tracing_opentelemetry::OpenTelemetrySpanExt;
use tracing_subscriber::{filter, layer::SubscriberExt, Layer};

/// Should be called at the start of each component
/// The `conditional_` prefix used in the methods of FutureRecordTracerExt and OptionalHeaderTracerExt
/// indicate the method's first parameter is a bool, however here the first parameter is an Option<&str>
/// with the URL of the OpenTelemetry collector to be used, or None, if OpenTelemetry is not used.
#[macro_export]
macro_rules! conditional_init_tracer {
    ($otel_endpoint:expr) => {
        init_tracer!($otel_endpoint, LevelFilter::TRACE)
    };
    ($otel_endpoint:expr, $level: expr) => {
        $otel_endpoint
            .map(|otel_endpoint| {
                OtelTracer::new(
                    otel_endpoint,
                    env!("CARGO_BIN_NAME"),
                    Some((module_path!(), $level)),
                )
                .expect("Open Telemetry Tracer is created")
            })
            .or_else(|| {
                tracing_subscriber::fmt::init();
                None
            })
    };
}

/// May be used when the component produces messages.
/// The `conditional_` prefix indicates a bool should be passed,
/// indicating whether OpenTelemetry is used.
/// If this is false, the methods usually do nothing.
pub trait FutureRecordTracerExt {
    fn optional_headers(self, headers: Option<OwnedHeaders>) -> Self;
    fn conditional_inject_current_span_into_headers(self, use_otel: bool) -> Self;
    fn conditional_inject_span_into_headers(self, use_otel: bool, span: &Span) -> Self;
}

impl FutureRecordTracerExt for FutureRecord<'_, str, [u8]> {
    fn optional_headers(self, headers: Option<OwnedHeaders>) -> Self {
        if let Some(headers) = headers {
            self.headers(headers)
        } else {
            self
        }
    }

    fn conditional_inject_current_span_into_headers(self, use_otel: bool) -> Self {
        self.conditional_inject_span_into_headers(use_otel, &tracing::Span::current())
    }

    fn conditional_inject_span_into_headers(self, use_otel: bool, span: &Span) -> Self {
        if use_otel {
            let mut headers = self.headers.clone().unwrap_or_default();
            opentelemetry::global::get_text_map_propagator(|propagator| {
                propagator.inject_context(&span.context(), &mut HeaderInjector(&mut headers))
            });
            self.headers(headers)
        } else {
            self
        }
    }
}

/// May be used when the component consumne messages.
/// The `conditional_` prefix indicates a bool should be passed,
/// indicating whether OpenTelemetry is used.
/// If this is false, the methods usually do nothing.
pub trait OptionalHeaderTracerExt {
    fn conditional_extract_to_current_span(self, use_otel: bool);
    fn conditional_extract_to_span(self, use_otel: bool, span: &Span);
}

impl OptionalHeaderTracerExt for Option<&BorrowedHeaders> {
    fn conditional_extract_to_current_span(self, use_otel: bool) {
        self.conditional_extract_to_span(use_otel, &tracing::Span::current())
    }

    fn conditional_extract_to_span(self, use_otel: bool, span: &Span) {
        if let Some(headers) = self {
            if use_otel {
                debug!("Kafka Header Found");
                span.set_parent(opentelemetry::global::get_text_map_propagator(
                    |propagator| propagator.extract(&HeaderExtractor(headers)),
                ));
            }
        }
    }
}

/// Create this object to initialise the Open Telemetry Tracer
pub struct OtelTracer;

impl OtelTracer {
    /// Initialises an OpenTelemetry service for the crate
    /// #Arguments
    /// * `endpoint` - The URI where the traces are sent
    /// * `service_name` - The name of the OpenTelemetry service to assign to the crate.
    /// * `target` - An optional pair, the first element is the name of the crate/module, the second is the level above which spans and events with the target are filtered.
    /// Note that is target is set, then all traces with different targets are filtered out (such as traces sent from dependencies).
    /// If target is None then no filtering is done.
    pub fn new(
        endpoint: &str,
        service_name: &str,
        target: Option<(&str, LevelFilter)>,
    ) -> Result<Self, TraceError> {
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

        if let Some((target, tracing_level)) = target {
            let filter = filter::Targets::new()
                .with_default(LevelFilter::OFF)
                .with_target(target, tracing_level);

            let telemetry = tracing_opentelemetry::layer()
                .with_tracer(tracer)
                .with_filter(filter);

            let subscriber = tracing_subscriber::Registry::default().with(telemetry);
            tracing::subscriber::set_global_default(subscriber).unwrap();
        } else {
            let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);
            let subscriber = tracing_subscriber::Registry::default().with(telemetry);
            tracing::subscriber::set_global_default(subscriber).unwrap();
        };
        Ok(Self)
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

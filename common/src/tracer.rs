use opentelemetry::{
    global::Error, propagation::{Extractor, Injector}, trace::TraceError
};
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::trace::Tracer;
use rdkafka::{
    message::{BorrowedHeaders, Headers, OwnedHeaders},
    producer::FutureRecord,
};
use tracing::{debug, level_filters::LevelFilter, Span};
use tracing_opentelemetry::{OpenTelemetryLayer, OpenTelemetrySpanExt};
use tracing_subscriber::{
    filter::{self, Filtered, Targets}, layer::SubscriberExt, registry::LookupSpan, EnvFilter, Layer
};

pub struct OtelOptions<'a> {
    pub endpoint: &'a str,
    pub level_filter: LevelFilter,
}

pub struct TracerOptions<'a> {
    pub otel_options: Option<OtelOptions<'a>>,
}

/// Should be called at the start of each component
/// The `conditional_` prefix used in the methods of FutureRecordTracerExt and OptionalHeaderTracerExt
/// indicate the method's first parameter is a bool, however here the first parameter is an Option<&str>
/// with the URL of the OpenTelemetry collector to be used, or None, if OpenTelemetry is not used.
#[macro_export]
macro_rules! conditional_init_tracer {
    ($options:expr) => {
        {
            let tracer = TracerEngine::new($options, env!("CARGO_BIN_NAME"), module_path!());
            if tracer.is_some() {
                if let Err(e) = tracer.set_otel_error_handler(|e| warn!("{e}")) {
                    warn!("{e}");
                }
            }
            tracer
        }

    };
}

/// Create this object to initialise the Open Telemetry Tracer
/// as well as the stdout tracer and otel_status tracer (exclusively
/// for OpenTelemetry errors)
pub struct OtelTracer<S> {
    layer: Filtered<OpenTelemetryLayer<S, Tracer>, Targets, S>,
}

impl<S> OtelTracer<S>
where
    S: tracing::Subscriber,
    for<'span> S: LookupSpan<'span>,
{
    pub fn new(
        options: OtelOptions,
        service_name: &str,
        module_name: &str,
    ) -> Result<Self, TraceError> {
        let otlp_exporter = opentelemetry_otlp::new_exporter()
            .tonic()
            .with_endpoint(options.endpoint);

        let otlp_resource = opentelemetry_sdk::Resource::new(vec![opentelemetry::KeyValue::new(
            "service.name",
            service_name.to_owned(),
        )]);
        let otlp_config = opentelemetry_sdk::trace::Config::default().with_resource(otlp_resource);

        opentelemetry::global::set_text_map_propagator(
            opentelemetry_sdk::propagation::TraceContextPropagator::new(),
        );

        let tracer = opentelemetry_otlp::new_pipeline()
            .tracing()
            .with_trace_config(otlp_config)
            .with_exporter(otlp_exporter)
            .install_batch(opentelemetry_sdk::runtime::Tokio)?;

        let filter = filter::Targets::new()
            .with_default(LevelFilter::OFF)
            .with_target(module_name, options.level_filter)
            .with_target("otel", options.level_filter);

        let layer = tracing_opentelemetry::layer()
            .with_tracer(tracer)
            .with_filter(filter);

        Ok(Self { layer })
    }
}

pub struct TracerEngine {
    use_otel: bool,
}

impl TracerEngine {
    pub fn is_some(&self) -> bool {
        self.use_otel
    }
    pub fn set_otel_error_handler<F>(&self, f : F) -> Result<(),Error> where F : Fn(Error) + Send + Sync + 'static {
        opentelemetry::global::set_error_handler(f)
    }
}

impl TracerEngine {
    /// Initialises an OpenTelemetry service for the crate
    /// #Arguments
    /// * `endpoint` - The URI where the traces are sent
    /// * `service_name` - The name of the OpenTelemetry service to assign to the crate.
    /// * `target` - An optional pair, the first element is the name of the crate/module, the second is the level above which spans and events with the target are filtered.
    /// Note that is target is set, then all traces with different targets are filtered out (such as traces sent from dependencies).
    /// If target is None then no filtering is done.
    pub fn new(options: TracerOptions, service_name: &str, module_name: &str) -> Self {
        let use_otel = options.otel_options.is_some();

        let stdout_tracer = tracing_subscriber::fmt::layer().with_writer(std::io::stdout);

        let otel_tracer = options.otel_options.and_then(|otel_options| {
            OtelTracer::<_>::new(otel_options, service_name, module_name).ok()
        });

        let log_filter = EnvFilter::from_default_env();

        let subscriber = tracing_subscriber::Registry::default()
            .with(stdout_tracer.with_filter(log_filter))
            .with(otel_tracer.map(|otel_tracer| otel_tracer.layer));

        //  This is only called once, so will never panic
        tracing::subscriber::set_global_default(subscriber)
            .expect("tracing::subscriber::set_global_default should only be called once");

        Self { use_otel }
    }

    /// Sets a span's parent to other_span
    pub fn set_span_parent_to(span: &Span, parent_span: &Span) {
        span.set_parent(parent_span.context());
    }
}

impl Drop for TracerEngine {
    fn drop(&mut self) {
        opentelemetry::global::shutdown_tracer_provider()
    }
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

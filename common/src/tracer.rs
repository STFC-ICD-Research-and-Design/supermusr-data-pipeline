use opentelemetry::{
    propagation::{Extractor, Injector},
    trace::{TraceContextExt, TraceError, Tracer},
    Context,
};
use opentelemetry_otlp::WithExportConfig;
use rdkafka::message::{BorrowedHeaders, Headers, OwnedHeaders};
use tracing::Span;
use tracing_opentelemetry::{self, OpenTelemetrySpanExt};
use tracing_subscriber::layer::SubscriberExt;

const SERVICE_NAME: &str = "SuperMuSR";

pub fn init_tracer() -> Result<(), TraceError> {
    let endpoint = "http://localhost:4317/v1/traces";
    let otlp_exporter = opentelemetry_otlp::new_exporter()
        .tonic()
        .with_endpoint(endpoint);

    let otlp_resource = opentelemetry_sdk::Resource::new(vec![opentelemetry::KeyValue::new(
        "service.name",
        SERVICE_NAME.to_owned(),
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
    Ok(())
}

pub fn end_tracer() {
    opentelemetry::global::shutdown_tracer_provider()
}

pub fn create_new_span(span_name: &str, context: Option<Context>) -> Context {
    let tracer = opentelemetry::global::tracer(SERVICE_NAME.to_owned());
    let span = if let Some(context) = context {
        tracer.start_with_context(span_name.to_owned(), &context)
    } else {
        tracer.start(span_name.to_owned())
    };
    Context::current_with_span(span)
}

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

pub fn inject_context_from_span(span: &Span) -> OwnedHeaders {
    inject_context(&span.context())
}
pub fn extract_context_to_span(headers: &BorrowedHeaders, span: &Span) {
    span.set_parent(extract_context(headers));
}

pub fn link_span_to_context(span: &Span, context: &Context) {
    span.add_link(context.span().span_context().clone());
}

pub fn link_span_to_span(span: &Span, other_span: &Span) {
    link_span_to_context(span, &other_span.context());
}

pub fn inject_context(parent_context: &Context) -> OwnedHeaders {
    let mut headers = OwnedHeaders::new();
    opentelemetry::global::get_text_map_propagator(|propagator| {
        propagator.inject_context(parent_context, &mut HeaderInjector(&mut headers))
    });
    headers
}

pub fn extract_context(headers: &BorrowedHeaders) -> Context {
    opentelemetry::global::get_text_map_propagator(|propagator| {
        propagator.extract(&HeaderExtractor(headers))
    })
}



pub struct HeaderInjector<'a>(pub &'a mut OwnedHeaders);

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

pub struct HeaderExtractor<'a>(pub &'a BorrowedHeaders);

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
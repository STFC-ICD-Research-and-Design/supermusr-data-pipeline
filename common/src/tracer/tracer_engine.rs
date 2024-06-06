use opentelemetry::global::Error;
use tracing::Span;
use tracing_opentelemetry::OpenTelemetrySpanExt;
use tracing_subscriber::{layer::SubscriberExt, EnvFilter, Layer};

use super::otel_tracer::{OtelOptions, OtelTracer};

pub struct TracerOptions<'a> {
    pub otel_options: Option<OtelOptions<'a>>,
}

/// This object initialises all tracers, given a TracerOptions struct.
/// If TracerOptions contains a OtelOptions struct then it initialises the
/// OtelTracer object as well.
pub struct TracerEngine {
    use_otel: bool,
}

impl TracerEngine {
    pub fn is_some(&self) -> bool {
        self.use_otel
    }
    pub fn set_otel_error_handler<F>(&self, f: F) -> Result<(), Error>
    where
        F: Fn(Error) + Send + Sync + 'static,
    {
        opentelemetry::global::set_error_handler(f)
    }
}

impl TracerEngine {
    /// Initialises the stdout tracer, and (if required) the OpenTelemetry service for the crate
    /// #Arguments
    /// * `options` - The caller-specified instance of TracerOptions.
    /// * `service_name` - The name of the OpenTelemetry service to assign to the crate.
    /// * `module_name` - The name of the current module.
    /// #Returns
    /// An instance of TracerEngine
    pub fn new(options: TracerOptions, service_name: &str, module_name: &str) -> Self {
        let use_otel = options.otel_options.is_some();

        let stdout_tracer = tracing_subscriber::fmt::layer().with_writer(std::io::stdout);

        let otel_tracer = options.otel_options.and_then(|otel_options| {
            OtelTracer::<_>::new(otel_options, service_name, module_name).ok()
        });
        // If otel_tracer did not work, update the use_otel variable
        let use_otel = use_otel && otel_tracer.is_some();

        // This filter is applied to the stdout tracer
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

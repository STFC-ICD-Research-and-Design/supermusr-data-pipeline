use super::otel_tracer::{OtelOptions, OtelTracer};
use opentelemetry::{global::Error, trace::TraceError};
use tracing::{level_filters::LevelFilter, Span};
use tracing_opentelemetry::OpenTelemetrySpanExt;
use tracing_subscriber::{layer::SubscriberExt, EnvFilter, Layer};

pub struct TracerOptions<'a> {
    otel_options: Option<OtelOptions<'a>>,
}

impl<'a> TracerOptions<'a> {
    pub fn new(endpoint: Option<&'a str>, level_filter: LevelFilter) -> Self {
        Self {
            otel_options: endpoint.map(|endpoint| OtelOptions {
                endpoint,
                level_filter,
            }),
        }
    }
}

/// This object initialises all tracers, given a TracerOptions struct.
/// If TracerOptions contains a OtelOptions struct then it initialises the
/// OtelTracer object as well.
pub struct TracerEngine {
    use_otel: bool,
    otel_setup_error: Option<TraceError>,
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

        let (otel_tracer, otel_setup_error) = options.otel_options.map(|otel_options| {
            match OtelTracer::<_>::new(otel_options, service_name, module_name) {
                Ok(otel_tracer) => (Some(otel_tracer), None),
                Err(e) => (None, Some(e))
            }
        }).unwrap_or((None, None));
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

        Self { use_otel, otel_setup_error }
    }

    /// Sets a span's parent to other_span
    pub fn set_span_parent_to(span: &Span, parent_span: &Span) {
        span.set_parent(parent_span.context());
    }

    pub fn use_otel(&self) -> bool {
        self.use_otel
    }

    /// This sets a custom error handler for open-telemetry.
    /// This is public so it can be used in the init_tracer macro,
    /// but should not be called anywhere else.
    pub fn set_otel_error_handler<F>(&self, f: F) -> Result<(), Error>
    where
        F: Fn(Error) + Send + Sync + 'static,
    {
        opentelemetry::global::set_error_handler(f)
    }

    pub fn get_otel_setup_error(&self) -> Option<&TraceError> {
        self.otel_setup_error.as_ref()
    }
}

impl Drop for TracerEngine {
    fn drop(&mut self) {
        opentelemetry::global::shutdown_tracer_provider()
    }
}

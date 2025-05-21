use opentelemetry::trace::TraceError;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::trace::Tracer;
use tracing::{level_filters::LevelFilter, warn};
use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::{EnvFilter, Layer, filter::Filtered, registry::LookupSpan};

pub(super) struct OtelOptions<'a> {
    pub(super) endpoint: &'a str,
    pub(super) namespace: String,
}

/// Create this object to initialise the Open Telemetry Tracer
pub struct OtelTracer<S> {
    pub(super) layer: Filtered<OpenTelemetryLayer<S, Tracer>, EnvFilter, S>,
}

impl<S> OtelTracer<S>
where
    S: tracing::Subscriber,
    for<'span> S: LookupSpan<'span>,
{
    /// Initialises an OpenTelemetry service for the crate
    ///
    /// ## Arguments
    /// * `options` - The caller-specified options for the service
    /// * `service_name` - The name of the OpenTelemetry service to assign to the crate.
    /// * `module_name` - The name of the current module.
    ///
    /// ## Returns
    /// If the tracer is set up correctly, an instance of OtelTracer containing the
    /// `tracing_opentelemetry` layer which can be added to the subscriber.
    /// If the operation fails, a TracerError is returned.
    pub(super) fn new(options: OtelOptions, service_name: &str) -> Result<Self, TraceError> {
        let otlp_exporter = opentelemetry_otlp::new_exporter()
            .tonic()
            .with_endpoint(options.endpoint);

        let service_name = opentelemetry::KeyValue::new("service.name", service_name.to_owned());
        let service_namespace =
            opentelemetry::KeyValue::new("service.namespace", options.namespace);

        let otlp_config = opentelemetry_sdk::trace::Config::default().with_resource(
            opentelemetry_sdk::Resource::new(vec![service_name, service_namespace]),
        );

        opentelemetry::global::set_text_map_propagator(
            opentelemetry_sdk::propagation::TraceContextPropagator::new(),
        );

        let tracer = opentelemetry_otlp::new_pipeline()
            .tracing()
            .with_trace_config(otlp_config)
            .with_exporter(otlp_exporter)
            .install_batch(opentelemetry_sdk::runtime::Tokio)?;

        let filter = match EnvFilter::builder()
            .with_default_directive(LevelFilter::OFF.into())
            .with_env_var("OTEL_LEVEL")
            .from_env()
        {
            Ok(filter) => filter,
            Err(e) => {
                warn!("Invalid directive(s) in OTEL_LEVEL: {e}");
                EnvFilter::default()
            }
        };
        let layer = tracing_opentelemetry::layer()
            .with_tracer(tracer)
            .with_filter(filter);

        Ok(Self { layer })
    }
}

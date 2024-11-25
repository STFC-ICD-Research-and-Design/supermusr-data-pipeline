use opentelemetry::trace::TraceError;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::trace::Tracer;
use tracing::level_filters::LevelFilter;
use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::{
    filter::{self, Filtered, Targets},
    registry::LookupSpan,
    Layer,
};

pub(super) struct OtelOptions<'a> {
    pub(super) endpoint: &'a str,
    pub(super) level_filter: LevelFilter,
    pub(super) pipeline_tag: Option<String>,
}

/// Create this object to initialise the Open Telemetry Tracer
pub struct OtelTracer<S> {
    pub(super) layer: Filtered<OpenTelemetryLayer<S, Tracer>, Targets, S>,
}

impl<S> OtelTracer<S>
where
    S: tracing::Subscriber,
    for<'span> S: LookupSpan<'span>,
{
    /// Initialises an OpenTelemetry service for the crate
    /// #Arguments
    /// * `options` - The caller-specified options for the service
    /// * `service_name` - The name of the OpenTelemetry service to assign to the crate.
    /// * `module_name` - The name of the current module.
    /// #Returns
    /// If the tracer is set up correctly, an instance of OtelTracer containing the
    /// `tracing_opentelemetry` layer which can be added to the subscriber.
    /// If the operation fails, a TracerError is returned.
    pub(super) fn new(
        options: OtelOptions,
        service_name: &str,
        module_name: &str,
    ) -> Result<Self, TraceError> {
        let otlp_exporter = opentelemetry_otlp::new_exporter()
            .tonic()
            .with_endpoint(options.endpoint);

        let otlp_config = opentelemetry_sdk::trace::Config::default().with_resource(
            opentelemetry_sdk::Resource::new(vec![opentelemetry::KeyValue::new(
                "service.name",
                format!(
                    "{}{}",
                    options.pipeline_tag.unwrap_or_default(),
                    service_name
                ),
            )]),
        );

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

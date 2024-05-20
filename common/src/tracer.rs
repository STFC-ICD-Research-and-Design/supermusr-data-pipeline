use opentelemetry::{
    global::Error,
    propagation::{Extractor, Injector},
    trace::TraceError,
};
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::trace::Tracer;
use rdkafka::{
    message::{BorrowedHeaders, Headers, OwnedHeaders},
    producer::FutureRecord,
};
use tracing::{debug, info, level_filters::LevelFilter, warn, Span};
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
                let tracer = OtelTracer::new(
                    otel_endpoint,
                    env!("CARGO_BIN_NAME"),
                    Some((module_path!(), $level)),
                );
                tracer.get_fail_status().map(|fail_status|
                    warn!(target: "otel-status", "{fail_status}")
                );
                tracer
            })
            .or_else(|| {
                tracing_subscriber::fmt::init();
                None
            });
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

fn error_handler(error_service_name: &str, e: Error) {
    /*let str = match e {
        opentelemetry::global::Error::Trace(e) => match e {
            TraceError::ExportFailed(e) => {
                format!("exporter error: {0}", e.exporter_name())
            }
            TraceError::ExportTimedOut(dur) => format!("exporter timeout: {dur:?}"),
            TraceError::Other(e) => format!("other trace error: {e}"),
            _ => format!("unknown trace error"),
        },
        opentelemetry::global::Error::Metric(e) => format!("metric error: {e}"),
        opentelemetry::global::Error::Other(e) => format!("other error: {e}"),
        _ => format!("unknown error"),
    };*/
    warn!(target: "otel-status", "{error_service_name}: {e}");
    //tracing::subscriber::with_default(tracing_subscriber::fmt::init(),||
    //);
}

/// Create this object to initialise the Open Telemetry Tracer
pub struct OtelTracer {
    fail_status: Option<String>
}

impl OtelTracer {
    fn create_otel_tracer(endpoint: &str, service_name: &str) -> Result<Tracer, TraceError> {
        let otlp_exporter = opentelemetry_otlp::new_exporter()
            .tonic()
            .with_endpoint(endpoint);

        let otlp_resource =
            opentelemetry_sdk::Resource::new(vec![opentelemetry::KeyValue::new(
                "service.name",
                service_name.to_owned(),
            )]);
        let otlp_config =
            opentelemetry_sdk::trace::Config::default().with_resource(otlp_resource);
            
        let error_service_name = service_name.to_owned();
        opentelemetry::global::set_error_handler(move |e| error_handler(&error_service_name, e))
            .unwrap();

        opentelemetry::global::set_text_map_propagator(
            opentelemetry_sdk::propagation::TraceContextPropagator::new(),
        );

        opentelemetry_otlp::new_pipeline()
            .tracing()
            .with_trace_config(otlp_config)
            .with_exporter(otlp_exporter)
            .install_batch(opentelemetry_sdk::runtime::Tokio)
    }

    /// Initialises an OpenTelemetry service for the crate
    /// #Arguments
    /// * `endpoint` - The URI where the traces are sent
    /// * `service_name` - The name of the OpenTelemetry service to assign to the crate.
    /// * `target` - An optional pair, the first element is the name of the crate/module, the second is the level above which spans and events with the target are filtered.
    /// Note that is target is set, then all traces with different targets are filtered out (such as traces sent from dependencies).
    /// If target is None then no filtering is done.
    #[tracing::instrument]
    pub fn new(
        endpoint: &str,
        service_name: &str,
        target: Option<(&str, LevelFilter)>,
    ) -> Self {
        let stdout_tracer = tracing_subscriber::fmt::layer().with_writer(std::io::stdout).pretty();

        let (otel_tracer,fail_status) = match Self::create_otel_tracer(endpoint, service_name) {
            Ok(tracer) => {
                //info_span!("OpenTelemetry").in_scope(||info!("Tracer Created"));
                (Some(tracer), None)
            },
            Err(e) => {
                //warn_span!("OpenTelemetry Error").in_scope(||warn!("{e}"));
                (None, Some(format!("{e}")))
            },
        };
        let otel_status_tracer = tracing_subscriber::fmt::layer().with_writer(std::io::stdout)
            .pretty()
            .with_filter(filter::Targets::new()
                .with_target("otel-status", LevelFilter::TRACE)
            );


        let telemetry = otel_tracer.map(|tracer| tracing_opentelemetry::layer().with_tracer(tracer));

        if let Some((target, tracing_level)) = target {
            let filter = filter::Targets::new()
                .with_default(LevelFilter::OFF)
                .with_target(target, tracing_level);

            let subscriber = tracing_subscriber::Registry::default().with(
                stdout_tracer
                    .with_filter(filter.clone())
                    .and_then(telemetry.map(|telemetry| telemetry.with_filter(filter)))
                    .and_then(otel_status_tracer)
            );

            //  This is only called once, so will never panic
            tracing::subscriber::set_global_default(subscriber)
                .expect("tracing::subscriber::set_global_default should only be called once");
        
        } else {
            let subscriber =
                tracing_subscriber::Registry::default().with(stdout_tracer.and_then(telemetry));

            //  This is only called once, so will never panic
            tracing::subscriber::set_global_default(subscriber)
                .expect("tracing::subscriber::set_global_default should only be called once");
        };
        info!("Tracing Test");
        Self { fail_status }
    }

    /// Sets a span's parent to other_span
    pub fn set_span_parent_to(span: &Span, parent_span: &Span) {
        span.set_parent(parent_span.context());
    }

    pub fn get_fail_status(&self) -> Option<&str> {
        self.fail_status.as_deref()
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

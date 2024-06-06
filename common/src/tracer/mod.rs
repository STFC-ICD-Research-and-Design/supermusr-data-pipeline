mod otel_tracer;
mod propagator;
mod tracer_engine;

pub use otel_tracer::{OtelOptions, OtelTracer};
pub use propagator::{FutureRecordTracerExt, OptionalHeaderTracerExt};
pub use tracer_engine::{TracerEngine, TracerOptions};

/// Should be called at the start of each component
/// The `conditional_` prefix used in the methods of FutureRecordTracerExt and OptionalHeaderTracerExt
/// indicate the method's first parameter is a bool, however here the first parameter is an Option<&str>
/// with the URL of the OpenTelemetry collector to be used, or None, if OpenTelemetry is not used.
#[macro_export]
macro_rules! init_tracer {
    ($options:expr) => {{
        let tracer = TracerEngine::new($options, env!("CARGO_BIN_NAME"), module_path!());
        // This is called here (in the macro) rather than as part of `TracerEngine::new`
        // to ensure the warning is emitted in the correct module.
        if tracer.is_some() {
            if let Err(e) = tracer.set_otel_error_handler(|e| warn!("{e}")) {
                warn!("{e}");
            }
        }
        tracer
    }};
}

//! # OpenTelemetry Tracing Integration
//!
//! This crate provides a simplified initialization function for setting up OpenTelemetry
//! tracing, metrics, and logging with sensible defaults for Rust applications.
//!
//! ## Features
//!
//! - **Distributed Tracing**: Automatically exports spans to OTLP-compatible collectors
//! - **Metrics Collection**: Periodic metric export with configurable intervals
//! - **Structured Logging**: Log export with automatic filtering of telemetry noise
//! - **Terminal Output**: Human-readable logs respecting `RUST_LOG` environment variable
//! - **Trace Context Propagation**: Automatic propagation of trace context across service boundaries
//!
//! ## Configuration
//!
//! The telemetry data is sent to `http://localhost:4317` by default. This can be overridden
//! by setting the `OTEL_EXPORTER_OTLP_ENDPOINT` environment variable.
//!
//! Terminal logging respects the `RUST_LOG` environment variable for filtering, defaulting
//! to `info` level if not set.
//!
//! ## Filtering
//!
//! To prevent telemetry loops and reduce noise, logs from the following components
//! are automatically filtered out:
//! - `hyper` - HTTP client/server library
//! - `tonic` - gRPC library
//! - `h2` - HTTP/2 implementation
//! - `reqwest` - HTTP client library
//! - `opentelemetry` - OpenTelemetry SDK itself

use std::time::Duration;

use opentelemetry::trace::TracerProvider;
use tracing::Subscriber;

use tracing_subscriber::{EnvFilter, Layer, layer::SubscriberExt};

/// Initializes OpenTelemetry tracing, metrics, and logging with sensible defaults.
///
/// This function sets up a complete observability stack including:
/// - Distributed tracing with OTLP export
/// - Metrics collection with periodic export (60-second intervals)
/// - Structured logging with automatic noise filtering
/// - Terminal logging that respects the `RUST_LOG` environment variable
///
/// ## Parameters
///
/// * `pkg_name` - The name of your service/application, used in telemetry metadata
/// * `pkg_version` - The version of your service/application, used in telemetry metadata
///
/// ## Returns
///
/// A tuple containing:
/// * `SdkMeterProvider` - The metrics provider for creating custom meters and instruments
/// * `Subscriber` - The configured tracing subscriber that should be initialized with `.init()`
///
/// ## Environment Variables
///
/// * `OTEL_EXPORTER_OTLP_ENDPOINT` - Override the default OTLP endpoint (default: `http://localhost:4317`)
/// * `RUST_LOG` - Control terminal logging levels (default: `info`)
///
/// ## Example
///
/// ```rust
/// use open_telemetry_tracing::init;
/// use tracing_subscriber::util::SubscriberInitExt;
///
/// const PKG_NAME: &str = env!("CARGO_PKG_NAME");
/// const PKG_VERSION: &str = env!("CARGO_PKG_VERSION");
/// let (meter_provider, subscriber) = open_telemetry_tracing::init(PKG_NAME, PKG_VERSION);
/// tracing::subscriber::set_global_default(subscriber).unwrap();
/// opentelemetry::global::set_meter_provider(meter_provider);
///
/// // Use the meter provider for custom metrics
///
/// let meter = opentelemetry::global::meter("payment-service");
/// let counter = meter.u64_counter("payments_processed").init();
/// counter.add(1, &[]);
/// ```
pub fn init(
    pkg_name: &'static str,
    pkg_version: &'static str,
) -> (
    opentelemetry_sdk::metrics::SdkMeterProvider,
    impl Subscriber + Send + Sync + 'static,
) {
    // Configure trace context propagation for distributed tracing
    // This ensures trace context is properly propagated across service boundaries
    opentelemetry::global::set_text_map_propagator(
        opentelemetry_sdk::propagation::TraceContextPropagator::new(),
    );

    // Create a shared resource definition that identifies this service
    // This metadata appears in all telemetry data (traces, metrics, logs)
    let resource = opentelemetry_sdk::Resource::builder()
        .with_service_name(pkg_name)
        .with_attribute(opentelemetry::KeyValue::new("service.version", pkg_version))
        .build();

    // === DISTRIBUTED TRACING SETUP ===
    // Configure the OTLP span exporter to send trace data to the collector
    let span_exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_tonic()
        .build()
        .unwrap();

    // Create the tracer provider with always-on sampling
    // In production, you might want to use probabilistic sampling for high-volume services
    let tracer_provider = opentelemetry_sdk::trace::SdkTracerProvider::builder()
        .with_sampler(opentelemetry_sdk::trace::Sampler::AlwaysOn)
        .with_resource(resource.clone())
        .with_batch_exporter(span_exporter)
        .build();

    // Create the tracing layer that bridges tracing spans to OpenTelemetry
    // Only INFO level and above spans are exported to reduce noise
    let trace_layer = tracing_opentelemetry::layer()
        .with_tracer(tracer_provider.tracer("default_tracer"))
        .with_tracked_inactivity(true)
        .with_filter(tracing::level_filters::LevelFilter::INFO);

    // === METRICS COLLECTION SETUP ===
    // Configure the OTLP metrics exporter with delta temporality
    // Delta temporality means only changes since the last export are sent
    let metrics_exporter = opentelemetry_otlp::MetricExporter::builder()
        .with_tonic()
        .with_temporality(opentelemetry_sdk::metrics::Temporality::Delta)
        .build()
        .unwrap();

    // Create a periodic reader that exports metrics every 60 seconds
    let metrics_reader = opentelemetry_sdk::metrics::PeriodicReader::builder(metrics_exporter)
        .with_interval(Duration::from_secs(60))
        .build();

    // Build the meter provider that applications use to create custom metrics
    let meter_provider = opentelemetry_sdk::metrics::SdkMeterProvider::builder()
        .with_resource(resource.clone())
        .with_reader(metrics_reader)
        .build();

    // Create the metrics layer that automatically exports tracing-derived metrics
    let metrics_layer = tracing_opentelemetry::MetricsLayer::new(meter_provider.clone());

    // === STRUCTURED LOGGING SETUP ===
    // Configure the OTLP log exporter for structured log export
    let log_exporter = opentelemetry_otlp::LogExporter::builder()
        .with_tonic()
        .build()
        .unwrap();

    // Create the log provider for exporting structured logs
    let log_provider = opentelemetry_sdk::logs::SdkLoggerProvider::builder()
        .with_resource(resource)
        .with_batch_exporter(log_exporter)
        .build();

    // Create a filter to prevent telemetry-induced-telemetry loops
    // This is necessary because HTTP libraries used by OTLP exporters generate their own logs
    // which would otherwise create an infinite loop of telemetry about telemetry.
    //
    // We suppress logs from:
    // - `hyper`: HTTP client/server (used by tonic)
    // - `tonic`: gRPC library (used by OTLP exporters)
    // - `h2`: HTTP/2 implementation (used by tonic)
    // - `reqwest`: HTTP client library (used by some exporters)
    // - `opentelemetry`: OpenTelemetry SDK internal logs
    //
    // Note: This filtering affects ALL logs from these components, not just OTLP-related ones.
    // This is a known limitation until proper context-aware filtering is implemented.
    // See: https://github.com/open-telemetry/opentelemetry-rust/issues/2877
    let filter_otel = tracing_subscriber::EnvFilter::new("info")
        .add_directive("hyper=off".parse().unwrap())
        .add_directive("tonic=off".parse().unwrap())
        .add_directive("h2=off".parse().unwrap())
        .add_directive("reqwest=off".parse().unwrap())
        .add_directive("opentelemetry=off".parse().unwrap());

    // Create the OpenTelemetry logging bridge with noise filtering
    let log_layer =
        opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge::new(&log_provider)
            .with_filter(filter_otel);

    // === TERMINAL LOGGING SETUP ===
    // Create an environment filter that respects the RUST_LOG environment variable
    // This allows users to control terminal log verbosity independently of telemetry export
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    // Create a human-readable formatter for terminal output
    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_level(true)
        .with_filter(env_filter);

    // === COMPOSE ALL LAYERS ===
    // Combine all the layers into a single subscriber
    // Order matters: later layers can see events from earlier layers
    let subsciber = tracing_subscriber::registry()
        .with(fmt_layer) // Terminal output (respects RUST_LOG)
        .with(trace_layer) // OpenTelemetry trace export
        .with(log_layer) // OpenTelemetry log export
        .with(metrics_layer); // OpenTelemetry metrics export

    (meter_provider, subsciber)
}

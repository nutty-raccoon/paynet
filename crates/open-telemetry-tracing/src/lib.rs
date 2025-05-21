use std::time::Duration;

use opentelemetry::trace::TracerProvider;
use tracing::Subscriber;

use tracing_subscriber::{EnvFilter, Layer, fmt::format::FmtSpan, layer::SubscriberExt};

/// By default, the data will be sent to `https://localhost:4317`,
/// to override this behavious set the `OTEL_EXPORTER_OTLP_ENDPOINT` env variable.
///
/// Terminal logs will respect the RUST_LOG environment variable.
/// OpenTelemetry tracing will use the provided level_filter.
///
/// Can panic.
pub fn init(
    pkg_name: &'static str,
    pkg_version: &'static str,
) -> (
    opentelemetry_sdk::metrics::SdkMeterProvider,
    impl Subscriber + Send + Sync + 'static,
) {
    opentelemetry::global::set_text_map_propagator(
        opentelemetry_sdk::propagation::TraceContextPropagator::new(),
    );

    // Resource definition
    let resource = opentelemetry_sdk::Resource::builder()
        .with_service_name(pkg_name)
        .with_attribute(opentelemetry::KeyValue::new("service.version", pkg_version))
        .build();

    // Traces
    let span_exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_tonic()
        .build()
        .unwrap();

    let tracer_provider = opentelemetry_sdk::trace::SdkTracerProvider::builder()
        .with_sampler(opentelemetry_sdk::trace::Sampler::AlwaysOn)
        .with_resource(resource.clone())
        .with_batch_exporter(span_exporter)
        .build();

    let trace_layer = tracing_opentelemetry::layer()
        .with_tracer(tracer_provider.tracer("default_tracer"))
        .with_tracked_inactivity(true)
        .with_filter(tracing::level_filters::LevelFilter::INFO);

    // Metrics
    let metrics_exporter = opentelemetry_otlp::MetricExporter::builder()
        .with_tonic()
        .with_temporality(opentelemetry_sdk::metrics::Temporality::Delta)
        .build()
        .unwrap();

    let metrics_reader = opentelemetry_sdk::metrics::PeriodicReader::builder(metrics_exporter)
        .with_interval(Duration::from_secs(60))
        .build();

    let meter_provider = opentelemetry_sdk::metrics::SdkMeterProvider::builder()
        .with_resource(resource.clone())
        .with_reader(metrics_reader)
        .build();

    let metrics_layer = tracing_opentelemetry::MetricsLayer::new(meter_provider.clone());

    // Logs
    let log_exporter = opentelemetry_otlp::LogExporter::builder()
        .with_tonic()
        .build()
        .unwrap();

    let log_provider = opentelemetry_sdk::logs::SdkLoggerProvider::builder()
        .with_resource(resource)
        .with_batch_exporter(log_exporter)
        .build();

    // To prevent a telemetry-induced-telemetry loop, OpenTelemetry's own internal
    // logging is properly suppressed. However, logs emitted by external components
    // (such as reqwest, tonic, etc.) are not suppressed as they do not propagate
    // OpenTelemetry context. Until this issue is addressed
    // (https://github.com/open-telemetry/opentelemetry-rust/issues/2877),
    // filtering like this is the best way to suppress such logs.
    //
    // The filter levels are set as follows:
    // - Allow `info` level and above by default.
    // - Completely restrict logs from `hyper`, `tonic`, `h2`, and `reqwest`.
    //
    // Note: This filtering will also drop logs from these components even when
    // they are used outside of the OTLP Exporter.
    let filter_otel = tracing_subscriber::EnvFilter::new("info")
        .add_directive("hyper=off".parse().unwrap())
        .add_directive("opentelemetry=off".parse().unwrap())
        .add_directive("tonic=off".parse().unwrap())
        .add_directive("h2=off".parse().unwrap())
        .add_directive("reqwest=off".parse().unwrap());
    let log_layer =
        opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge::new(&log_provider)
            .with_filter(filter_otel);

    // Create a new EnvFilter that respects the RUST_LOG environment variable
    // Default to "info" if RUST_LOG is not set
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_level(true)
        .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
        .with_filter(env_filter);

    let subsciber = tracing_subscriber::registry()
        .with(fmt_layer)
        .with(trace_layer)
        .with(log_layer)
        .with(metrics_layer);

    (meter_provider, subsciber)
}

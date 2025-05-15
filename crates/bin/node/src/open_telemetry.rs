use opentelemetry::trace::TracerProvider;
use tracing::{Level, level_filters::LevelFilter};
use tracing_subscriber::{
    Layer, fmt::format::FmtSpan, layer::SubscriberExt, util::SubscriberInitExt,
};

/// By default, the data will be sent to `https://localhost:4317`,
/// to override this behavious set the `OTEL_EXPORTER_OTLP_ENDPOINT` env variable
pub async fn init_tracing() {
    opentelemetry::global::set_text_map_propagator(
        opentelemetry_sdk::propagation::TraceContextPropagator::new(),
    );

    const PKG_NAME: &str = env!("CARGO_PKG_NAME");
    const PKG_VERSION: &str = env!("CARGO_PKG_VERSION");

    let resource = opentelemetry_sdk::Resource::builder()
        .with_service_name(PKG_NAME)
        .with_attribute(opentelemetry::KeyValue::new("service.version", PKG_VERSION))
        .build();

    let exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_tonic()
        .build()
        .unwrap();

    let tracer_provider = opentelemetry_sdk::trace::SdkTracerProvider::builder()
        .with_sampler(opentelemetry_sdk::trace::Sampler::AlwaysOn)
        .with_resource(resource)
        .with_batch_exporter(exporter)
        .build();

    let telemetry = tracing_opentelemetry::layer()
        .with_tracer(tracer_provider.tracer("default_tracer"))
        .with_tracked_inactivity(true)
        .with_filter(tracing::level_filters::LevelFilter::INFO);

    let fmt = tracing_subscriber::fmt::layer()
        .with_level(true)
        .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE);

    tracing_subscriber::registry()
        .with(LevelFilter::from_level(Level::INFO))
        .with(telemetry)
        .with(fmt)
        .init();
}

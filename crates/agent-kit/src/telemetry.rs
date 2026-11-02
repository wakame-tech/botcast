//! tracing と OpenTelemetry（OTLP の gRPC で collector に送る）の初期化

use opentelemetry::trace::TracerProvider as _;
use opentelemetry::KeyValue;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::runtime::Tokio;
use opentelemetry_sdk::trace::TracerProvider;
use opentelemetry_sdk::Resource;
use std::str::FromStr;
use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::EnvFilter;

/// グローバルの subscriber を設定する。
/// `service_name` は span の `service.name` と、ログを `trace` まで出す crate の名前（`env!("CARGO_CRATE_NAME")`）
pub fn init_tracing(service_name: &str, otlp_collector_endpoint: String) -> anyhow::Result<()> {
    let subscriber = tracing_subscriber::registry();
    let exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_tonic()
        .with_endpoint(otlp_collector_endpoint)
        .build()?;

    let tracer_provider = TracerProvider::builder()
        .with_batch_exporter(exporter, Tokio)
        .with_resource(Resource::new(vec![KeyValue::new(
            "service.name",
            service_name.to_string(),
        )]))
        .build();
    let otel_layer = OpenTelemetryLayer::new(tracer_provider.tracer(service_name.to_string()));
    let subscriber = subscriber
        .with(otel_layer)
        .with(EnvFilter::from_str(&format!("info,{}=trace", service_name))?);

    let fmt_layer = tracing_subscriber::fmt::layer().pretty();
    let subscriber = subscriber.with(fmt_layer);
    tracing::subscriber::set_global_default(subscriber)?;
    Ok(())
}

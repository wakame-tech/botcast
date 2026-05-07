use opentelemetry::trace::TracerProvider as _;
use opentelemetry::KeyValue;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::runtime::Tokio;
use opentelemetry_sdk::trace::TracerProvider;
use opentelemetry_sdk::Resource;
use std::{str::FromStr, sync::Arc};
use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::EnvFilter;
use worker::{api::start_api, usecase::Provider, worker::start_worker};
use worker::usecase::mcp_client::McpClient;

async fn seed_cms_collections() -> anyhow::Result<()> {
    let mcp_cmd = std::env::var("MCP_SERVER_CMD").unwrap_or_else(|_| "node".to_string());
    let mcp_args_str = match std::env::var("MCP_SERVER_ARGS") {
        Ok(v) => v,
        Err(_) => {
            tracing::warn!("MCP_SERVER_ARGS not set, skipping CMS collection seeding");
            return Ok(());
        }
    };
    let mcp_args: Vec<&str> = mcp_args_str.split_whitespace().collect();
    let client = McpClient::new(&mcp_cmd, &mcp_args).await?;

    let podcasts_schema = serde_json::json!({
        "type": "object",
        "properties": {
            "title": { "type": "string" },
            "icon": { "type": "string" },
            "description": { "type": "string" },
            "user_id": { "type": "string" }
        },
        "required": ["title", "icon", "user_id"]
    });
    let episodes_schema = serde_json::json!({
        "type": "object",
        "properties": {
            "title": { "type": "string" },
            "podcast_id": { "type": "string" },
            "description": { "type": "string" },
            "audio_url": { "type": "string" },
            "srt_url": { "type": "string" },
            "sections": { "type": "array" },
            "duration_sec": { "type": "integer" },
            "user_id": { "type": "string" }
        },
        "required": ["title", "podcast_id", "user_id"]
    });

    client.ensure_collection("podcasts", podcasts_schema).await?;
    client.ensure_collection("episodes", episodes_schema).await?;
    client.close().await?;
    Ok(())
}

fn init_tracing(otlp_collector_endpoint: String) -> anyhow::Result<()> {
    let crate_name = env!("CARGO_CRATE_NAME");

    let subscriber = tracing_subscriber::registry();
    let exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_tonic()
        .with_endpoint(otlp_collector_endpoint)
        .build()?;
    // let exporter = opentelemetry_stdout::SpanExporter::default();

    let tracer_provider = TracerProvider::builder()
        .with_batch_exporter(exporter, Tokio)
        .with_resource(Resource::new(vec![KeyValue::new(
            "service.name",
            crate_name.to_string(),
        )]))
        .build();
    let otel_layer = OpenTelemetryLayer::new(tracer_provider.tracer("worker"));
    let subscriber = subscriber
        .with(otel_layer)
        .with(EnvFilter::from_str(&format!("info,{}=trace", crate_name,))?);

    let fmt_layer = tracing_subscriber::fmt::layer().pretty();
    let subscriber = subscriber.with(fmt_layer);
    tracing::subscriber::set_global_default(subscriber)?;
    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let otlp_collector_endpoint = std::env::var("OTLP_COLLECTOR_ENDPOINT")?;
    init_tracing(otlp_collector_endpoint)?;

    let kafru_db = Arc::new(
        kafru::database::Db::new(None)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to connect to kafru DB: {}", e))?,
    );
    let kafru_queue = Arc::new(kafru::queue::Queue::new(Some(kafru_db.clone())).await);

    seed_cms_collections().await?;

    let provider = Arc::new(Provider::new(kafru_queue));
    start_worker(provider.clone(), kafru_db);
    start_api(provider).await
}

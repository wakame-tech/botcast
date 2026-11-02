use std::sync::Arc;
use worker::{
    api::start_api,
    usecase::Provider,
    worker::{start_schedule_sync, start_worker},
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let otlp_collector_endpoint = std::env::var("OTLP_COLLECTOR_ENDPOINT")?;
    agent_kit::telemetry::init_tracing(env!("CARGO_CRATE_NAME"), otlp_collector_endpoint)?;

    let kafru_db = Arc::new(
        kafru::database::Db::new(None)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to connect to kafru DB: {}", e))?,
    );
    let kafru_queue = Arc::new(kafru::queue::Queue::new(Some(kafru_db.clone())).await);

    worker::usecase::collections::seed_from_env().await?;

    // TTS エンジンの設定は起動時に検証する
    let tts_config = audio_generator::config::TtsConfig::from_env()
        .map_err(|e| e.context("invalid TTS engine config"))?;
    let provider = Arc::new(Provider::new(kafru_queue, &tts_config)?);
    start_worker(provider.clone(), kafru_db.clone());
    start_schedule_sync(provider.clone(), kafru_db);
    start_api(provider).await
}

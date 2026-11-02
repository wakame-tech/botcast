//! テスト用の部品（feature `testing`）。アプリのテストからも使う

use crate::safe_fetch::{ContentKind, FetchedContent, UrlFetcher};
use async_trait::async_trait;
use futures::future::BoxFuture;
use opentelemetry::trace::TracerProvider as _;
use opentelemetry_sdk::export::trace::{ExportResult, SpanData, SpanExporter};
use opentelemetry_sdk::trace::TracerProvider;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tracing_subscriber::layer::SubscriberExt;

/// 書き出した span をメモリに貯める exporter
#[derive(Debug, Clone, Default)]
pub struct InMemorySpanExporter(Arc<Mutex<Vec<SpanData>>>);

impl InMemorySpanExporter {
    pub fn spans(&self) -> Vec<SpanData> {
        self.0.lock().unwrap().clone()
    }
}

impl SpanExporter for InMemorySpanExporter {
    fn export(&mut self, batch: Vec<SpanData>) -> BoxFuture<'static, ExportResult> {
        self.0.lock().unwrap().extend(batch);
        Box::pin(std::future::ready(Ok(())))
    }
}

/// span をメモリに書き出す subscriber を作る。テストの中で `tracing::subscriber::with_default` に渡す
pub fn test_subscriber() -> (
    impl tracing::Subscriber,
    InMemorySpanExporter,
    TracerProvider,
) {
    let exporter = InMemorySpanExporter::default();
    let provider = TracerProvider::builder()
        .with_simple_exporter(exporter.clone())
        .build();
    let layer = tracing_opentelemetry::layer().with_tracer(provider.tracer("test"));
    (
        tracing_subscriber::registry().with(layer),
        exporter,
        provider,
    )
}

/// URL ごとの内容を返すフェイク（無い URL はエラー）
#[derive(Default)]
pub struct FakeFetcher {
    pub pages: HashMap<String, String>,
    pub fetched: Mutex<Vec<String>>,
}

#[async_trait]
impl UrlFetcher for FakeFetcher {
    async fn fetch(&self, url: &str) -> anyhow::Result<FetchedContent> {
        self.fetched.lock().unwrap().push(url.to_string());
        let text = self
            .pages
            .get(url)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("host is not in the allowed hosts"))?;
        Ok(FetchedContent {
            url: url.to_string(),
            content_type: "application/json".to_string(),
            kind: ContentKind::Json,
            text,
            truncated: false,
        })
    }
}

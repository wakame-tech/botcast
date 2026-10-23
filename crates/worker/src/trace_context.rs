//! kafru のキューをまたいで OpenTelemetry の trace context（W3C `traceparent`）を引き継ぐ。
//!
//! ジョブを積むときに今の span の context をジョブの引数（`QueueData.parameters`）に入れ、
//! ジョブを実行するときにそれを親にして span を始める。これで HTTP の受け付けからジョブの実行までが 1 本のトレースになる。

use opentelemetry::propagation::TextMapPropagator;
use opentelemetry::trace::TraceContextExt;
use opentelemetry::Context;
use opentelemetry_sdk::propagation::TraceContextPropagator;
use serde_json::Value;
use std::collections::HashMap;
use tracing_opentelemetry::OpenTelemetrySpanExt;

/// W3C の `traceparent`（`00-{トレース ID}-{span ID}-{flags}`）を入れるジョブの引数のキー。
/// ジョブ一覧はここからトレース ID を取り出し、Jaeger へリンクする
pub(crate) const TRACEPARENT_PARAM: &str = "traceparent";

/// span の `traceparent`。トレースを送っていない（OpenTelemetry の layer が無い）なら `None`
pub(crate) fn traceparent(span: &tracing::Span) -> Option<String> {
    let mut carrier = HashMap::new();
    TraceContextPropagator::new().inject_context(&span.context(), &mut carrier);
    carrier.remove("traceparent")
}

/// ジョブの引数に今の span の `traceparent` を入れる
pub(crate) fn insert_into_params(params: &mut HashMap<String, Value>, span: &tracing::Span) {
    if let Some(traceparent) = traceparent(span) {
        params.insert(TRACEPARENT_PARAM.to_string(), Value::String(traceparent));
    }
}

/// ジョブの引数の `traceparent` を context にする。持たない（以前に積んだ・定期実行の）・壊れているなら `None`
pub(crate) fn extract_from_params(params: &HashMap<String, Value>) -> Option<Context> {
    let traceparent = params.get(TRACEPARENT_PARAM)?.as_str()?;
    let carrier = HashMap::from([("traceparent".to_string(), traceparent.to_string())]);
    let cx = TraceContextPropagator::new().extract(&carrier);
    cx.span().span_context().is_valid().then_some(cx)
}

/// ジョブの引数の `traceparent` のトレース ID（32 桁の 16 進数）
pub(crate) fn trace_id_from_params(params: &HashMap<String, Value>) -> Option<String> {
    let cx = extract_from_params(params)?;
    let trace_id = cx.span().span_context().trace_id();
    Some(trace_id.to_string())
}

/// span の status を Error にしてメッセージを付ける
pub(crate) fn record_error(span: &tracing::Span, message: &str) {
    span.record("otel.status_code", "ERROR");
    span.record("otel.status_message", message);
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use futures::future::BoxFuture;
    use opentelemetry::trace::TracerProvider as _;
    use opentelemetry_sdk::export::trace::{ExportResult, SpanData, SpanExporter};
    use opentelemetry_sdk::trace::TracerProvider;
    use std::sync::{Arc, Mutex};
    use tracing_subscriber::layer::SubscriberExt;

    /// 書き出した span をメモリに貯める exporter
    #[derive(Debug, Clone, Default)]
    pub(crate) struct InMemorySpanExporter(Arc<Mutex<Vec<SpanData>>>);

    impl InMemorySpanExporter {
        pub(crate) fn spans(&self) -> Vec<SpanData> {
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
    pub(crate) fn test_subscriber() -> (
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

    #[test]
    fn params_round_trip_parent_context() {
        let (subscriber, _exporter, _provider) = test_subscriber();
        tracing::subscriber::with_default(subscriber, || {
            let span = tracing::info_span!("create_job");
            let mut params = HashMap::new();
            insert_into_params(&mut params, &span);

            let traceparent = params[TRACEPARENT_PARAM].as_str().unwrap();
            let id = trace_id_from_params(&params).unwrap();
            assert_eq!(id.len(), 32);
            assert!(
                traceparent.starts_with(&format!("00-{id}-")),
                "{traceparent}"
            );
            let parent = extract_from_params(&params).unwrap();
            assert_eq!(parent.span().span_context().trace_id().to_string(), id);
        });
    }

    #[test]
    fn params_without_trace_context() {
        // 以前に積んだジョブ・定期実行のジョブは traceparent を持たない
        let params = HashMap::from([("args".to_string(), serde_json::json!({}))]);
        assert!(extract_from_params(&params).is_none());
        assert!(trace_id_from_params(&params).is_none());

        let broken = HashMap::from([(TRACEPARENT_PARAM.to_string(), serde_json::json!("garbage"))]);
        assert!(trace_id_from_params(&broken).is_none());
    }

    #[test]
    fn no_trace_context_without_otel_layer() {
        // OpenTelemetry の layer が無ければ何も入れない
        let span = tracing::info_span!("create_job");
        let mut params = HashMap::new();
        insert_into_params(&mut params, &span);
        assert!(params.is_empty());
    }
}

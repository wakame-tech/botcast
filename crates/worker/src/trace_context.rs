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

/// trace context（`traceparent` / `tracestate`）を入れるジョブの引数のキー
pub(crate) const TRACE_CONTEXT_PARAM: &str = "trace_context";
/// トレース ID（32 桁の 16 進数）を入れるジョブの引数のキー。ジョブ一覧から Jaeger へリンクするのに使う
pub(crate) const TRACE_ID_PARAM: &str = "trace_id";

/// span の context を W3C trace context のヘッダー（`traceparent` / `tracestate`）にする。
/// トレースを送っていない（span が無い・OpenTelemetry の layer が無い）なら空
pub(crate) fn inject(span: &tracing::Span) -> HashMap<String, String> {
    let mut carrier = HashMap::new();
    TraceContextPropagator::new().inject_context(&span.context(), &mut carrier);
    carrier
}

/// W3C trace context のヘッダーから親の context を取り出す。無い・壊れているなら `None`
pub(crate) fn extract(carrier: &HashMap<String, String>) -> Option<Context> {
    let cx = TraceContextPropagator::new().extract(carrier);
    cx.span().span_context().is_valid().then_some(cx)
}

/// span のトレース ID（32 桁の 16 進数）。トレースを送っていないなら `None`
pub(crate) fn trace_id(span: &tracing::Span) -> Option<String> {
    let cx = span.context();
    let span_context = cx.span().span_context().clone();
    span_context
        .is_valid()
        .then(|| span_context.trace_id().to_string())
}

/// ジョブの引数に今の span の trace context とトレース ID を入れる
pub(crate) fn insert_into_params(params: &mut HashMap<String, Value>, span: &tracing::Span) {
    let carrier = inject(span);
    if carrier.is_empty() {
        return;
    }
    if let Some(id) = trace_id(span) {
        params.insert(TRACE_ID_PARAM.to_string(), Value::String(id));
    }
    params.insert(
        TRACE_CONTEXT_PARAM.to_string(),
        serde_json::to_value(carrier).unwrap_or_default(),
    );
}

/// ジョブの引数から親の context を取り出す。trace context を持たない（以前に積んだ・定期実行の）ジョブは `None`
pub(crate) fn extract_from_params(params: &HashMap<String, Value>) -> Option<Context> {
    let carrier: HashMap<String, String> =
        serde_json::from_value(params.get(TRACE_CONTEXT_PARAM)?.clone()).ok()?;
    extract(&carrier)
}

/// ジョブの引数に入っているトレース ID
pub(crate) fn trace_id_from_params(params: &HashMap<String, Value>) -> Option<String> {
    params
        .get(TRACE_ID_PARAM)
        .and_then(Value::as_str)
        .map(str::to_string)
}

/// HTTP のヘッダーに span の trace context（`traceparent` / `tracestate`）を入れる（既にあれば置き換える）
pub(crate) fn inject_into_headers(headers: &mut axum::http::HeaderMap, span: &tracing::Span) {
    for (key, value) in inject(span) {
        if let (Ok(name), Ok(value)) = (
            axum::http::HeaderName::try_from(key),
            axum::http::HeaderValue::try_from(value),
        ) {
            headers.insert(name, value);
        }
    }
}

/// HTTP のヘッダーから親の context を取り出す。無い・壊れているなら `None`
pub(crate) fn extract_from_headers(headers: &axum::http::HeaderMap) -> Option<Context> {
    let carrier: HashMap<String, String> = ["traceparent", "tracestate"]
        .into_iter()
        .filter_map(|key| {
            let value = headers.get(key)?.to_str().ok()?;
            Some((key.to_string(), value.to_string()))
        })
        .collect();
    extract(&carrier)
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

            let traceparent = params[TRACE_CONTEXT_PARAM]["traceparent"].as_str().unwrap();
            let id = trace_id_from_params(&params).unwrap();
            assert_eq!(id.len(), 32);
            assert!(
                traceparent.starts_with(&format!("00-{id}-")),
                "{traceparent}"
            );
            assert_eq!(Some(id.clone()), trace_id(&span));

            let parent = extract_from_params(&params).unwrap();
            assert_eq!(parent.span().span_context().trace_id().to_string(), id);
        });
    }

    #[test]
    fn headers_round_trip_parent_context() {
        let (subscriber, _exporter, _provider) = test_subscriber();
        tracing::subscriber::with_default(subscriber, || {
            let span = tracing::info_span!("request");
            let mut headers = axum::http::HeaderMap::new();
            // クライアントが送ってきた traceparent は worker の span で置き換える
            headers.insert(
                "traceparent",
                "00-0af7651916cd43dd8448eb211c80319c-b7ad6b7169203331-01"
                    .parse()
                    .unwrap(),
            );
            inject_into_headers(&mut headers, &span);
            let parent = extract_from_headers(&headers).unwrap();
            assert_eq!(
                Some(parent.span().span_context().trace_id().to_string()),
                trace_id(&span)
            );
        });
        assert!(extract_from_headers(&axum::http::HeaderMap::new()).is_none());
    }

    #[test]
    fn params_without_trace_context() {
        // 以前に積んだジョブ・定期実行のジョブは trace context を持たない
        let params = HashMap::from([("args".to_string(), serde_json::json!({}))]);
        assert!(extract_from_params(&params).is_none());
        assert!(trace_id_from_params(&params).is_none());

        let broken = HashMap::from([(
            TRACE_CONTEXT_PARAM.to_string(),
            serde_json::json!({ "traceparent": "garbage" }),
        )]);
        assert!(extract_from_params(&broken).is_none());
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

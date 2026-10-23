use crate::trace_context;
use crate::usecase::provider::Provider;
use crate::usecase::task_service::Args;
use async_trait::async_trait;
use kafru::task::{RecordId, TaskHandler};
use serde_json::Value;
use std::{
    collections::HashMap,
    sync::{Arc, OnceLock},
};
use tracing::{field::Empty, Instrument};
use tracing_opentelemetry::OpenTelemetrySpanExt;

static PROVIDER: OnceLock<Arc<Provider>> = OnceLock::new();

pub(crate) fn init_provider(provider: Arc<Provider>) {
    let _ = PROVIDER.set(provider);
}

pub(crate) fn create() -> Box<dyn TaskHandler> {
    Box::new(ExecuteTaskJob)
}

pub(crate) struct ExecuteTaskJob;

/// ジョブ 1 件の span を作る。ジョブを積んだときの trace context があればそれを親にする
/// （無ければ新しいトレースを始める）
pub(crate) fn job_span(
    params: &HashMap<String, Value>,
    queue_id: Option<&RecordId>,
) -> tracing::Span {
    let span = tracing::info_span!(
        "execute_task",
        otel.kind = "consumer",
        job.id = queue_id.map(|id| id.to_string()),
        otel.status_code = Empty,
        otel.status_message = Empty,
    );
    if let Some(parent) = trace_context::extract_from_params(params) {
        span.set_parent(parent);
    }
    span
}

/// ジョブの結果を span に書く。失敗なら status を Error にしてメッセージを付ける
pub(crate) fn record_result(span: &tracing::Span, result: &Result<(), String>) {
    match result {
        Ok(()) => {
            span.record("otel.status_code", "OK");
        }
        Err(e) => {
            span.record("otel.status_code", "ERROR");
            span.record("otel.status_message", e.as_str());
        }
    }
}

#[async_trait]
impl TaskHandler for ExecuteTaskJob {
    async fn run(
        &self,
        params: HashMap<String, Value>,
        queue_id: Option<RecordId>,
        _agent_id: Option<RecordId>,
    ) -> Result<(), String> {
        let span = job_span(&params, queue_id.as_ref());
        let result = run(params, queue_id).instrument(span.clone()).await;
        if let Err(e) = &result {
            tracing::error!(parent: &span, error = %e, "job failed");
        }
        record_result(&span, &result);
        result
    }
}

async fn run(params: HashMap<String, Value>, queue_id: Option<RecordId>) -> Result<(), String> {
    let provider = PROVIDER.get().ok_or("Provider not initialized")?;
    let task_service = provider.task_service();
    // trace context を持たずに積まれたジョブ（定期実行・以前のジョブ）は、ここで始めたトレースの ID を書き足す
    if let (Some(job_id), None) = (queue_id, trace_context::trace_id_from_params(&params)) {
        if let Some(trace_id) = trace_context::trace_id(&tracing::Span::current()) {
            if let Err(e) = task_service
                .record_trace_id(job_id, params.clone(), trace_id)
                .await
            {
                tracing::warn!("{e}");
            }
        }
    }
    let args_value = params
        .get("args")
        .ok_or("args not found in params")?
        .clone();
    let args: Args = serde_json::from_value(args_value).map_err(|e| e.to_string())?;
    task_service
        .execute_args(args)
        .await
        // context の連鎖まで含めてジョブのメッセージ・span に残す
        .map_err(|crate::error::Error::Other(e)| format!("{e:#}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::trace_context::tests::test_subscriber;
    use opentelemetry::trace::Status;

    #[test]
    fn job_span_continues_trace_of_create_job() {
        let (subscriber, exporter, provider) = test_subscriber();
        tracing::subscriber::with_default(subscriber, || {
            let mut params = HashMap::new();
            let create_job = tracing::info_span!("create_job");
            create_job.in_scope(|| {
                trace_context::insert_into_params(&mut params, &tracing::Span::current());
            });
            drop(create_job);

            let job = job_span(&params, None);
            job.in_scope(|| tracing::info_span!("generate_audio").in_scope(|| {}));
            record_result(&job, &Err("boom".to_string()));
            drop(job);
        });
        provider.force_flush();

        let spans = exporter.spans();
        let get = |name: &str| spans.iter().find(|s| s.name == name).unwrap();
        let (create_job, job, child) = (
            get("create_job"),
            get("execute_task"),
            get("generate_audio"),
        );
        // HTTP → ジョブ → ジョブの中の span が 1 本のトレースになる
        assert_eq!(
            job.span_context.trace_id(),
            create_job.span_context.trace_id()
        );
        assert_eq!(job.parent_span_id, create_job.span_context.span_id());
        assert_eq!(child.parent_span_id, job.span_context.span_id());
        assert_eq!(job.status, Status::error("boom"));
    }

    #[test]
    fn job_span_without_trace_context_starts_new_trace() {
        let (subscriber, exporter, provider) = test_subscriber();
        tracing::subscriber::with_default(subscriber, || {
            let job = job_span(&HashMap::new(), None);
            assert!(trace_context::trace_id(&job).is_some());
            record_result(&job, &Ok(()));
        });
        provider.force_flush();
        let spans = exporter.spans();
        assert_eq!(spans.len(), 1);
        assert_eq!(spans[0].status, Status::Ok);
    }
}

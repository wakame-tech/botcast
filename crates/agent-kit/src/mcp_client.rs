//! MCP サーバー（子プロセス・stdio）のクライアント。
//!
//! - アプリが固定で入れる引数（botcast-cms の `tenantId` など）は LLM に見せるスキーマから外し、呼ぶときに上書きで入れる。
//!   LLM の出力でアプリの境界（テナントなど）を越えさせない
//! - ツールの呼び出しは span `execute_tool {name}` にし、`params._meta.traceparent` に今の span の W3C `traceparent` を入れる
//!   （OpenTelemetry の MCP の semantic conventions）。サーバーが取り出せば、サーバー側の span も同じトレースになる

use crate::trace_context::{record_error, traceparent};
use anyhow::Context;
use rmcp::{
    ServiceExt,
    model::{CallToolRequestParams, JsonObject, Meta, Tool},
    service::RunningService,
    transport::{ConfigureCommandExt, TokioChildProcess},
};
use serde_json::Value;
use tokio::process::Command;
use tracing::{Instrument, field::Empty};

pub struct McpClient {
    service: RunningService<rmcp::RoleClient, ()>,
    /// LLM に見せるツール定義。`fixed_args` のキーは取り除いてある
    tools: Vec<Tool>,
    /// 毎回の呼び出しに入れる引数（LLM の引数を上書きする）
    fixed_args: JsonObject,
}

/// `fixed_args` のキーを入力スキーマ（`properties` / `required`）から外す。LLM に選ばせない
fn strip_fixed_params(mut tool: Tool, fixed_args: &JsonObject) -> Tool {
    if fixed_args.is_empty() {
        return tool;
    }
    let schema = std::sync::Arc::make_mut(&mut tool.input_schema);
    if let Some(Value::Object(props)) = schema.get_mut("properties") {
        props.retain(|k, _| !fixed_args.contains_key(k));
    }
    if let Some(Value::Array(required)) = schema.get_mut("required") {
        required.retain(|v| v.as_str().is_none_or(|k| !fixed_args.contains_key(k)));
    }
    tool
}

/// LLM の引数に `fixed_args` を上書きで入れる
fn merge_args(arguments: Value, fixed_args: &JsonObject) -> JsonObject {
    let mut arguments = arguments.as_object().cloned().unwrap_or_default();
    for (k, v) in fixed_args {
        arguments.insert(k.clone(), v.clone());
    }
    arguments
}

/// span の `traceparent` を入れた `_meta`。トレースを送っていなければ `None`
fn trace_meta(span: &tracing::Span) -> Option<Meta> {
    let traceparent = traceparent(span)?;
    let mut meta = Meta::new();
    meta.0.insert("traceparent".to_string(), Value::String(traceparent));
    Some(meta)
}

impl McpClient {
    /// `mcp_cmd mcp_args...` を子プロセスで起動してつなぐ。`fixed_args` は毎回の呼び出しに入れる引数
    pub async fn new(mcp_cmd: &str, mcp_args: &[&str], fixed_args: JsonObject) -> anyhow::Result<Self> {
        Self::spawn(
            Command::new(mcp_cmd).configure(|cmd| {
                cmd.args(mcp_args);
            }),
            fixed_args,
        )
        .await
    }

    /// 組み立てた `command`（環境変数なども呼び出し側が決める）を子プロセスで起動してつなぐ
    pub async fn spawn(command: Command, fixed_args: JsonObject) -> anyhow::Result<Self> {
        let transport =
            TokioChildProcess::new(command).context("Failed to spawn MCP server process")?;

        let service = ().serve(transport).await.context("Failed to connect to MCP server")?;
        let tools = service.list_all_tools().await.context("Failed to list MCP tools")?;
        let tools = tools
            .into_iter()
            .map(|t| strip_fixed_params(t, &fixed_args))
            .collect();

        Ok(Self {
            service,
            tools,
            fixed_args,
        })
    }

    pub fn tools(&self) -> &[Tool] {
        &self.tools
    }

    /// ツールを呼び、結果のテキストを返す。
    /// span にはツール名と成否（`mcp.tool.is_error` = ツールがエラーを返したか）だけを入れ、引数・結果の本文は入れない
    pub async fn call_tool(
        &self,
        name: impl Into<std::borrow::Cow<'static, str>>,
        arguments: Value,
    ) -> anyhow::Result<String> {
        let name = name.into();
        let span = tracing::info_span!(
            "execute_tool",
            otel.name = format!("execute_tool {name}"),
            otel.kind = "client",
            gen_ai.operation.name = "execute_tool",
            gen_ai.tool.name = %name,
            mcp.tool.is_error = Empty,
            otel.status_code = Empty,
            otel.status_message = Empty,
        );
        let meta = trace_meta(&span);
        let result = self
            .call_tool_inner(name, arguments, meta)
            .instrument(span.clone())
            .await;
        match &result {
            Ok((_, is_error)) => {
                span.record("mcp.tool.is_error", is_error);
                if *is_error {
                    record_error(&span, "tool returned an error");
                }
            }
            Err(e) => record_error(&span, &format!("{e:#}")),
        }
        result.map(|(text, _)| text)
    }

    /// ツールを呼び、結果のテキストと、ツールがエラーを返したかを返す
    async fn call_tool_inner(
        &self,
        name: std::borrow::Cow<'static, str>,
        arguments: Value,
        meta: Option<Meta>,
    ) -> anyhow::Result<(String, bool)> {
        let mut params =
            CallToolRequestParams::new(name).with_arguments(merge_args(arguments, &self.fixed_args));
        params.meta = meta;
        let result = self
            .service
            .call_tool(params)
            .await
            .context("Failed to call MCP tool")?;

        let text = result
            .content
            .iter()
            .filter_map(|c| {
                if let rmcp::model::RawContent::Text(t) = &c.raw {
                    Some(t.text.as_str())
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
            .join("\n");

        Ok((text, result.is_error.unwrap_or(false)))
    }

    pub async fn close(self) -> anyhow::Result<()> {
        self.service.cancel().await.context("Failed to close MCP client")?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::test_subscriber;
    use serde_json::json;

    fn fixed() -> JsonObject {
        json!({ "tenantId": "botcast" }).as_object().cloned().unwrap()
    }

    #[test]
    fn strip_fixed_params_removes_fixed_keys_from_schema() {
        let schema = json!({
            "type": "object",
            "properties": {
                "tenantId": { "type": "string" },
                "collectionId": { "type": "string" }
            },
            "required": ["tenantId", "collectionId"]
        });
        let schema = schema.as_object().cloned().unwrap();
        let tool = Tool::new("RecordApi_list", "list records", std::sync::Arc::new(schema));

        let tool = strip_fixed_params(tool, &fixed());

        let props = tool.input_schema["properties"].as_object().unwrap();
        assert!(!props.contains_key("tenantId"), "tenantId を LLM に見せない");
        assert!(props.contains_key("collectionId"));
        let required: Vec<&str> = tool.input_schema["required"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap())
            .collect();
        assert_eq!(required, vec!["collectionId"]);
    }

    #[test]
    fn strip_fixed_params_keeps_schema_without_fixed_args() {
        let schema = json!({ "type": "object", "properties": { "tenantId": {} }, "required": ["tenantId"] });
        let tool = Tool::new("t", "d", std::sync::Arc::new(schema.as_object().cloned().unwrap()));
        let tool = strip_fixed_params(tool, &JsonObject::new());
        assert!(tool.input_schema["properties"].as_object().unwrap().contains_key("tenantId"));
    }

    #[test]
    fn merge_args_overrides_llm_values_with_fixed_args() {
        let merged = merge_args(json!({ "tenantId": "other", "collectionId": "c" }), &fixed());
        assert_eq!(Value::Object(merged), json!({ "tenantId": "botcast", "collectionId": "c" }));
        // オブジェクトでない引数は空として扱う
        assert_eq!(Value::Object(merge_args(json!(null), &fixed())), json!({ "tenantId": "botcast" }));
    }

    #[test]
    fn trace_meta_carries_traceparent_of_span() {
        let (subscriber, _exporter, _provider) = test_subscriber();
        tracing::subscriber::with_default(subscriber, || {
            let span = tracing::info_span!("execute_tool");
            let meta = trace_meta(&span).expect("traceparent");
            let tp = meta.0["traceparent"].as_str().unwrap();
            assert_eq!(Some(tp.to_string()), traceparent(&span));
            assert!(tp.starts_with("00-"), "{tp}");
        });
    }

    #[test]
    fn trace_meta_is_none_without_otel_layer() {
        let span = tracing::info_span!("execute_tool");
        assert!(trace_meta(&span).is_none());
    }
}

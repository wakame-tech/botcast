use crate::trace_context::record_error;
use anyhow::Context;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use tracing::{Instrument, field::Empty};

use super::mcp_client::McpClient;
use rmcp::model::Tool;

const ANTHROPIC_API_URL: &str = "https://api.anthropic.com/v1/messages";
const ANTHROPIC_VERSION: &str = "2023-06-01";
/// thinking も含めた 1 回の応答の上限。台本全体を 1 回の応答で返すので余裕を持たせる
const MAX_TOKENS: u32 = 16000;
const MAX_ITERATIONS: usize = 20;

/// エージェントに渡す CMS の MCP ツール（読み取りだけ）。書き込み系（`RecordApi_update` など）は渡さない。
/// 保存は worker が LLM の出力を検証してから行う
pub(crate) const READ_ONLY_CMS_TOOLS: &[&str] = &[
    "CollectionApi_list",
    "CollectionApi_read",
    "RecordApi_list",
    "RecordApi_listWithRelations",
    "RecordApi_read",
    "RecordApi_referencedBy",
];

#[derive(Debug, Serialize, Deserialize)]
struct AnthropicTool {
    name: String,
    description: String,
    input_schema: Value,
}

#[derive(Debug, Serialize, Deserialize)]
struct Message {
    role: String,
    content: Value,
}

#[derive(Debug, Deserialize)]
struct AnthropicResponse {
    /// 応答のブロックをそのまま持つ。次のターンでは assistant の発言としてこのまま送り返す
    /// （thinking ブロックは署名ごと変えずに返す必要がある）
    content: Vec<Value>,
    stop_reason: String,
    #[serde(default)]
    model: Option<String>,
    #[serde(default)]
    usage: Option<Usage>,
    /// `stop_reason` が `refusal` のときの理由
    #[serde(default)]
    stop_details: Option<Value>,
}

/// トークン数（span の属性に使う）
#[derive(Debug, Default, Deserialize)]
struct Usage {
    #[serde(default)]
    input_tokens: u64,
    #[serde(default)]
    output_tokens: u64,
    #[serde(default)]
    cache_creation_input_tokens: Option<u64>,
    #[serde(default)]
    cache_read_input_tokens: Option<u64>,
}

/// LLM の呼び出し 1 回の span（OpenTelemetry の GenAI の semantic conventions に合わせる）。
/// プロンプト・応答の本文は入れない
fn llm_span(model: &str) -> tracing::Span {
    tracing::info_span!(
        "chat",
        otel.name = format!("chat {model}"),
        otel.kind = "client",
        gen_ai.operation.name = "chat",
        gen_ai.system = "anthropic",
        gen_ai.request.model = model,
        gen_ai.request.max_tokens = MAX_TOKENS,
        gen_ai.response.model = Empty,
        gen_ai.response.finish_reasons = Empty,
        gen_ai.usage.input_tokens = Empty,
        gen_ai.usage.output_tokens = Empty,
        gen_ai.usage.cache_creation_input_tokens = Empty,
        gen_ai.usage.cache_read_input_tokens = Empty,
        http.response.status_code = Empty,
        otel.status_code = Empty,
        otel.status_message = Empty,
    )
}

/// LLM の応答（モデル・停止理由・トークン数）を span に書く
fn record_llm_response(span: &tracing::Span, resp: &AnthropicResponse) {
    if let Some(model) = &resp.model {
        span.record("gen_ai.response.model", model.as_str());
    }
    span.record("gen_ai.response.finish_reasons", resp.stop_reason.as_str());
    if let Some(usage) = &resp.usage {
        span.record("gen_ai.usage.input_tokens", usage.input_tokens);
        span.record("gen_ai.usage.output_tokens", usage.output_tokens);
        if let Some(n) = usage.cache_creation_input_tokens {
            span.record("gen_ai.usage.cache_creation_input_tokens", n);
        }
        if let Some(n) = usage.cache_read_input_tokens {
            span.record("gen_ai.usage.cache_read_input_tokens", n);
        }
    }
}

/// 応答のブロックのうちエージェントが使うもの。`thinking` / `redacted_thinking` などそれ以外は `Other`
#[derive(Debug, PartialEq, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum ContentBlock {
    Text {
        text: String,
    },
    ToolUse {
        id: String,
        name: String,
        input: Value,
    },
    #[serde(other)]
    Other,
}

impl AnthropicResponse {
    fn blocks(&self) -> anyhow::Result<Vec<ContentBlock>> {
        self.content
            .iter()
            .map(|b| {
                serde_json::from_value(b.clone()).context("Failed to parse Anthropic content block")
            })
            .collect()
    }
}

/// エージェントのループの 1 ターンの結果
enum TurnOutcome {
    /// LLM が答えを返して終わった（その本文）
    Done(String),
    /// ツールを呼んだので次のターンに進む
    Continue,
}

/// MCP のツールのうち `allowed` に入っているものだけを、Anthropic のツール定義にする
fn tools_for_anthropic(tools: &[Tool], allowed: &[&str]) -> Vec<AnthropicTool> {
    tools
        .iter()
        .filter(|t| allowed.contains(&t.name.as_ref()))
        .map(|t| AnthropicTool {
            name: t.name.to_string(),
            description: t.description.as_deref().unwrap_or("").to_string(),
            input_schema: Value::Object((*t.input_schema).clone()),
        })
        .collect()
}

/// 最終応答の本文（text ブロックをつなげたもの）
fn final_text(blocks: &[ContentBlock]) -> String {
    blocks
        .iter()
        .filter_map(|b| {
            if let ContentBlock::Text { text } = b {
                Some(text.as_str())
            } else {
                None
            }
        })
        .collect::<String>()
}

/// リクエストの本文。`output_schema` があれば structured outputs（`output_config.format`）で
/// 最終応答の形を固定する（ツールの呼び出しはそのまま使える）
fn request_body(
    model: &str,
    system: &str,
    tools: &[AnthropicTool],
    messages: &[Message],
    output_schema: Option<&Value>,
) -> Value {
    let mut body = json!({
        "model": model,
        "max_tokens": MAX_TOKENS,
        "system": system,
        "tools": tools,
        "messages": messages,
    });
    if let Some(schema) = output_schema {
        body["output_config"] = json!({
            "format": { "type": "json_schema", "schema": schema }
        });
    }
    body
}

pub struct AgentService {
    api_key: String,
    model: String,
    mcp_client: McpClient,
    /// LLM に渡す（呼ばせる）ツールの名前
    allowed_tools: &'static [&'static str],
    http_client: reqwest::Client,
}

impl AgentService {
    /// `model` は API に渡すモデルの ID（`llm_models` から決める。[`super::llm_models::resolve_model`]）。
    /// `allowed_tools` は MCP のツールのうち LLM に渡すもの（例: [`READ_ONLY_CMS_TOOLS`]）
    pub fn new(
        api_key: String,
        model: String,
        mcp_client: McpClient,
        allowed_tools: &'static [&'static str],
    ) -> Self {
        Self {
            api_key,
            model,
            mcp_client,
            allowed_tools,
            http_client: reqwest::Client::new(),
        }
    }

    #[tracing::instrument(
        name = "agent.run",
        skip_all,
        fields(gen_ai.request.model = %self.model, agent.turns = Empty),
        err
    )]
    /// エージェントのループを回し、最終応答の本文を返す。
    /// `output_schema` があれば最終応答はそのスキーマの JSON になる
    pub async fn run(
        &self,
        system: &str,
        prompt: &str,
        output_schema: Option<&Value>,
    ) -> anyhow::Result<String> {
        let tools = tools_for_anthropic(self.mcp_client.tools(), self.allowed_tools);
        let mut messages: Vec<Message> = vec![Message {
            role: "user".to_string(),
            content: json!(prompt),
        }];

        for turn in 0..MAX_ITERATIONS {
            tracing::Span::current().record("agent.turns", turn + 1);
            let turn_span = tracing::info_span!(
                "agent.turn",
                agent.turn = turn,
                gen_ai.response.finish_reasons = Empty,
                agent.tool_calls = Empty,
            );
            let outcome = self
                .run_turn(system, &tools, &mut messages, output_schema)
                .instrument(turn_span.clone())
                .await;
            match outcome {
                Ok(TurnOutcome::Done(text)) => return Ok(text),
                Ok(TurnOutcome::Continue) => continue,
                Err(e) => {
                    record_error(&turn_span, &format!("{e:#}"));
                    return Err(e);
                }
            }
        }

        anyhow::bail!("Agent exceeded max iterations ({})", MAX_ITERATIONS)
    }

    /// エージェントのループの 1 ターン: LLM を呼び、ツールを使うならツールを呼んで結果を `messages` に足す
    async fn run_turn(
        &self,
        system: &str,
        tools: &[AnthropicTool],
        messages: &mut Vec<Message>,
        output_schema: Option<&Value>,
    ) -> anyhow::Result<TurnOutcome> {
        let body = request_body(&self.model, system, tools, messages, output_schema);

        let llm_span = llm_span(&self.model);
        let resp = match self
            .call_llm(&body, &llm_span)
            .instrument(llm_span.clone())
            .await
        {
            Ok(resp) => resp,
            Err(e) => {
                record_error(&llm_span, &format!("{e:#}"));
                return Err(e);
            }
        };
        tracing::Span::current()
            .record("gen_ai.response.finish_reasons", resp.stop_reason.as_str());

        let blocks = resp.blocks()?;
        messages.push(Message {
            role: "assistant".to_string(),
            content: Value::Array(resp.content.clone()),
        });

        match resp.stop_reason.as_str() {
            "end_turn" | "stop_sequence" => return Ok(TurnOutcome::Done(final_text(&blocks))),
            "tool_use" => {}
            // 出力が途中で切れている（structured outputs でもスキーマを満たさない）
            "max_tokens" => anyhow::bail!(
                "LLM response was truncated at max_tokens ({MAX_TOKENS})"
            ),
            "refusal" => anyhow::bail!(
                "LLM refused to respond (category: {})",
                resp.stop_details
                    .as_ref()
                    .and_then(|d| d["category"].as_str())
                    .unwrap_or("unknown")
            ),
            other => anyhow::bail!("Unexpected stop_reason: {other}"),
        }

        let mut tool_results = Vec::new();
        for block in &blocks {
            if let ContentBlock::ToolUse { id, name, input } = block {
                // 渡していないツール（書き込み系など）は呼ばない
                if !self.allowed_tools.contains(&name.as_str()) {
                    tracing::warn!("Agent requested a tool that is not allowed: {name}");
                    tool_results.push(json!({
                        "type": "tool_result",
                        "tool_use_id": id,
                        "content": format!("Tool {name} is not available"),
                        "is_error": true,
                    }));
                    continue;
                }
                tracing::info!("Agent calling tool: {} with {:?}", name, input);
                let result = self
                    .mcp_client
                    .call_tool(name.clone(), input.clone())
                    .await
                    .unwrap_or_else(|e| format!("Tool error: {}", e));
                tool_results.push(json!({
                    "type": "tool_result",
                    "tool_use_id": id,
                    "content": result,
                }));
            }
        }
        tracing::Span::current().record("agent.tool_calls", tool_results.len());
        messages.push(Message {
            role: "user".to_string(),
            content: json!(tool_results),
        });
        Ok(TurnOutcome::Continue)
    }

    /// Anthropic の Messages API を 1 回呼ぶ。応答のモデル・停止理由・トークン数を `span` に書く
    async fn call_llm(
        &self,
        body: &Value,
        span: &tracing::Span,
    ) -> anyhow::Result<AnthropicResponse> {
        let response = self
            .http_client
            .post(ANTHROPIC_API_URL)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", ANTHROPIC_VERSION)
            .header("content-type", "application/json")
            .json(body)
            .send()
            .await
            .context("Failed to call Anthropic API")?;

        span.record("http.response.status_code", response.status().as_u16());
        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("Anthropic API error {}: {}", status, body);
        }

        let resp: AnthropicResponse = response
            .json()
            .await
            .context("Failed to parse Anthropic response")?;
        record_llm_response(span, &resp);
        Ok(resp)
    }

    pub async fn close(self) -> anyhow::Result<()> {
        self.mcp_client.close().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 5.5 系のモデルは thinking が常に有効で、応答に `thinking` ブロックが入る
    fn response_with_thinking() -> AnthropicResponse {
        serde_json::from_value(json!({
            "content": [
                { "type": "thinking", "thinking": "", "signature": "sig-abc" },
                { "type": "redacted_thinking", "data": "opaque" },
                { "type": "text", "text": "読みます" },
                { "type": "tool_use", "id": "toolu_1", "name": "RecordApi_read", "input": { "recordId": "r1" } }
            ],
            "stop_reason": "tool_use",
            "model": "claude-opus-5-5"
        }))
        .unwrap()
    }

    #[test]
    fn parses_thinking_blocks_as_other() {
        let blocks = response_with_thinking().blocks().unwrap();
        assert_eq!(
            blocks,
            vec![
                ContentBlock::Other,
                ContentBlock::Other,
                ContentBlock::Text {
                    text: "読みます".to_string()
                },
                ContentBlock::ToolUse {
                    id: "toolu_1".to_string(),
                    name: "RecordApi_read".to_string(),
                    input: json!({ "recordId": "r1" }),
                },
            ]
        );
    }

    fn tool(name: &str) -> Tool {
        serde_json::from_value(json!({
            "name": name,
            "description": format!("{name} desc"),
            "inputSchema": { "type": "object", "properties": {} }
        }))
        .unwrap()
    }

    #[test]
    fn passes_only_read_only_cms_tools() {
        // botcast-cms の MCP サーバーが出すツール
        let all = [
            "BatchApi_execute", "CanvasApi_createEdge", "CanvasApi_deleteEdge", "CanvasApi_listEdges",
            "CanvasApi_query", "CollectionApi_create", "CollectionApi_delete", "CollectionApi_list",
            "CollectionApi_read", "CollectionApi_update", "RecordApi_copyRecord", "RecordApi_create",
            "RecordApi_delete", "RecordApi_deleteImage", "RecordApi_eval", "RecordApi_generate",
            "RecordApi_generateData", "RecordApi_getImage", "RecordApi_list",
            "RecordApi_listWithRelations", "RecordApi_mergeRecords", "RecordApi_read",
            "RecordApi_referencedBy", "RecordApi_splitRecord", "RecordApi_update",
            "RecordApi_uploadImage", "ScriptApi_execute",
        ];
        let tools: Vec<Tool> = all.iter().map(|n| tool(n)).collect();
        let passed: Vec<String> = tools_for_anthropic(&tools, READ_ONLY_CMS_TOOLS)
            .into_iter()
            .map(|t| t.name)
            .collect();
        assert_eq!(
            passed,
            vec![
                "CollectionApi_list", "CollectionApi_read", "RecordApi_list",
                "RecordApi_listWithRelations", "RecordApi_read", "RecordApi_referencedBy",
            ]
        );
        for write in ["RecordApi_update", "RecordApi_create", "RecordApi_delete", "BatchApi_execute"] {
            assert!(!passed.iter().any(|n| n == write), "{write}");
        }
    }

    #[test]
    fn request_body_sets_output_format_only_when_given() {
        let tools = tools_for_anthropic(&[tool("RecordApi_read")], READ_ONLY_CMS_TOOLS);
        let messages = vec![Message { role: "user".to_string(), content: json!("p") }];
        let schema = json!({ "type": "object", "additionalProperties": false });
        let body = request_body("claude-haiku-5-5", "sys", &tools, &messages, Some(&schema));
        assert_eq!(
            body["output_config"],
            json!({ "format": { "type": "json_schema", "schema": schema } })
        );
        assert_eq!(body["tools"][0]["name"], "RecordApi_read");
        // forced tool_choice は 5.5 系で 400 になるので使わない
        assert!(body.get("tool_choice").is_none());
        let body = request_body("m", "sys", &tools, &messages, None);
        assert!(body.get("output_config").is_none());
    }

    #[test]
    fn final_text_joins_text_blocks_without_separator() {
        let resp: AnthropicResponse = serde_json::from_value(json!({
            "content": [
                { "type": "thinking", "thinking": "", "signature": "s" },
                { "type": "text", "text": "{\"sections\":" },
                { "type": "text", "text": "[]}" }
            ],
            "stop_reason": "end_turn"
        }))
        .unwrap();
        assert_eq!(final_text(&resp.blocks().unwrap()), "{\"sections\":[]}");
    }

    #[test]
    fn raw_content_keeps_thinking_signature_for_replay() {
        let resp = response_with_thinking();
        let replay = Value::Array(resp.content.clone());
        assert_eq!(replay[0]["type"], "thinking");
        assert_eq!(replay[0]["signature"], "sig-abc");
        assert_eq!(replay[1]["data"], "opaque");
    }
}

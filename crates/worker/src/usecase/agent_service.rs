use crate::trace_context::record_error;
use anyhow::Context;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use tracing::{Instrument, field::Empty};

use super::mcp_client::McpClient;

const ANTHROPIC_API_URL: &str = "https://api.anthropic.com/v1/messages";
const ANTHROPIC_VERSION: &str = "2023-06-01";
const MAX_TOKENS: u32 = 8192;
const MAX_ITERATIONS: usize = 20;

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
    content: Vec<ContentBlock>,
    stop_reason: String,
    #[serde(default)]
    model: Option<String>,
    #[serde(default)]
    usage: Option<Usage>,
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


#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum ContentBlock {
    Text { text: String },
    ToolUse { id: String, name: String, input: Value },
}

/// エージェントのループの 1 ターンの結果
enum TurnOutcome {
    /// LLM が答えを返して終わった（その本文）
    Done(String),
    /// ツールを呼んだので次のターンに進む
    Continue,
}

pub struct AgentService {
    api_key: String,
    model: String,
    mcp_client: McpClient,
    http_client: reqwest::Client,
}

impl AgentService {
    /// `model` は API に渡すモデルの ID（`llm_models` から決める。[`super::llm_models::resolve_model`]）
    pub fn new(api_key: String, model: String, mcp_client: McpClient) -> Self {
        Self {
            api_key,
            model,
            mcp_client,
            http_client: reqwest::Client::new(),
        }
    }

    fn tools_for_anthropic(&self) -> Vec<AnthropicTool> {
        self.mcp_client
            .tools()
            .iter()
            .map(|t| AnthropicTool {
                name: t.name.to_string(),
                description: t.description.as_deref().unwrap_or("").to_string(),
                input_schema: Value::Object((*t.input_schema).clone()),
            })
            .collect()
    }

    #[tracing::instrument(
        name = "agent.run",
        skip_all,
        fields(gen_ai.request.model = %self.model, agent.turns = Empty),
        err
    )]
    pub async fn run(&self, system: &str, prompt: &str) -> anyhow::Result<String> {
        let tools = self.tools_for_anthropic();
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
                .run_turn(system, &tools, &mut messages)
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
    ) -> anyhow::Result<TurnOutcome> {
        let body = json!({
            "model": self.model,
            "max_tokens": MAX_TOKENS,
            "system": system,
            "tools": tools,
            "messages": messages,
        });

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

        let assistant_content: Value =
            serde_json::to_value(&resp.content).context("Failed to serialize assistant content")?;
        messages.push(Message {
            role: "assistant".to_string(),
            content: assistant_content,
        });

        if resp.stop_reason == "end_turn" || resp.stop_reason == "stop_sequence" {
            let text = resp
                .content
                .iter()
                .filter_map(|b| {
                    if let ContentBlock::Text { text } = b {
                        Some(text.as_str())
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>()
                .join("\n");
            return Ok(TurnOutcome::Done(text));
        }

        if resp.stop_reason == "tool_use" {
            let mut tool_results = Vec::new();
            for block in &resp.content {
                if let ContentBlock::ToolUse { id, name, input } = block {
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
            return Ok(TurnOutcome::Continue);
        }

        anyhow::bail!("Unexpected stop_reason: {}", resp.stop_reason)
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

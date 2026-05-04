use anyhow::Context;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

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
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum ContentBlock {
    Text { text: String },
    ToolUse { id: String, name: String, input: Value },
}

pub struct AgentService {
    api_key: String,
    model: String,
    mcp_client: McpClient,
    http_client: reqwest::Client,
}

impl AgentService {
    pub fn new(api_key: String, mcp_client: McpClient) -> Self {
        let model =
            std::env::var("ANTHROPIC_MODEL").unwrap_or_else(|_| "claude-opus-4-7".to_string());
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

    pub async fn run(&self, system: &str, prompt: &str) -> anyhow::Result<String> {
        let tools = self.tools_for_anthropic();
        let mut messages: Vec<Message> = vec![Message {
            role: "user".to_string(),
            content: json!(prompt),
        }];

        for _ in 0..MAX_ITERATIONS {
            let body = json!({
                "model": self.model,
                "max_tokens": MAX_TOKENS,
                "system": system,
                "tools": tools,
                "messages": messages,
            });

            let response = self
                .http_client
                .post(ANTHROPIC_API_URL)
                .header("x-api-key", &self.api_key)
                .header("anthropic-version", ANTHROPIC_VERSION)
                .header("content-type", "application/json")
                .json(&body)
                .send()
                .await
                .context("Failed to call Anthropic API")?;

            if !response.status().is_success() {
                let status = response.status();
                let body = response.text().await.unwrap_or_default();
                anyhow::bail!("Anthropic API error {}: {}", status, body);
            }

            let resp: AnthropicResponse =
                response.json().await.context("Failed to parse Anthropic response")?;

            let assistant_content: Value = serde_json::to_value(&resp.content)
                .context("Failed to serialize assistant content")?;
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
                return Ok(text);
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
                messages.push(Message {
                    role: "user".to_string(),
                    content: json!(tool_results),
                });
                continue;
            }

            anyhow::bail!("Unexpected stop_reason: {}", resp.stop_reason);
        }

        anyhow::bail!("Agent exceeded max iterations ({})", MAX_ITERATIONS)
    }

    pub async fn close(self) -> anyhow::Result<()> {
        self.mcp_client.close().await
    }
}

//! 台本を LLM に書かせる（`GenerateScript` と、番組の定期実行の LLM モードで共用する）。
//!
//! 番組の持ち主の API キー（BYOK）・モデルで、CMS の MCP ツールのうち読み取りだけと worker のツール
//! （`fetch_url` など）を渡してエージェントを回し、structured outputs で台本の JSON を受け取る。
//! 検証・保存は呼び出し側が行う

use super::agent_service::{AgentService, LocalTool, Usage, READ_ONLY_CMS_TOOLS};
use super::cms_client::CmsClient;
use super::llm_api_key::resolve_api_key;
use super::llm_models::resolve_model;
use super::mcp_client::McpClient;
use super::script_output::script_output_schema;
use anyhow::Context;
use async_trait::async_trait;
use std::sync::Arc;

/// エージェントの実行結果
pub(crate) struct AgentRun {
    /// 最終応答の本文（台本の JSON）
    pub result: anyhow::Result<String>,
    /// LLM を呼んだときのトークン数の合計（失敗しても入る。呼ぶ前に失敗したら `None`）
    pub usage: Option<Usage>,
}

/// 台本を書くエージェント（本番は [`LlmScriptAgent`]、テストはフェイク）
#[async_trait]
pub(crate) trait ScriptAgent: Send + Sync {
    /// `owner` は番組の `owner_id`（API キー・モデルを決める）
    async fn generate(
        &self,
        owner: Option<&str>,
        system: &str,
        prompt: &str,
        local_tools: Vec<Arc<dyn LocalTool>>,
    ) -> AgentRun;
}

pub(crate) struct LlmScriptAgent {
    cms: Arc<CmsClient>,
}

impl LlmScriptAgent {
    pub(crate) fn new(cms: Arc<CmsClient>) -> Self {
        Self { cms }
    }

    async fn prepare(
        &self,
        owner: Option<&str>,
        local_tools: Vec<Arc<dyn LocalTool>>,
    ) -> anyhow::Result<AgentService> {
        // API キーは番組の持ち主が CMS に保存したもの（BYOK）。このジョブの間だけ持つ
        let api_key = resolve_api_key(self.cms.as_ref(), owner).await?;
        // モデルは番組の持ち主の設定 → llm_models の既定の順で決める
        let model = resolve_model(self.cms.as_ref(), owner).await?;
        let mcp_cmd = std::env::var("MCP_SERVER_CMD").unwrap_or_else(|_| "node".to_string());
        let mcp_args_str =
            std::env::var("MCP_SERVER_ARGS").context("MCP_SERVER_ARGS is not set")?;
        let mcp_args: Vec<&str> = mcp_args_str.split_whitespace().collect();
        let mcp_client = McpClient::new(&mcp_cmd, &mcp_args)
            .await
            .context("Failed to create MCP client")?;
        Ok(AgentService::new(api_key, model, mcp_client, READ_ONLY_CMS_TOOLS)
            .with_local_tools(local_tools))
    }
}

#[async_trait]
impl ScriptAgent for LlmScriptAgent {
    async fn generate(
        &self,
        owner: Option<&str>,
        system: &str,
        prompt: &str,
        local_tools: Vec<Arc<dyn LocalTool>>,
    ) -> AgentRun {
        let agent = match self.prepare(owner, local_tools).await {
            Ok(agent) => agent,
            Err(e) => {
                return AgentRun {
                    result: Err(e),
                    usage: None,
                }
            }
        };
        let output_schema = script_output_schema();
        let result = agent.run(system, prompt, Some(&output_schema)).await;
        let usage = Some(agent.usage());
        // エージェントが失敗しても MCP サーバーの子プロセスは止める
        if let Err(e) = agent.close().await {
            tracing::warn!("Failed to close MCP client: {e:#}");
        }
        AgentRun {
            result: result.context("Agent failed"),
            usage,
        }
    }
}

use anyhow::Context;
use rmcp::{
    ServiceExt,
    model::{CallToolRequestParams, Tool},
    service::RunningService,
    transport::{ConfigureCommandExt, TokioChildProcess},
};
use serde_json::Value;
use tokio::process::Command;

pub struct McpClient {
    service: RunningService<rmcp::RoleClient, ()>,
    tools: Vec<Tool>,
}

impl McpClient {
    pub async fn new(mcp_cmd: &str, mcp_args: &[&str]) -> anyhow::Result<Self> {
        let transport = TokioChildProcess::new(Command::new(mcp_cmd).configure(|cmd| {
            cmd.args(mcp_args);
        }))
        .context("Failed to spawn MCP server process")?;

        let service = ().serve(transport).await.context("Failed to connect to MCP server")?;
        let tools = service.list_all_tools().await.context("Failed to list MCP tools")?;

        Ok(Self { service, tools })
    }

    pub fn tools(&self) -> &[Tool] {
        &self.tools
    }

    pub async fn call_tool(
        &self,
        name: impl Into<std::borrow::Cow<'static, str>>,
        arguments: Value,
    ) -> anyhow::Result<String> {
        let arguments = arguments
            .as_object()
            .cloned()
            .unwrap_or_default();

        let result = self
            .service
            .call_tool(CallToolRequestParams::new(name).with_arguments(arguments))
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

        Ok(text)
    }

    pub async fn close(self) -> anyhow::Result<()> {
        self.service.cancel().await.context("Failed to close MCP client")?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mcp_server_path() -> String {
        std::env::var("MCP_SERVER_ARGS")
            .unwrap_or_else(|_| "/Users/kmt/dev/botcast-cms/mcp/build/index.js".to_string())
    }

    #[tokio::test]
    #[ignore = "requires botcast-cms MCP server to be built"]
    async fn mcp_client_lists_tools() {
        let path = mcp_server_path();
        let args: Vec<&str> = path.split_whitespace().collect();
        let client = McpClient::new("node", &args).await.expect("failed to create client");
        let tools = client.tools();
        assert!(!tools.is_empty(), "expected at least one tool");
        let tool_names: Vec<&str> = tools.iter().map(|t| t.name.as_ref()).collect();
        assert!(
            tool_names.contains(&"RecordApi_list"),
            "expected RecordApi_list tool"
        );
        client.close().await.unwrap();
    }
}

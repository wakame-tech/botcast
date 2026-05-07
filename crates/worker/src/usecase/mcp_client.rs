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
        std::env::var("MCP_SERVER_ARGS").expect("MCP_SERVER_ARGS is not set")
    }

    async fn make_client() -> McpClient {
        let path = mcp_server_path();
        let args: Vec<&str> = path.split_whitespace().collect();
        McpClient::new("node", &args).await.expect("failed to create client")
    }

    async fn resolve_collection_id(client: &McpClient, name: &str) -> String {
        let raw = client
            .call_tool("CollectionApi_list", serde_json::json!({}))
            .await
            .expect("failed to list collections");
        let start = raw.find('[').or_else(|| raw.find('{')).expect("no JSON in response");
        let json: serde_json::Value = serde_json::from_str(&raw[start..]).expect("invalid JSON");
        json.as_array()
            .unwrap()
            .iter()
            .find(|c| c["name"] == name)
            .and_then(|c| c["id"].as_str())
            .and_then(|id| id.strip_prefix("collection:"))
            .unwrap_or_else(|| panic!("collection '{}' not found", name))
            .to_string()
    }

    fn parse_created_id(result: &str) -> String {
        let start = result.find('{').expect("no JSON in result");
        let json: serde_json::Value = serde_json::from_str(&result[start..]).expect("invalid JSON");
        json["id"].as_str().expect("id field missing").to_string()
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

    #[tokio::test]
    #[ignore = "requires botcast-cms MCP server to be built"]
    async fn mcp_client_creates_podcast() {
        let client = make_client().await;
        let col_id = resolve_collection_id(&client, "podcasts").await;

        let result = client
            .call_tool(
                "RecordApi_create",
                serde_json::json!({
                    "collectionId": col_id,
                    "requestBody": { "data": { "title": "test podcast", "icon": "🎙️", "user_id": "test-user" } }
                }),
            )
            .await
            .expect("failed to call tool");
        eprintln!("result: {}", result);
        assert!(result.contains("201"), "expected 201: {}", result);
        client.close().await.unwrap();
    }


    #[tokio::test]
    #[ignore = "requires botcast-cms MCP server to be built"]
    async fn mcp_client_creates_episode() {
        let client = make_client().await;

        // まず podcast を作成して podcast_id を取得
        let podcast_col_id = resolve_collection_id(&client, "podcasts").await;
        let podcast_result = client
            .call_tool(
                "RecordApi_create",
                serde_json::json!({
                    "collectionId": podcast_col_id,
                    "requestBody": { "data": { "title": "podcast for episode test", "icon": "🎙️", "user_id": "test-user" } }
                }),
            )
            .await
            .expect("failed to create podcast");
        assert!(podcast_result.contains("201"), "podcast creation failed: {}", podcast_result);
        let podcast_record_id = parse_created_id(&podcast_result);
        eprintln!("created podcast id: {}", podcast_record_id);

        // episode を作成
        let ep_col_id = resolve_collection_id(&client, "episodes").await;
        let result = client
            .call_tool(
                "RecordApi_create",
                serde_json::json!({
                    "collectionId": ep_col_id,
                    "requestBody": {
                        "data": {
                            "title": "test episode",
                            "podcast_id": podcast_record_id,
                            "user_id": "test-user"
                        }
                    }
                }),
            )
            .await
            .expect("failed to call tool");
        eprintln!("result: {}", result);
        assert!(result.contains("201"), "expected 201: {}", result);
        client.close().await.unwrap();
    }

    #[tokio::test]
    #[ignore = "requires botcast-cms MCP server to be built"]
    async fn mcp_client_calls_record_list() {
        let path = mcp_server_path();
        let args: Vec<&str> = path.split_whitespace().collect();
        let client = McpClient::new("node", &args).await.expect("failed to create client");
        let tool_names: Vec<&str> = client.tools().iter().map(|t| t.name.as_ref()).collect();
        eprintln!("available tools: {:?}", tool_names);
        let result = client
            .call_tool("RecordApi_list", serde_json::json!({ "collectionId": "episodes" }))
            .await
            .expect("failed to call tool");
        assert!(!result.is_empty(), "expected non-empty result");
        eprintln!("result: {}", result);
        assert!(result.contains("Status:"), "expected API response in: {}", result);
        client.close().await.unwrap();
    }
}

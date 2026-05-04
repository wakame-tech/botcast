---
id: "01KNM2BBT5E7JFE2PGCRB9VRG6"
name: "MCPサーバーが全APIをLLMエージェント向けツールとして公開する"
status: "stable"
---

## 関連ファイル

- `cms/crates/api/src/openapi_adapter.rs`
- `mcp/src/index.ts` (Generated)
- `mcp/package.json`
- `spec/tsp-output/schema/openapi.yaml` (Generated)

## 機能概要

OpenAPI 仕様から自動生成された TypeScript MCP サーバーが、StdioServerTransport を通じて 11 個のツール（Collection CRUD、Record CRUD + Generate、Script Execute、Job List + Create）を LLM エージェントに公開する。Zod でランタイムバリデーションを行い、Axios で CMS API にリクエストを転送する。

## 意図

CMS の全機能を MCP プロトコル経由で LLM エージェントが直接利用できるようにし、MCP-first アーキテクチャを実現する。

## 主要なメンバー

- `McpToolDefinition { name, description, inputSchema, method, pathTemplate }` — ツール定義
- `StdioServerTransport` — LLM エージェントとの標準入出力通信
- `ListToolsRequestSchema` ハンドラ — 利用可能なツール一覧を返す
- `CallToolRequestSchema` ハンドラ — ツールを実行する

## シナリオ

### LLMエージェントがツール一覧を取得する

1. エージェントが ListTools リクエストを送信する
2. MCP サーバーが 11 個のツール定義を返す
3. 各ツールは name, description, inputSchema を含む

### LLMエージェントがツールを実行する

1. エージェントが CallTool リクエストでツール名と引数を送信する
2. MCP サーバーが Zod で引数を検証する
3. HTTP リクエストを構築し CMS API に転送する
4. API レスポンスをテキストとしてエージェントに返す

### ツール引数のバリデーションに失敗する

1. エージェントが不正な引数でツールを実行する
2. Zod バリデーションが失敗する
3. エラーメッセージをエージェントに返す

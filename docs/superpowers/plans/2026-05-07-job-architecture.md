# Job 実行アーキテクチャ改善 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** job 実行ロジックを botcast に集約し、botcast に rmcp MCP サーバーを追加して `generate_audio` / `generate_script` / `list_jobs` / `get_job_status` ツールを公開する。botcast-cms から Job API と BotcastWorker を削除してデータストア+MCP サーバーに純化する。

**Architecture:** botcast が kafru worker + MCP サーバー（Streamable HTTP）として動作し、job 投入・状態管理を一手に引き受ける。botcast-cms は Collection/Record/Script API のみに専念し、BotcastWorker・kafru 依存を完全に除去する。LLM Agent は botcast MCP と CMS MCP の両方を接続して使う。

**Tech Stack:** Rust / axum / rmcp 1.6.0 (features: server, transport-streamable-http-server) / kafru 1.0.4 / TypeSpec / Node.js MCP SDK

**GitHub Issue:** https://github.com/wakame-tech/botcast/issues/114

---

## ファイルマップ

### botcast-cms（削除・修正）

| 操作 | パス |
|---|---|
| 修正 | `botcast-cms/spec/main.tsp` |
| 削除 | `botcast-cms/cms/crates/worker/` （crate 丸ごと） |
| 修正 | `botcast-cms/cms/Cargo.toml` |
| 修正 | `botcast-cms/cms/crates/api/Cargo.toml` |
| 修正 | `botcast-cms/cms/crates/api/src/main.rs` |
| 修正 | `botcast-cms/cms/crates/api/src/openapi_adapter.rs` |
| 修正 | `botcast-cms/cms/crates/openapi-gen/src/apis/mod.rs` |
| 削除 | `botcast-cms/cms/crates/openapi-gen/src/apis/job.rs` |
| 修正 | `botcast-cms/cms/crates/openapi-gen/src/models.rs` |
| 修正 | `botcast-cms/cms/crates/openapi-gen/src/server/mod.rs` |
| 修正 | `botcast-cms/mcp/src/index.ts` |

### botcast（追加・修正）

| 操作 | パス |
|---|---|
| 修正 | `crates/worker/Cargo.toml` |
| 新規 | `crates/worker/src/mcp_server/mod.rs` |
| 修正 | `crates/worker/src/usecase/task_service.rs` |
| 修正 | `crates/worker/src/api/router.rs` |
| 修正 | `crates/worker/src/lib.rs` |

---

## Task 1: botcast-cms — TypeSpec から Job を削除

**Files:**
- Modify: `botcast-cms/spec/main.tsp`

- [ ] **Step 1: Job モデルと JobApi インターフェースを削除する**

`botcast-cms/spec/main.tsp` から以下のブロックを丸ごと削除する（Canvas セクションの直前まで）：

```diff
-model Job {
-  name: string;
-  params: Record<{}>;
-
-  @visibility(Lifecycle.Read)
-  status: string;
-}
-
-alias CreateOrUpdateJob = CreateOrUpdate<Job>;
-
-@route("/jobs")
-@tag("Job")
-interface JobApi {
-  @post create(@body body: CreateOrUpdateJob): {
-    @statusCode statusCode: 204;
-  };
-
-  @get list(): {
-    @statusCode statusCode: 200;
-    @body body: Job[];
-  };
-}
-
 // Canvas: Graph edges between records
```

- [ ] **Step 2: TypeSpec → OpenAPI YAML を再生成する**

```bash
cd /Users/kmt/dev/botcast-cms/spec && npm run build
```

Expected: エラーなく完了し `tsp-output/` に openapi.yaml が更新される。

- [ ] **Step 3: コンパイルが通ることを確認する**

```bash
cd /Users/kmt/dev/botcast-cms/cms && cargo check 2>&1 | grep -E "^error" | head -20
```

Expected: まだ `job.rs` が残っているのでエラーが出る（次 Task で修正）。現時点では spec 側の変更のみコミット。

- [ ] **Step 4: コミット**

```bash
cd /Users/kmt/dev/botcast-cms
git add spec/main.tsp spec/tsp-output/
git commit -m "feat: TypeSpecからJob定義を削除"
```

---

## Task 2: botcast-cms — openapi-gen から Job を削除

**Files:**
- Delete: `botcast-cms/cms/crates/openapi-gen/src/apis/job.rs`
- Modify: `botcast-cms/cms/crates/openapi-gen/src/apis/mod.rs`
- Modify: `botcast-cms/cms/crates/openapi-gen/src/models.rs`
- Modify: `botcast-cms/cms/crates/openapi-gen/src/server/mod.rs`

- [ ] **Step 1: `apis/job.rs` を削除する**

```bash
rm /Users/kmt/dev/botcast-cms/cms/crates/openapi-gen/src/apis/job.rs
```

- [ ] **Step 2: `apis/mod.rs` から job モジュールを削除する**

`botcast-cms/cms/crates/openapi-gen/src/apis/mod.rs` を編集：

```diff
 pub mod canvas;
 pub mod collections;
-pub mod job;
 pub mod record;
 pub mod script;
```

- [ ] **Step 3: `models.rs` から Job 関連の struct を削除する**

`botcast-cms/cms/crates/openapi-gen/src/models.rs` の以下の struct を削除する：
- `pub struct CreateOrUpdateJob { ... }` （`impl CreateOrUpdateJob`, `impl std::fmt::Display for CreateOrUpdateJob`, `impl std::str::FromStr for CreateOrUpdateJob` も含む全ブロック）
- `pub struct Job { ... }` （impl ブロックも含む）

検索して行番号を特定してから削除する：

```bash
grep -n "CreateOrUpdateJob\|pub struct Job " /Users/kmt/dev/botcast-cms/cms/crates/openapi-gen/src/models.rs | head -10
```

- [ ] **Step 4: `server/mod.rs` から Job ルーターを削除する**

```bash
grep -n "job\|Job" /Users/kmt/dev/botcast-cms/cms/crates/openapi-gen/src/server/mod.rs | head -20
```

出力を確認し、Job ルーター登録部分を削除する（`JobApi` を使っている箇所）。

- [ ] **Step 5: コンパイル確認**

```bash
cd /Users/kmt/dev/botcast-cms/cms && cargo check -p openapi 2>&1 | grep "^error" | head -20
```

Expected: openapi crate でエラーがなくなる。

- [ ] **Step 6: コミット**

```bash
cd /Users/kmt/dev/botcast-cms
git add cms/crates/openapi-gen/
git commit -m "feat: openapi-genからJob定義を削除"
```

---

## Task 3: botcast-cms — BotcastWorker を削除し api crate を更新

**Files:**
- Delete: `botcast-cms/cms/crates/worker/` (crate 全体)
- Modify: `botcast-cms/cms/Cargo.toml`
- Modify: `botcast-cms/cms/crates/api/Cargo.toml`
- Modify: `botcast-cms/cms/crates/api/src/main.rs`
- Modify: `botcast-cms/cms/crates/api/src/openapi_adapter.rs`

- [ ] **Step 1: `worker` crate を削除する**

```bash
rm -rf /Users/kmt/dev/botcast-cms/cms/crates/worker
```

- [ ] **Step 2: ワークスペース `Cargo.toml` から worker crate を除外する（自動除外を確認）**

```bash
cat /Users/kmt/dev/botcast-cms/cms/Cargo.toml
```

`members = ["crates/*"]` のグロブ指定の場合はディレクトリ削除で自動的に除外される。明示的に `"crates/worker"` が書かれている場合は削除する。

- [ ] **Step 3: `api/Cargo.toml` から worker 依存を削除する**

`botcast-cms/cms/crates/api/Cargo.toml` を編集して `worker = { path = "../worker" }` の行を削除する。また `kafru` 直接依存があれば削除する。

- [ ] **Step 4: `main.rs` から BotcastWorker を削除する**

`botcast-cms/cms/crates/api/src/main.rs` を以下のように編集：

```diff
-use worker::BotcastWorker;

 #[derive(Clone)]
 pub struct AppState {
     pub dify_sandbox_client: Arc<DifySandboxClient>,
-    pub worker: Arc<BotcastWorker>,
     pub collection_repo: Arc<dyn CollectionRepository>,
     pub db: Surreal<Client>,
     pub jwt_secret: String,
     pub api_key: Option<String>,
     pub namespace: String,
     pub database: String,
 }
```

`main()` 関数内から BotcastWorker の初期化・起動コードを削除：

```diff
-    let worker = BotcastWorker::new()
-        .await
-        .expect("Failed to create BotcastWorker");
-    let worker = Arc::new(worker);
-    worker.launch().await;
-
     let collection_repo = SurrealCollectionRepository::try_from_env()
```

`AppState` の初期化から `worker` フィールドを削除：

```diff
     let state = AppState {
         dify_sandbox_client: Arc::new(
             DifySandboxClient::try_new().expect("Failed to create DifySandboxClient"),
         ),
-        worker: worker.clone(),
         collection_repo: Arc::new(collection_repo),
```

- [ ] **Step 5: `openapi_adapter.rs` から Job 実装と worker フィールドを削除する**

`botcast-cms/cms/crates/api/src/openapi_adapter.rs` から以下を削除：

```diff
-use worker::{BotcastJob, BotcastWorker};
```

`ApiImpl` struct から `worker` フィールドを削除：

```diff
 pub struct ApiImpl {
     pub dify_sandbox_client: Arc<DifySandboxClient>,
-    pub worker: Arc<BotcastWorker>,
     ...
```

`ApiImpl::new()` から `worker: state.worker,` を削除する。

`impl Job<AppError> for ApiImpl { ... }` ブロック全体を削除する（`fn job_api_create`, `fn job_api_list` と、ヘルパー関数 `try_into_map`, `try_into_openapi_job` も削除）。

`use` 宣言から Job トレイト関連のインポートを削除：

```diff
-        job::{Job, JobApiCreateResponse, JobApiListResponse},
```

- [ ] **Step 6: コンパイル確認**

```bash
cd /Users/kmt/dev/botcast-cms/cms && cargo check 2>&1 | grep "^error" | head -20
```

Expected: エラーなし。

- [ ] **Step 7: コミット**

```bash
cd /Users/kmt/dev/botcast-cms
git add cms/
git commit -m "feat: BotcastWorkerとJob APIをbotcast-cmsから削除"
```

---

## Task 4: botcast-cms — MCP サーバーから Job ツールを削除

**Files:**
- Modify: `botcast-cms/mcp/src/index.ts`

- [ ] **Step 1: `JobApi_list` と `JobApi_create` のエントリを削除する**

`botcast-cms/mcp/src/index.ts` の `toolDefinitionMap` から以下の2エントリを削除する：

```bash
grep -n "JobApi_list\|JobApi_create" /Users/kmt/dev/botcast-cms/mcp/src/index.ts
```

出力された行番号のブロックを削除する。（`["JobApi_list", { ... }],` と `["JobApi_create", { ... }],` の2ブロック）

- [ ] **Step 2: ビルド確認**

```bash
cd /Users/kmt/dev/botcast-cms/mcp && npm run build 2>&1 | tail -5
```

Expected: エラーなし。

- [ ] **Step 3: コミット**

```bash
cd /Users/kmt/dev/botcast-cms
git add mcp/
git commit -m "feat: MCP serverからJobApi toolsを削除"
```

---

## Task 5: botcast — TaskService に list_jobs / get_job_status を追加

**Files:**
- Modify: `crates/worker/src/usecase/task_service.rs`

- [ ] **Step 1: 既存テストが通ることを確認する**

```bash
cd /Users/kmt/dev/botcast && cargo test -p worker -- task_service 2>&1 | tail -10
```

- [ ] **Step 2: `Job` レスポンス型と list_jobs / get_job_status メソッドを追加する**

`crates/worker/src/usecase/task_service.rs` の `use` 宣言に追加：

```diff
+use kafru::queue::{QueueListConditions, QueueStatus};
```

ファイル末尾（`#[cfg(test)]` の直前）に追加：

```rust
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub(crate) struct JobInfo {
    pub id: String,
    pub name: String,
    pub status: String,
}

impl TryFrom<kafru::queue::QueueData> for JobInfo {
    type Error = Error;

    fn try_from(q: kafru::queue::QueueData) -> Result<Self, Self::Error> {
        Ok(JobInfo {
            id: q.id
                .map(|r| r.to_string())
                .ok_or_else(|| Error::Other(anyhow::anyhow!("missing job id")))?,
            name: q.name.unwrap_or_default(),
            status: q.status.map(|s| s.to_string()).unwrap_or_default(),
        })
    }
}
```

`TaskService` の `impl` ブロック内に追加：

```rust
pub(crate) async fn list_jobs(&self) -> anyhow::Result<Vec<JobInfo>, Error> {
    self.kafru_queue
        .list(QueueListConditions {
            status: Some(vec![
                QueueStatus::Waiting.to_string(),
                QueueStatus::InProgress.to_string(),
                QueueStatus::Error.to_string(),
                QueueStatus::Completed.to_string(),
            ]),
            queue: Some(vec!["botcast-worker-default".to_string()]),
            limit: Some(100),
        })
        .await
        .map_err(|e| Error::Other(anyhow::anyhow!(e)))?
        .into_iter()
        .map(JobInfo::try_from)
        .collect()
}

pub(crate) async fn get_job_status(&self, job_id: &str) -> anyhow::Result<JobInfo, Error> {
    let id = kafru::queue::Queue::parse_record_id(job_id)
        .map_err(|e| Error::Other(anyhow::anyhow!("invalid job_id: {}", e)))?;
    let data = self
        .kafru_queue
        .get(id)
        .await
        .map_err(|e| Error::Other(anyhow::anyhow!("job not found: {}", e)))?;
    JobInfo::try_from(data)
}
```

- [ ] **Step 3: `list_jobs` のユニットテストを追加する（コンパイル確認用）**

`#[cfg(test)]` ブロック内に追加：

```rust
#[test]
fn job_info_try_from_missing_id_returns_error() {
    let data = kafru::queue::QueueData {
        id: None,
        name: Some("test".to_string()),
        ..Default::default()
    };
    let result = JobInfo::try_from(data);
    assert!(result.is_err());
}
```

- [ ] **Step 4: コンパイル確認**

```bash
cd /Users/kmt/dev/botcast && cargo check -p worker 2>&1 | grep "^error" | head -20
```

> Note: `Queue::parse_record_id` が存在しない場合は `kafru::queue::Queue::get` の引数型を確認し、`surrealdb::RecordId::from_table_key` 等の適切な構築方法を使う。

```bash
grep -r "pub fn get\|RecordId\|parse" /Users/kmt/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/kafru-1.0.4/src/queue.rs | head -10
```

- [ ] **Step 5: コミット**

```bash
cd /Users/kmt/dev/botcast
git add crates/worker/src/usecase/task_service.rs
git commit -m "feat: TaskServiceにlist_jobs/get_job_statusを追加"
```

---

## Task 6: botcast — MCP サーバーを実装する

**Files:**
- Modify: `crates/worker/Cargo.toml`
- Create: `crates/worker/src/mcp_server/mod.rs`
- Modify: `crates/worker/src/lib.rs`

- [ ] **Step 1: rmcp の features を更新する**

`crates/worker/Cargo.toml` を編集：

```diff
-rmcp = { version = "1.6.0", features = ["client", "transport-child-process"] }
+rmcp = { version = "1.6.0", features = ["client", "server", "transport-child-process", "transport-streamable-http-server"] }
+schemars = "0.8"
```

- [ ] **Step 2: `mcp_server/mod.rs` を作成する**

`crates/worker/src/mcp_server/mod.rs` を新規作成：

```rust
use crate::usecase::provider::Provider;
use rmcp::{
    ServerHandler, ServiceExt,
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::ServerInfo,
    tool, tool_handler, tool_router,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct GenerateAudioParams {
    pub episode_id: Uuid,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct GenerateScriptParams {
    pub episode_id: Uuid,
    pub prompt: String,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct GetJobStatusParams {
    pub job_id: String,
}

#[derive(Clone)]
pub struct BotcastMcpServer {
    provider: Arc<Provider>,
    tool_router: ToolRouter<Self>,
}

#[tool_handler(router = self.tool_router)]
impl ServerHandler for BotcastMcpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            name: "botcast".into(),
            version: env!("CARGO_PKG_VERSION").into(),
            ..Default::default()
        }
    }
}

#[tool_router(router = tool_router)]
impl BotcastMcpServer {
    pub fn new(provider: Arc<Provider>) -> Self {
        Self {
            provider,
            tool_router: Self::tool_router(),
        }
    }

    #[tool(description = "エピソードの音声を生成するジョブをエンキューします")]
    pub async fn generate_audio(
        &self,
        params: Parameters<GenerateAudioParams>,
    ) -> String {
        match self
            .provider
            .task_service()
            .create_task(crate::usecase::task_service::Args::GenerateAudio {
                episode_id: params.episode_id,
            })
            .await
        {
            Ok(()) => serde_json::json!({ "status": "enqueued" }).to_string(),
            Err(e) => serde_json::json!({ "error": e.to_string() }).to_string(),
        }
    }

    #[tool(description = "エピソードの台本を生成するジョブをエンキューします")]
    pub async fn generate_script(
        &self,
        params: Parameters<GenerateScriptParams>,
    ) -> String {
        match self
            .provider
            .task_service()
            .create_task(crate::usecase::task_service::Args::GenerateScript {
                episode_id: params.episode_id,
                prompt: params.prompt.clone(),
            })
            .await
        {
            Ok(()) => serde_json::json!({ "status": "enqueued" }).to_string(),
            Err(e) => serde_json::json!({ "error": e.to_string() }).to_string(),
        }
    }

    #[tool(description = "ジョブ一覧を取得します")]
    pub async fn list_jobs(&self) -> String {
        match self.provider.task_service().list_jobs().await {
            Ok(jobs) => serde_json::to_string(&jobs).unwrap_or_else(|e| e.to_string()),
            Err(e) => serde_json::json!({ "error": e.to_string() }).to_string(),
        }
    }

    #[tool(description = "ジョブの状態を取得します")]
    pub async fn get_job_status(&self, params: Parameters<GetJobStatusParams>) -> String {
        match self
            .provider
            .task_service()
            .get_job_status(&params.job_id)
            .await
        {
            Ok(info) => serde_json::to_string(&info).unwrap_or_else(|e| e.to_string()),
            Err(e) => serde_json::json!({ "error": e.to_string() }).to_string(),
        }
    }
}
```

- [ ] **Step 3: `lib.rs` に mcp_server モジュールを追加する**

`crates/worker/src/lib.rs` の先頭に追加：

```diff
+pub(crate) mod mcp_server;
```

- [ ] **Step 4: コンパイル確認**

```bash
cd /Users/kmt/dev/botcast && cargo check -p worker 2>&1 | grep "^error" | head -30
```

Expected: エラーなし。

- [ ] **Step 5: コミット**

```bash
cd /Users/kmt/dev/botcast
git add crates/worker/Cargo.toml crates/worker/src/mcp_server/ crates/worker/src/lib.rs
git commit -m "feat: botcast MCP serverを追加(generate_audio/generate_script/list_jobs/get_job_status)"
```

---

## Task 7: botcast — HTTP ルーターに MCP エンドポイントと /jobs を追加・/createTask を廃止

**Files:**
- Modify: `crates/worker/src/api/router.rs`

- [ ] **Step 1: 現在の router.rs を確認する**

```bash
cat /Users/kmt/dev/botcast/crates/worker/src/api/router.rs
```

- [ ] **Step 2: MCP Streamable HTTP ハンドラと `/jobs` エンドポイントを追加し `/createTask` を削除する**

`crates/worker/src/api/router.rs` を以下のように更新する：

```rust
use super::AppState;
use crate::{
    error::Error,
    mcp_server::BotcastMcpServer,
};
use axum::{
    extract::State,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use rmcp::transport::streamable_http_server::tower::StreamableHttpService;
use serde_json::json;
use std::sync::Arc;
use tracing::instrument;

#[instrument(skip(state))]
async fn list_jobs(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, Error> {
    let jobs = state.0.task_service().list_jobs().await?;
    Ok(Json(jobs))
}

async fn version() -> Result<impl IntoResponse, Error> {
    let worker_version = env!("CARGO_PKG_VERSION");
    Ok(Json(json!({
        "worker": worker_version,
    })))
}

pub(crate) fn routers(provider: Arc<crate::usecase::provider::Provider>) -> Router<Arc<AppState>> {
    let mcp_server = BotcastMcpServer::new(provider);
    let mcp_service = StreamableHttpService::new(
        move || Ok(mcp_server.clone()),
        Default::default(),
        Default::default(),
    );

    Router::new()
        .route("/version", get(version))
        .route("/jobs", get(list_jobs))
        .nest_service("/mcp", mcp_service)
}
```

- [ ] **Step 3: `api/mod.rs`（または `start_api`）で routers() 呼び出しを更新する**

`crates/worker/src/api/mod.rs` の `start_api` で `routers()` に `provider` を渡すよう変更：

```diff
 pub async fn start_api(provider: Arc<Provider>) -> anyhow::Result<()> {
     let state = Arc::new(AppState(provider.clone()));
     let router = routers(provider)
```

- [ ] **Step 4: コンパイル確認**

```bash
cd /Users/kmt/dev/botcast && cargo check -p worker 2>&1 | grep "^error" | head -30
```

> Note: `StreamableHttpService::new` の API は rmcp バージョンによって異なる可能性がある。コンパイルエラーが出た場合は以下で正確なシグネチャを確認する：
> ```bash
> grep -n "pub fn new\|pub struct StreamableHttpService" \
>   ~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/rmcp-1.6.0/src/transport/streamable_http_server/tower.rs | head -10
> ```

- [ ] **Step 5: コミット**

```bash
cd /Users/kmt/dev/botcast
git add crates/worker/src/api/
git commit -m "feat: HTTPルーターにMCP(/mcp)とjob一覧(/jobs)を追加・/createTaskを削除"
```

---

## Task 8: 動作確認

- [ ] **Step 1: botcast worker を起動して MCP エンドポイントが応答することを確認する**

```bash
cd /Users/kmt/dev/botcast && just worker &
sleep 3
curl -s -X POST http://localhost:9001/mcp \
  -H "Content-Type: application/json" \
  -H "Accept: application/json, text/event-stream" \
  -d '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"0.1"}}}' \
  | head -5
```

Expected: `{"jsonrpc":"2.0","result":{"protocolVersion":...,"serverInfo":{"name":"botcast",...}}}` が返る。

- [ ] **Step 2: `/jobs` エンドポイントが応答することを確認する**

```bash
curl -s http://localhost:9001/jobs
```

Expected: `[]` または job 一覧の JSON 配列が返る。

- [ ] **Step 3: botcast-cms が起動することを確認する（worker/kafru なし）**

```bash
cd /Users/kmt/dev/botcast-cms/cms && cargo check 2>&1 | grep "^error" | head -10
```

Expected: エラーなし。

- [ ] **Step 4: 最終コミット（必要なら）**

```bash
cd /Users/kmt/dev/botcast
git add -A
git status  # 未コミットの変更がないことを確認
```

---

## 実装上の注意点

### kafru `get_job_status` の RecordId 構築

kafru の `Queue::get(id: RecordId)` に渡す `RecordId` の構築方法：

```bash
grep -n "RecordId\|pub fn get" \
  ~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/kafru-1.0.4/src/queue.rs | head -15
```

`surrealdb::RecordId` は `("table", "id")` のタプルで構築できる。job_id の文字列フォーマット（例: `queue:ulid`）を確認してから適切に parse すること。

### rmcp `StreamableHttpService` のシグネチャ確認

```bash
cat ~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/rmcp-1.6.0/src/transport/streamable_http_server/tower.rs | head -80
```

`allowed_origins` や `allow_any_host()` の設定が必要な場合は `StreamableHttpServerConfig` を使う。

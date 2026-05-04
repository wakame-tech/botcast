//! Worker クレート — タスクキュー処理と API サーバー。
//!
//! # システム概要
//!
//! Botcast はポッドキャスト自動生成システム。
//!
//! | コンポーネント | 技術 | 役割 |
//! |---|---|---|
//! | frontend | React / TanStack Router / UnoCSS | UI |
//! | api | Rust / Axum / OpenAPI | 認証・ユーザー管理 |
//! | worker | Rust / kafru / VoiceVox | タスク実行 |
//! | botcast-cms | Node.js / Payload CMS | Podcast/Episode/Script データ管理 |
//! | storage | Cloudflare R2 | 音声・字幕ファイル |
//!
//! ## 機能
//!
//! - ユーザー (`User`) 作成・ログイン (Supabase Auth)
//! - ポッドキャスト (`Podcast`) / エピソード (`Episode`) の管理 (botcast-cms)
//! - LLM エージェント (Claude) が MCP 経由で台本を生成し CMS に保存
//! - 台本から VoiceVox TTS で音声を合成し R2 にアップロード
//!
//! クレート依存関係とタスクフローの詳細は [`usecase`] モジュールを参照。

pub mod api;
pub mod error;
pub mod jobs;
pub mod usecase;
pub mod worker;

use crate::jobs::{execute_task_job, test_job::TestJob};
use kafru::{
    database::Db,
    manager::Manager,
    queue::{Queue, QueueData, QueueListConditions, QueueStatus},
    task::TaskRegistry,
};
use serde_json::Value;
use std::{collections::HashMap, sync::Arc};
use tokio::task::JoinHandle;

/// ジョブのサマリ情報。API 層から参照する。
pub struct BotcastJob {
    pub name: String,
    pub params: HashMap<String, Value>,
    pub status: String,
}

impl TryFrom<QueueData> for BotcastJob {
    type Error = anyhow::Error;

    fn try_from(data: QueueData) -> Result<Self, Self::Error> {
        Ok(BotcastJob {
            name: data.name.ok_or_else(|| anyhow::anyhow!("Job name missing"))?,
            params: data
                .parameters
                .ok_or_else(|| anyhow::anyhow!("Job parameters missing"))?,
            status: data
                .status
                .map(|s| s.to_string())
                .ok_or_else(|| anyhow::anyhow!("Job status missing"))?,
        })
    }
}

/// kafru ワーカー管理クライアント。API 層からジョブの投入・一覧取得に使う。
pub struct BotcastWorker {
    server: String,
    task_registry: Arc<TaskRegistry>,
    queue: Arc<Queue<'static>>,
    db: Arc<Db>,
}

impl BotcastWorker {
    pub async fn new() -> anyhow::Result<Self> {
        let server = "botcast-worker".to_string();
        let db = Db::new(None)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to connect to database: {}", e))?;
        let db = Arc::new(db);

        let mut task_registry = TaskRegistry::new().await;
        task_registry
            .register("testjob".to_string(), || Box::new(TestJob))
            .await;
        task_registry
            .register(
                "execute_task".to_string(),
                execute_task_job::create,
            )
            .await;

        Ok(Self {
            server,
            task_registry: Arc::new(task_registry),
            queue: Arc::new(Queue::new(Some(db.clone())).await),
            db,
        })
    }

    pub async fn launch(&self) -> JoinHandle<()> {
        let mut manager = Manager::new(self.server.clone(), "admin".to_string()).await;
        let _ = manager
            .worker(
                "default".to_string(),
                5,
                self.task_registry.clone(),
                1,
                Some(self.db.clone()),
            )
            .await;
        let _ = manager
            .scheduler("default".to_string(), 1, Some(self.db.clone()))
            .await;

        tokio::spawn(async move {
            tracing::info!("BotcastWorker watching...");
            let _ = manager.wait().await;
        })
    }

    pub async fn list_jobs(&self) -> anyhow::Result<Vec<BotcastJob>> {
        let jobs = self
            .queue
            .list(QueueListConditions {
                status: Some(vec![
                    QueueStatus::Waiting.to_string(),
                    QueueStatus::InProgress.to_string(),
                    QueueStatus::Error.to_string(),
                    QueueStatus::Completed.to_string(),
                ]),
                queue: Some(vec![format!("{}-default", self.server)]),
                limit: Some(100),
            })
            .await
            .map_err(|e| anyhow::anyhow!("Failed to list jobs: {}", e))?
            .into_iter()
            .map(BotcastJob::try_from)
            .collect::<anyhow::Result<Vec<BotcastJob>>>()?;
        Ok(jobs)
    }

    pub async fn enqueue_job(
        &self,
        name: String,
        params: HashMap<String, Value>,
    ) -> anyhow::Result<()> {
        let queue_data = QueueData {
            queue: Some(format!("{}-default", self.server)),
            name: Some(name),
            handler: Some("testjob".to_string()),
            parameters: Some(params),
            ..Default::default()
        };
        self.queue
            .push(queue_data)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to enqueue job: {}", e))?;
        Ok(())
    }
}

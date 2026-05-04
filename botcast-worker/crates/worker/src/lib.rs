//! Worker クレート — タスクキュー処理と API サーバー。
//!
//! kafru をジョブキューとして使用し、`GenerateAudio` と `GenerateScript` の
//! 2 種類のタスクを非同期実行する。API サーバーは `/createTask` エンドポイントで
//! タスクをエンキューする。
//!
//! # クレート構成
//!
//! ```text
//! worker
//! ├── api          – Axum HTTP サーバー
//! ├── jobs         – kafru ジョブハンドラー
//! └── usecase
//!     ├── task_service    – タスクエンキュー・実行
//!     ├── episode_service – CMS からエピソードを取得して音声生成
//!     ├── agent_service   – Anthropic API エージェントループ
//!     └── mcp_client      – botcast-cms MCP サーバークライアント
//! ```

pub mod api;
pub mod error;
pub mod jobs;
pub mod usecase;
pub mod worker;

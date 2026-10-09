//! Worker クレート — ジョブキュー処理と MCP サーバー。
//!
//! # システム概要
//!
//! Botcast はポッドキャスト自動生成システム。
//!
//! | コンポーネント | 技術 | 役割 |
//! |---|---|---|
//! | web | React / TanStack Router / UnoCSS | UI。botcast-cms に JWT で直接アクセスする |
//! | worker | Rust / Axum / kafru / rmcp / VoiceVox | ジョブ実行。HTTP で MCP サーバー (`/mcp`) とジョブ一覧 (`/jobs`) を提供する |
//! | botcast-cms | Rust / Axum / SurrealDB | 認証 (JWT)・Podcast/Episode/Script データ・音声/字幕ファイル管理 |
//!
//! ## 機能
//!
//! - ユーザー登録・サインイン (botcast-cms の `/auth/signup`・`/auth/signin`)
//! - ポッドキャスト (`Podcast`) / エピソード (`Episode`) の管理 (botcast-cms)
//! - MCP ツール (`generate_audio` / `generate_script` / `list_jobs` / `get_job_status`) でジョブを投入・確認
//! - LLM エージェント (Claude) が MCP 経由で台本を生成し CMS に保存
//! - 台本から VoiceVox TTS で音声を合成し botcast-cms に保存
//! - 番組のスクリプトで前回エピソード宛てのお便りを読む新しいエピソードを定期生成 (`GenerateEpisode`・`ScheduleSync`)
//!
//! クレート依存関係とタスクフローの詳細は [`usecase`] モジュールを参照。

pub mod api;
pub mod error;
pub mod jobs;
pub(crate) mod mcp_server;
pub mod usecase;
pub mod worker;

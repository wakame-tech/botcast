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
pub(crate) mod mcp_server;
pub mod storage;
pub mod usecase;
pub mod worker;

//! LLM エージェントの worker の共通部分。
//!
//! botcast・socia の worker が使う。アプリ固有の処理（CMS のコレクション・ジョブの引数・ドメインの検証）は持たない。
//!
//! | モジュール | 役割 |
//! |---|---|
//! | [`telemetry`] | tracing と OpenTelemetry（OTLP）の初期化 |
//! | [`trace_context`] | kafru のジョブをまたいで `traceparent` を引き継ぐ |
//! | [`schedule`] | アプリの定期実行の設定を kafru のスケジュールへ同期する |
//! | [`agent`] | Anthropic のエージェントのループ（MCP のツール・client tool・structured outputs） |
//! | [`mcp_client`] | MCP サーバー（子プロセス）のクライアント |
//! | [`safe_fetch`] / [`fetch_url_tool`] | 許可したホストだけを取得する Web の取得（SSRF 対策）と client tool `fetch_url` |
//! | [`api_key`] | LLM の API キー（ログに出さない型） |

pub mod agent;
pub mod api_key;
pub mod fetch_url_tool;
pub mod mcp_client;
pub mod safe_fetch;
pub mod schedule;
pub mod telemetry;
#[cfg(any(test, feature = "testing"))]
pub mod testing;
pub mod trace_context;

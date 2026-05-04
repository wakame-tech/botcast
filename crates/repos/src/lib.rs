//! Repos クレート — オブジェクトストレージ (Cloudflare R2) アクセス層。
//!
//! [`storage::Storage`] トレイトを中心に、R2 へのアップロード・ダウンロードを抽象化する。

pub mod error;
pub mod provider;
pub mod r2_storage;
pub mod storage;

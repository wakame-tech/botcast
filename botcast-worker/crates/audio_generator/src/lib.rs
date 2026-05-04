//! Audio Generator クレート — VoiceVox TTS と ffmpeg を使った音声合成。
//!
//! [`generate_audio::generate_audio`] が [`openapi_client::models::Section`] の配列を受け取り、
//! wav を結合して mp3 と SRT ファイルを生成する。
//!
//! # 依存サービス
//! - **VoiceVox Engine** (`VOICEVOX_ENDPOINT`): セリフセクションの音声合成
//! - **ffmpeg**: wav 結合・mp3 エンコード

pub mod ffmpeg;
pub mod generate_audio;
pub mod voicevox;
pub mod workdir;

mod audio_downloader;

use async_trait::async_trait;
use openapi_client::models::Section;
use std::path::PathBuf;
use workdir::WorkDir;

#[async_trait]
pub trait AudioGenerator: Send + Sync {
    async fn generate(
        &self,
        i: &mut usize,
        workdir: &WorkDir,
        section: Section,
    ) -> anyhow::Result<Vec<(PathBuf, String)>>;
}

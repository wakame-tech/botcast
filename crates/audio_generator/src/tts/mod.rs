//! TTS エンジンの窓口。エンジンは「文 1 つ → 音声のバイト列」だけを受け持つ。
//!
//! 文の分割・形式の正規化・長さ・SRT・結合・キャッシュ・同時実行数は共通の層
//! （[`crate::generate_audio`]）で行う。

pub mod openai;
pub mod voicevox;

use crate::models::{RefAudio, ResolvedVoice};
use anyhow::Result;
use async_trait::async_trait;

/// 1 文の合成の依頼。
#[derive(Debug, Clone, Copy)]
pub struct SynthesisRequest<'a> {
    pub text: &'a str,
    pub voice: &'a ResolvedVoice,
    /// 名前空間（worker はテナント ID）。参照音声のサーバー側の声の ID を分けるのに使う
    pub namespace: &'a str,
}

#[async_trait]
pub trait TtsEngine: Send + Sync + std::fmt::Debug {
    /// 1 文を合成し、音声のバイト列を返す。形式（wav / mp3 / サンプルレート / チャンネル数）はエンジン任せで、
    /// 共通の層が ffmpeg でそろえる。
    async fn synthesize(&self, req: SynthesisRequest<'_>) -> Result<Vec<u8>>;

    /// 参照音声を名前空間 `namespace` の `voice_id` としてエンジンに同期する。
    /// 参照音声を持たないエンジンでは何もしない。
    async fn ensure_voice(
        &self,
        _namespace: &str,
        _voice_id: &str,
        _ref_audio: &RefAudio,
    ) -> Result<()> {
        Ok(())
    }

    /// エンジンに届くかを確かめる。
    async fn health(&self) -> Result<()>;

    /// エンジンに渡す前の本文の加工（VOICEVOX 互換は絵文字を除く）。空になった文は合成しない。
    fn prepare_text(&self, text: &str) -> String {
        text.to_string()
    }

    /// キャッシュのキーに足すエンジンの版（VOICEVOX 互換は `GET /version`、OpenAI 互換は model）。
    /// 取れなければ空文字。
    async fn version(&self) -> String {
        String::new()
    }
}

/// エラーの応答の本文を、長すぎない文字列にする。
pub(crate) async fn error_body(res: reqwest::Response) -> String {
    let text = res.text().await.unwrap_or_default();
    let mut text: String = text.chars().take(500).collect();
    if text.is_empty() {
        text.push_str("(empty body)");
    }
    text
}

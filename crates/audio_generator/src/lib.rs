//! Audio Generator クレート — TTS エンジンと ffmpeg を使った音声合成。
//!
//! [`generate_audio::generate_audio`] が声の解決済みのセクション（[`models::ResolvedSection`]）の配列を受け取り、
//! 各文を合成して mp3 と SRT を作る。CMS は知らない（声の解決は呼び出し側で行う）。
//!
//! # 構成
//! - [`tts::TtsEngine`]: 「文 1 つ → 音声のバイト列」の窓口。VOICEVOX 互換（[`tts::voicevox`]）と
//!   OpenAI 互換（[`tts::openai`]。Irodori-TTS-Server など）の 2 種類
//! - [`config`] / [`registry::TtsRegistry`]: 環境変数（`TTS_ENGINES` / `TTS_ENGINE_<NAME>_*`、互換の
//!   `VOICEVOX_ENDPOINT`）から作るエンジンの一覧
//! - [`generate_audio`] はエンジン非依存の共通の層。文の分割（[`text`]）・キャッシュ・エンジンごとの同時実行数・
//!   正規化（44.1kHz・s16・2ch の wav）・長さと SRT・結合
//! - [`models::resolve_legacy_sections`]: 旧形式の `speaker`（VOICEVOX のスタイル ID）を既定のエンジンの声にする互換の層
//!
//! # 依存サービス
//! - **TTS エンジン**: セリフセクションの音声合成
//! - **ffmpeg**: 正規化・切り出し・結合・mp3 エンコード

pub mod config;
pub mod ffmpeg;
pub mod generate_audio;
pub mod models;
pub mod registry;
pub mod text;
pub mod tts;
pub mod workdir;

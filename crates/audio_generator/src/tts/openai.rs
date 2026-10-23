//! OpenAI 互換の窓口（`POST {endpoint}/audio/speech`）。Irodori-TTS-Server・OpenAI など。
//!
//! - 本文: `model`・`input`・`voice`・`response_format: wav`・`speed`・`irodori: { seed, caption }`
//!   （caption は声の `style`）。`params.extra` は本文にそのまま足す
//! - 参照音声が優先: 参照音声がある声は `voice` を参照音声の版ごとの ID（[`server_voice_id`]）にし、
//!   `voice_id` はその ID の名前に使うだけ。caption は送らない（参照音声と食い違うと声が不安定になる）
//! - 認証: `TOKEN` があれば `Authorization: Bearer`
//! - 参照音声の同期（Irodori）: サーバー側の声の ID は `{voice_id}-{ハッシュの先頭 12 文字}`
//!   （[`server_voice_id`]）。`GET {endpoint}/audio/voices` に無ければ `POST {endpoint}/audio/voices`
//!   （multipart の `file` と `voice_id`）。既にあれば上書きしないので、同じ声の版が変わっても、
//!   並行するジョブや複数のワーカーが互いの参照音声を置き換えない
//! - 絵文字は口調・効果音の指定になるのでそのまま渡す

use super::{error_body, SynthesisRequest, TtsEngine};
use crate::{config::Secret, models::RefAudio};
use anyhow::{bail, Context, Result};
use async_trait::async_trait;
use reqwest::{multipart, RequestBuilder, StatusCode};
use serde_json::{json, Map, Value};
use std::{collections::HashSet, sync::Mutex, time::Duration};
use tracing::instrument;

/// `params.extra` で上書きさせない本文のキー
const RESERVED_KEYS: [&str; 4] = ["model", "input", "voice", "response_format"];

/// `TTS_ENGINE_<NAME>_MODEL` が無いときの model
pub const DEFAULT_MODEL: &str = "irodori-tts";

/// サーバー側の声の ID に入れる参照音声のハッシュの長さ
const VOICE_HASH_LEN: usize = 12;

/// サーバー側の声の ID に使える文字（ASCII の英数字・`-`・`_`）だけにする。それ以外は `_` に置き換える
fn sanitize_id_part(s: &str) -> String {
    s.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

/// 参照音声の版ごとのサーバー側の声の ID（`{namespace}-{voice_id}-{ハッシュの先頭 12 文字}`）。
/// 名前空間（テナント）ごとに分け、同じ slug の声が衝突しないようにする。
/// 各部分は [`sanitize_id_part`] で正規化する。区切りの `-` が部分の中にあって ID が重なっても、
/// 参照音声のハッシュが同じ（= 同じ音声）ときだけなので害は無い。
pub fn server_voice_id(namespace: &str, voice_id: &str, hash: &str) -> String {
    let hash: String = hash
        .chars()
        .filter(char::is_ascii_alphanumeric)
        .take(VOICE_HASH_LEN)
        .collect();
    format!(
        "{}-{}-{hash}",
        sanitize_id_part(namespace),
        sanitize_id_part(voice_id)
    )
}

pub struct OpenAiEngine {
    endpoint: String,
    token: Option<Secret>,
    model: String,
    client: reqwest::Client,
    /// このプロセスでサーバーにあることを確かめた声の ID（[`server_voice_id`]）
    synced_voices: Mutex<HashSet<String>>,
}

impl std::fmt::Debug for OpenAiEngine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OpenAiEngine")
            .field("endpoint", &self.endpoint)
            .field("token", &self.token)
            .field("model", &self.model)
            .finish()
    }
}

/// 参照音声の拡張子を MIME タイプから決め、分からなければ中身から決める（サーバーは拡張子で形式を判断する）。
fn audio_extension(content_type: &str, bytes: &[u8]) -> &'static str {
    let mime = content_type.split(';').next().unwrap_or_default().trim();
    match mime.to_ascii_lowercase().as_str() {
        "audio/wav" | "audio/wave" | "audio/x-wav" | "audio/vnd.wave" => return "wav",
        "audio/flac" | "audio/x-flac" => return "flac",
        "audio/ogg" | "audio/opus" => return "ogg",
        "audio/mpeg" | "audio/mp3" => return "mp3",
        "audio/mp4" | "audio/m4a" | "audio/x-m4a" => return "m4a",
        _ => {}
    }
    if bytes.starts_with(b"RIFF") {
        "wav"
    } else if bytes.starts_with(b"fLaC") {
        "flac"
    } else if bytes.starts_with(b"OggS") {
        "ogg"
    } else if bytes.starts_with(b"ID3")
        || (bytes.len() > 1 && bytes[0] == 0xFF && bytes[1] & 0xE0 == 0xE0)
    {
        "mp3"
    } else if bytes.len() > 8 && &bytes[4..8] == b"ftyp" {
        "m4a"
    } else {
        "wav"
    }
}

impl OpenAiEngine {
    pub fn new(
        endpoint: impl Into<String>,
        token: Option<Secret>,
        model: Option<String>,
        timeout: Duration,
    ) -> Result<Self> {
        Ok(Self {
            endpoint: endpoint.into().trim_end_matches('/').to_string(),
            token,
            model: model.unwrap_or_else(|| DEFAULT_MODEL.to_string()),
            client: reqwest::Client::builder().timeout(timeout).build()?,
            synced_voices: Mutex::new(HashSet::new()),
        })
    }

    fn auth(&self, req: RequestBuilder) -> RequestBuilder {
        match &self.token {
            Some(token) => req.bearer_auth(token.expose()),
            None => req,
        }
    }

    /// 合成に使うサーバー側の声の ID（参照音声があれば版ごとの ID）
    fn speech_voice(req: SynthesisRequest<'_>) -> String {
        match &req.voice.ref_audio {
            Some(ref_audio) => server_voice_id(req.namespace, &req.voice.voice_id, &ref_audio.hash),
            None => req.voice.voice_id.clone(),
        }
    }

    fn speech_body(&self, req: SynthesisRequest<'_>) -> Value {
        let voice = req.voice;
        let params = &voice.params;
        let ignored: Vec<&str> = [
            ("pitch", params.pitch),
            ("intonation", params.intonation),
            ("volume", params.volume),
        ]
        .into_iter()
        .filter_map(|(name, v)| v.map(|_| name))
        .collect();
        if !ignored.is_empty() {
            tracing::warn!(
                "params {ignored:?} are not supported by OpenAI compatible engine; ignored (voice {})",
                voice.voice_id
            );
        }

        let mut body = Map::new();
        body.insert("model".to_string(), json!(self.model));
        body.insert("input".to_string(), json!(req.text));
        body.insert("voice".to_string(), json!(Self::speech_voice(req)));
        body.insert("response_format".to_string(), json!("wav"));
        if let Some(speed) = params.speed {
            body.insert("speed".to_string(), json!(speed));
        }
        let mut irodori = Map::new();
        if let Some(seed) = voice.seed {
            irodori.insert("seed".to_string(), json!(seed));
        }
        // 参照音声がある声は参照音声が声を決める。caption（声の caption・セリフの style）を一緒に渡すと、
        // 参照音声と食い違って声が不安定になる（Irodori-TTS v4.1 のモデルカード）ので送らない
        match (&voice.style, &voice.ref_audio) {
            (Some(style), None) => {
                irodori.insert("caption".to_string(), json!(style));
            }
            (Some(_), Some(_)) => {
                tracing::debug!(
                    "caption is not sent because voice {} has a reference audio",
                    voice.voice_id
                );
            }
            (None, _) => {}
        }
        if !irodori.is_empty() {
            body.insert("irodori".to_string(), Value::Object(irodori));
        }
        for (key, value) in &params.extra {
            // 本文・声・model・形式はキャッシュのキーと合わなくなるので上書きさせない
            if RESERVED_KEYS.contains(&key.as_str()) {
                tracing::warn!("params.extra.{key} is ignored (voice {})", voice.voice_id);
                continue;
            }
            body.insert(key.clone(), value.clone());
        }
        Value::Object(body)
    }

    async fn has_voice(&self, voice_id: &str) -> Result<bool> {
        let url = format!("{}/audio/voices", self.endpoint);
        let res = self
            .auth(self.client.get(&url))
            .send()
            .await
            .with_context(|| format!("GET {url}"))?;
        if !res.status().is_success() {
            bail!("GET {url}: {} {}", res.status(), error_body(res).await);
        }
        let list: Value = res.json().await.with_context(|| format!("GET {url}"))?;
        Ok(list["data"]
            .as_array()
            .is_some_and(|voices| voices.iter().any(|v| v["id"] == voice_id)))
    }

    fn voice_file(voice_id: &str, ref_audio: &RefAudio) -> Result<multipart::Part> {
        let extension = audio_extension(&ref_audio.content_type, &ref_audio.bytes);
        let file_name = format!("{voice_id}.{extension}");
        Ok(multipart::Part::bytes(ref_audio.bytes.to_vec())
            .file_name(file_name)
            .mime_str("application/octet-stream")?)
    }

    /// 参照音声を POST する。他のジョブが先に同じ版を上げていた（409）ときもそのまま使う。
    async fn upload_voice(&self, voice_id: &str, ref_audio: &RefAudio) -> Result<()> {
        let url = format!("{}/audio/voices", self.endpoint);
        let form = multipart::Form::new()
            .part("file", Self::voice_file(voice_id, ref_audio)?)
            .text("voice_id", voice_id.to_string());
        let res = self
            .auth(self.client.post(&url))
            .multipart(form)
            .send()
            .await
            .with_context(|| format!("POST {url}"))?;
        let status = res.status();
        if !status.is_success() && status != StatusCode::CONFLICT {
            bail!("POST {url}: {status} {}", error_body(res).await);
        }
        Ok(())
    }
}

#[async_trait]
impl TtsEngine for OpenAiEngine {
    #[instrument(skip(self, req), fields(voice = %req.voice.voice_id))]
    async fn synthesize(&self, req: SynthesisRequest<'_>) -> Result<Vec<u8>> {
        let url = format!("{}/audio/speech", self.endpoint);
        let res = self
            .auth(self.client.post(&url))
            .json(&self.speech_body(req))
            .send()
            .await
            .with_context(|| format!("POST {url}"))?;
        if !res.status().is_success() {
            // サーバーの再起動などで参照音声が消えたかもしれないので、次はあるかを確かめ直す
            let voice = Self::speech_voice(req);
            self.synced_voices.lock().unwrap().remove(&voice);
            bail!(
                "POST {url}: voice={voice}, text={}, {} {}",
                req.text,
                res.status(),
                error_body(res).await
            );
        }
        Ok(res.bytes().await?.to_vec())
    }

    #[instrument(skip(self, ref_audio))]
    async fn ensure_voice(
        &self,
        namespace: &str,
        voice_id: &str,
        ref_audio: &RefAudio,
    ) -> Result<()> {
        let server_id = server_voice_id(namespace, voice_id, &ref_audio.hash);
        if self.synced_voices.lock().unwrap().contains(&server_id) {
            return Ok(());
        }
        // 版ごとに ID が違うので、既にあれば同じ参照音声。上書きしない
        if !self.has_voice(&server_id).await? {
            self.upload_voice(&server_id, ref_audio).await?;
            tracing::info!("uploaded reference audio of voice {voice_id} as {server_id}");
        }
        self.synced_voices.lock().unwrap().insert(server_id);
        Ok(())
    }

    async fn health(&self) -> Result<()> {
        let url = format!("{}/models", self.endpoint);
        let res = self
            .auth(self.client.get(&url))
            .send()
            .await
            .with_context(|| format!("GET {url}"))?;
        if !res.status().is_success() {
            bail!("GET {url}: {} {}", res.status(), error_body(res).await);
        }
        Ok(())
    }

    async fn version(&self) -> String {
        self.model.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_audio_extension() {
        assert_eq!(audio_extension("", b"RIFF....WAVE"), "wav");
        assert_eq!(audio_extension("", b"fLaC...."), "flac");
        assert_eq!(audio_extension("", b"ID3....."), "mp3");
        assert_eq!(audio_extension("", b"OggS...."), "ogg");
        assert_eq!(audio_extension("audio/unknown", b"????"), "wav");
    }

    #[test]
    fn audio_extension_prefers_content_type() {
        assert_eq!(audio_extension("audio/mpeg", b"RIFF....WAVE"), "mp3");
        assert_eq!(audio_extension("audio/x-wav", b"ID3....."), "wav");
        assert_eq!(audio_extension("audio/flac", b""), "flac");
        assert_eq!(audio_extension("audio/ogg; codecs=opus", b""), "ogg");
        assert_eq!(audio_extension("audio/x-m4a", b""), "m4a");
    }

    #[test]
    fn server_voice_id_has_namespace_and_hash_prefix() {
        assert_eq!(
            server_voice_id("botcast", "alice", "0123456789abcdef0123"),
            "botcast-alice-0123456789ab"
        );
        assert_eq!(
            server_voice_id("botcast", "alice", "abc"),
            "botcast-alice-abc"
        );
        // テナントが違えば同じ slug でも別の ID
        assert_ne!(
            server_voice_id("t1", "alice", "abc"),
            server_voice_id("t2", "alice", "abc")
        );
        // サーバーが受け付ける文字にそろえる
        assert_eq!(
            server_voice_id("my-tenant", "narrator-1 声/x", "ab-cd"),
            "my-tenant-narrator-1___x-abcd"
        );
    }
}

//! OpenAI 互換の窓口（`POST {endpoint}/audio/speech`）。Irodori-TTS-Server・OpenAI など。
//!
//! - 本文: `model`・`input`・`voice`・`response_format: wav`・`speed`・`irodori: { seed, caption }`
//!   （caption は声の `style`）。`params.extra` は本文にそのまま足す
//! - 認証: `TOKEN` があれば `Authorization: Bearer`
//! - 参照音声の同期（Irodori）: `GET {endpoint}/audio/voices` に `voice_id` が無ければ
//!   `POST {endpoint}/audio/voices`（multipart の `file` と `voice_id`）。サーバーはハッシュを持たないので、
//!   このプロセスで同期したハッシュと違う（またはまだ同期していない）ときは
//!   `PUT {endpoint}/audio/voices/{voice_id}` で置き換える
//! - 絵文字は口調・効果音の指定になるのでそのまま渡す

use super::{error_body, SynthesisRequest, TtsEngine};
use crate::config::Secret;
use anyhow::{bail, Context, Result};
use async_trait::async_trait;
use reqwest::{multipart, RequestBuilder, StatusCode};
use serde_json::{json, Map, Value};
use std::{collections::HashMap, sync::Mutex, time::Duration};
use tracing::instrument;

/// `TTS_ENGINE_<NAME>_MODEL` が無いときの model
pub const DEFAULT_MODEL: &str = "irodori-tts";

pub struct OpenAiEngine {
    endpoint: String,
    token: Option<Secret>,
    model: String,
    client: reqwest::Client,
    /// voice_id → このプロセスで同期した参照音声のハッシュ
    synced_voices: Mutex<HashMap<String, String>>,
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

/// 参照音声の拡張子を中身から決める（サーバーは拡張子で形式を判断する）。
fn audio_extension(bytes: &[u8]) -> &'static str {
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
            synced_voices: Mutex::new(HashMap::new()),
        })
    }

    fn auth(&self, req: RequestBuilder) -> RequestBuilder {
        match &self.token {
            Some(token) => req.bearer_auth(token.expose()),
            None => req,
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
        body.insert("voice".to_string(), json!(voice.voice_id));
        body.insert("response_format".to_string(), json!("wav"));
        if let Some(speed) = params.speed {
            body.insert("speed".to_string(), json!(speed));
        }
        let mut irodori = Map::new();
        if let Some(seed) = voice.seed {
            irodori.insert("seed".to_string(), json!(seed));
        }
        if let Some(style) = &voice.style {
            irodori.insert("caption".to_string(), json!(style));
        }
        if !irodori.is_empty() {
            body.insert("irodori".to_string(), Value::Object(irodori));
        }
        for (key, value) in &params.extra {
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

    fn voice_file(voice_id: &str, bytes: &[u8]) -> Result<multipart::Part> {
        let file_name = format!("{voice_id}.{}", audio_extension(bytes));
        Ok(multipart::Part::bytes(bytes.to_vec())
            .file_name(file_name)
            .mime_str("application/octet-stream")?)
    }

    async fn upload_voice(
        &self,
        voice_id: &str,
        bytes: &[u8],
        replace: bool,
    ) -> Result<StatusCode> {
        let (method, url, form) = if replace {
            let url = format!(
                "{}/audio/voices/{}",
                self.endpoint,
                urlencoding::encode(voice_id)
            );
            let form = multipart::Form::new().part("file", Self::voice_file(voice_id, bytes)?);
            (reqwest::Method::PUT, url, form)
        } else {
            let url = format!("{}/audio/voices", self.endpoint);
            let form = multipart::Form::new()
                .part("file", Self::voice_file(voice_id, bytes)?)
                .text("voice_id", voice_id.to_string());
            (reqwest::Method::POST, url, form)
        };
        let res = self
            .auth(self.client.request(method.clone(), &url))
            .multipart(form)
            .send()
            .await
            .with_context(|| format!("{method} {url}"))?;
        let status = res.status();
        if !status.is_success() && status != StatusCode::CONFLICT {
            bail!("{method} {url}: {status} {}", error_body(res).await);
        }
        Ok(status)
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
            bail!(
                "POST {url}: voice={}, text={}, {} {}",
                req.voice.voice_id,
                req.text,
                res.status(),
                error_body(res).await
            );
        }
        Ok(res.bytes().await?.to_vec())
    }

    #[instrument(skip(self, bytes))]
    async fn ensure_voice(&self, voice_id: &str, hash: &str, bytes: &[u8]) -> Result<()> {
        let synced = self.synced_voices.lock().unwrap().get(voice_id).cloned();
        if synced.as_deref() == Some(hash) {
            return Ok(());
        }
        let exists = self.has_voice(voice_id).await?;
        let replace = if exists {
            true
        } else {
            // 他のジョブが先にアップロードしていたら置き換える
            self.upload_voice(voice_id, bytes, false).await? == StatusCode::CONFLICT
        };
        if replace {
            self.upload_voice(voice_id, bytes, true).await?;
        }
        tracing::info!("synced reference audio of voice {voice_id} ({hash})");
        self.synced_voices
            .lock()
            .unwrap()
            .insert(voice_id.to_string(), hash.to_string());
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

    fn cache_salt(&self) -> String {
        self.model.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_audio_extension() {
        assert_eq!(audio_extension(b"RIFF....WAVE"), "wav");
        assert_eq!(audio_extension(b"fLaC...."), "flac");
        assert_eq!(audio_extension(b"ID3....."), "mp3");
        assert_eq!(audio_extension(b"OggS...."), "ogg");
        assert_eq!(audio_extension(b"????"), "wav");
    }
}

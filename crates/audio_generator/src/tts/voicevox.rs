//! VOICEVOX 互換の窓口（`/audio_query` → `/synthesis`）。VOICEVOX・AivisSpeech など。

use super::{error_body, SynthesisRequest, TtsEngine};
use crate::{models::VoiceParams, text::strip_emoji};
use anyhow::{bail, Context, Result};
use async_trait::async_trait;
use serde_json::Value;
use std::time::Duration;
use tracing::instrument;

#[derive(Debug, Clone)]
pub struct VoicevoxEngine {
    endpoint: String,
    client: reqwest::Client,
}

impl VoicevoxEngine {
    pub fn new(endpoint: impl Into<String>, timeout: Duration) -> Result<Self> {
        Ok(Self {
            endpoint: endpoint.into().trim_end_matches('/').to_string(),
            client: reqwest::Client::builder().timeout(timeout).build()?,
        })
    }

    #[instrument(skip(self))]
    async fn query(&self, text: &str, speaker: &str) -> Result<Value> {
        let url = format!(
            "{}/audio_query?text={}&speaker={}",
            self.endpoint,
            urlencoding::encode(text),
            urlencoding::encode(speaker),
        );
        let res = self.client.post(&url).send().await?;
        if !res.status().is_success() {
            bail!(
                "audio_query failed: speaker={speaker}, text={text}, {} {}",
                res.status(),
                error_body(res).await
            );
        }
        Ok(res.json().await?)
    }

    #[instrument(skip(self, query))]
    async fn synthesis(&self, query: &Value, speaker: &str) -> Result<Vec<u8>> {
        let url = format!(
            "{}/synthesis?speaker={}",
            self.endpoint,
            urlencoding::encode(speaker)
        );
        let res = self.client.post(&url).json(query).send().await?;
        if !res.status().is_success() {
            bail!(
                "synthesis failed: speaker={speaker}, {} {}",
                res.status(),
                error_body(res).await
            );
        }
        Ok(res.bytes().await?.to_vec())
    }
}

/// params を audio_query の結果に上書きし、`extra` を足す。
fn apply_params(query: &mut Value, params: &VoiceParams) {
    let Some(obj) = query.as_object_mut() else {
        return;
    };
    let scales = [
        ("speedScale", params.speed),
        ("pitchScale", params.pitch),
        ("intonationScale", params.intonation),
        ("volumeScale", params.volume),
    ];
    for (key, value) in scales {
        if let Some(value) = value {
            obj.insert(key.to_string(), Value::from(value));
        }
    }
    for (key, value) in &params.extra {
        obj.insert(key.clone(), value.clone());
    }
}

#[async_trait]
impl TtsEngine for VoicevoxEngine {
    async fn synthesize(&self, req: SynthesisRequest<'_>) -> Result<Vec<u8>> {
        let voice = req.voice;
        // style は声の styles（名前 → スタイル ID）で引き、無ければ声の既定のスタイル
        let speaker = match &voice.style {
            Some(style) => match voice.styles.get(style) {
                Some(id) => id.as_str(),
                None => {
                    tracing::warn!(
                        "style {style:?} is not in styles of voice {}; using default style",
                        voice.voice_id
                    );
                    voice.voice_id.as_str()
                }
            },
            None => voice.voice_id.as_str(),
        };
        let mut query = self.query(req.text, speaker).await?;
        apply_params(&mut query, &voice.params);
        self.synthesis(&query, speaker).await
    }

    async fn health(&self) -> Result<()> {
        let url = format!("{}/version", self.endpoint);
        let res = self
            .client
            .get(&url)
            .send()
            .await
            .with_context(|| format!("GET {url}"))?;
        if !res.status().is_success() {
            bail!("GET {url}: {} {}", res.status(), error_body(res).await);
        }
        Ok(())
    }

    fn prepare_text(&self, text: &str) -> String {
        strip_emoji(text).trim().to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn params_override_query() {
        let mut query = json!({"speedScale": 1.0, "pitchScale": 0.0, "accent_phrases": []});
        let mut extra = serde_json::Map::new();
        extra.insert("outputSamplingRate".to_string(), json!(48000));
        apply_params(
            &mut query,
            &VoiceParams {
                speed: Some(1.2),
                pitch: None,
                intonation: Some(1.5),
                volume: None,
                extra,
            },
        );
        assert_eq!(
            query,
            json!({
                "speedScale": 1.2,
                "pitchScale": 0.0,
                "intonationScale": 1.5,
                "outputSamplingRate": 48000,
                "accent_phrases": []
            })
        );
    }
}

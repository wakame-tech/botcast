//! 設定から作った TTS エンジンの一覧。worker の起動時に 1 度だけ作り、ジョブで使い回す。

use crate::{
    config::{EngineKind, TtsConfig},
    models::{resolve_legacy_sections, ResolvedSection, ResolvedVoice, Section},
    tts::{openai::OpenAiEngine, voicevox::VoicevoxEngine, SynthesisRequest, TtsEngine},
};
use anyhow::{anyhow, bail, Context, Result};
use std::{collections::BTreeMap, path::PathBuf, sync::Arc};
use tokio::sync::Semaphore;

/// ログに出す URL（userinfo と query を落とす）
fn redact_url(url: &str) -> String {
    match reqwest::Url::parse(url) {
        Ok(mut url) => {
            let _ = url.set_username("");
            let _ = url.set_password(None);
            url.set_query(None);
            url.to_string()
        }
        Err(_) => "(invalid url)".to_string(),
    }
}

#[derive(Debug, Clone)]
pub(crate) struct EngineEntry {
    pub(crate) engine: Arc<dyn TtsEngine>,
    /// エンジンごとの同時実行数（`MAX_CONCURRENCY`）
    pub(crate) permits: Arc<Semaphore>,
}

#[derive(Debug, Clone, Default)]
pub struct TtsRegistry {
    engines: BTreeMap<String, EngineEntry>,
    default_engine: Option<String>,
    cache_dir: Option<PathBuf>,
}

impl TtsRegistry {
    /// 環境変数から設定を読み、検証してエンジンを作る。
    pub fn from_env() -> Result<Self> {
        Self::from_config(&TtsConfig::from_env().context("invalid TTS engine config")?)
    }

    pub fn from_config(config: &TtsConfig) -> Result<Self> {
        let mut registry = Self {
            default_engine: config.default_engine.clone(),
            cache_dir: config.cache_dir.clone(),
            ..Default::default()
        };
        for engine in &config.engines {
            let tts: Arc<dyn TtsEngine> = match engine.kind {
                EngineKind::Voicevox => {
                    Arc::new(VoicevoxEngine::new(&engine.endpoint, engine.timeout)?)
                }
                EngineKind::OpenAi => Arc::new(OpenAiEngine::new(
                    &engine.endpoint,
                    engine.token.clone(),
                    engine.model.clone(),
                    engine.timeout,
                )?),
            };
            tracing::info!(
                "TTS engine {}: {:?} {} (max_concurrency={})",
                engine.name,
                engine.kind,
                redact_url(&engine.endpoint),
                engine.max_concurrency
            );
            registry = registry.with_engine(&engine.name, tts, engine.max_concurrency);
        }
        if registry.engines.is_empty() {
            tracing::warn!("no TTS engine is configured (TTS_ENGINES / VOICEVOX_ENDPOINT)");
        }
        Ok(registry)
    }

    /// エンジンを足す（テストでフェイクのエンジンを入れるのにも使う）。
    pub fn with_engine(
        mut self,
        name: &str,
        engine: Arc<dyn TtsEngine>,
        max_concurrency: usize,
    ) -> Self {
        self.engines.insert(
            name.to_string(),
            EngineEntry {
                engine,
                permits: Arc::new(Semaphore::new(max_concurrency.max(1))),
            },
        );
        if self.default_engine.is_none() {
            self.default_engine = Some(name.to_string());
        }
        self
    }

    pub fn with_cache_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.cache_dir = Some(dir.into());
        self
    }

    pub fn cache_dir(&self) -> Option<&PathBuf> {
        self.cache_dir.as_ref()
    }

    pub fn engine_names(&self) -> impl Iterator<Item = &str> {
        self.engines.keys().map(String::as_str)
    }

    /// 旧形式の speaker に使うエンジンの名前。
    pub fn default_engine(&self) -> Result<&str> {
        self.default_engine.as_deref().ok_or_else(|| {
            anyhow!("no TTS engine is configured (set TTS_ENGINES or VOICEVOX_ENDPOINT)")
        })
    }

    /// 旧形式のセクション（`speaker` に VOICEVOX のスタイル ID）を既定のエンジンの声として解決する。
    /// セリフがあるのにエンジンが 1 つも無ければエラー。
    pub fn resolve_legacy_sections(&self, sections: Vec<Section>) -> Result<Vec<ResolvedSection>> {
        let has_serif = sections
            .iter()
            .any(|s| matches!(s, Section::SerifSection(_)));
        let default_engine = if has_serif {
            self.default_engine()?
        } else {
            ""
        };
        Ok(resolve_legacy_sections(sections, default_engine))
    }

    pub(crate) fn entry(&self, name: &str) -> Result<&EngineEntry> {
        self.engines.get(name).ok_or_else(|| {
            anyhow!(
                "TTS engine {name:?} is not configured (configured: {:?})",
                self.engines.keys().collect::<Vec<_>>()
            )
        })
    }

    /// 本文 1 つをそのまま合成し、エンジンが返した音声のバイト列を返す（声の試聴に使う）。
    ///
    /// 文に分けず、キャッシュ・正規化もしない（VOICEVOX 互換・OpenAI 互換とも wav が返る）。
    /// 参照音声があればエンジンに同期してから合成する。同時実行数（`MAX_CONCURRENCY`）は
    /// [`crate::generate_audio`] と共有するので、ジョブの合成と合わせて上限を超えない。
    pub async fn synthesize_text(
        &self,
        namespace: &str,
        voice: &ResolvedVoice,
        text: &str,
    ) -> Result<Vec<u8>> {
        let entry = self.entry(&voice.engine)?;
        let prepared = entry.engine.prepare_text(text);
        if prepared.trim().is_empty() {
            bail!("no text to synthesize");
        }
        if let Some(ref_audio) = &voice.ref_audio {
            entry
                .engine
                .ensure_voice(namespace, &voice.voice_id, ref_audio)
                .await
                .with_context(|| {
                    format!(
                        "failed to sync reference audio of voice {:?} to TTS engine {:?}",
                        voice.voice_id, voice.engine
                    )
                })?;
        }
        let _permit = entry.permits.acquire().await?;
        entry
            .engine
            .synthesize(SynthesisRequest {
                text: &prepared,
                voice,
                namespace,
            })
            .await
            .with_context(|| format!("TTS engine {:?} failed to synthesize", voice.engine))
    }

    /// エンジンに届くかを確かめる。届かなければ、どのエンジンかを書いたエラーを返す。
    pub async fn health_check<'a>(&self, names: impl IntoIterator<Item = &'a str>) -> Result<()> {
        for name in names {
            self.entry(name)?
                .engine
                .health()
                .await
                .with_context(|| format!("TTS engine {name:?} is unreachable"))?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redact_url_drops_credentials() {
        assert_eq!(
            redact_url("https://user:pass@gpu:8088/v1?api_key=x"),
            "https://gpu:8088/v1"
        );
    }
}

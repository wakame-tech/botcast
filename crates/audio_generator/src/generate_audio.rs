//! エンジン非依存の共通の層: 文の分割 → 合成（キャッシュ・エンジンごとの同時実行数）→ 正規化 →
//! 長さと SRT → 結合。

use crate::{
    ffmpeg::{concat_audios, normalize, slice_and_normalize, wav_duration},
    models::{AudioSection, ResolvedSection, ResolvedVoice},
    registry::{EngineEntry, TtsRegistry},
    text::{split_text, MAX_SENTENCE_CHARS},
    tts::SynthesisRequest,
    workdir::WorkDir,
};
use anyhow::{bail, Context, Result};
use futures::future::try_join_all;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use srtlib::{Subtitle, Subtitles, Timestamp};
use std::{
    collections::{BTreeSet, HashSet},
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};
use tokio::sync::Semaphore;
use tracing::instrument;

/// 音声のセクションのダウンロードの同時実行数
const MAX_CONCURRENT_DOWNLOADS: usize = 2;

pub struct SynthesisResult {
    pub out_path: PathBuf,
    pub srt: String,
    pub duration_sec: f64,
}

/// キーを並べ替えた JSON の文字列（キャッシュのキーを安定させる）
fn canonical_json(value: &Value) -> String {
    match value {
        Value::Object(map) => {
            let mut keys: Vec<&String> = map.keys().collect();
            keys.sort();
            let fields: Vec<String> = keys
                .into_iter()
                .map(|k| format!("{}:{}", Value::from(k.as_str()), canonical_json(&map[k])))
                .collect();
            format!("{{{}}}", fields.join(","))
        }
        Value::Array(items) => {
            let items: Vec<String> = items.iter().map(canonical_json).collect();
            format!("[{}]", items.join(","))
        }
        other => other.to_string(),
    }
}

/// 合成結果のキャッシュのキー:
/// `hash(engine, エンジンの設定（model など）, voice_id, ref_audio のハッシュ, params, style, styles, seed, text)`
pub fn cache_key(engine: &str, salt: &str, voice: &ResolvedVoice, text: &str) -> String {
    let value = json!({
        "version": 1,
        "engine": engine,
        "salt": salt,
        "voice_id": voice.voice_id,
        "ref_audio": voice.ref_audio.as_ref().map(|r| &r.hash),
        "params": voice.params,
        "style": voice.style,
        "styles": voice.styles,
        "seed": voice.seed,
        "text": text,
    });
    hex::encode(Sha256::digest(canonical_json(&value).as_bytes()))
}

enum Item {
    Tts {
        entry: EngineEntry,
        voice: Arc<ResolvedVoice>,
        /// エンジンに渡す本文
        text: String,
        /// 字幕に出す本文
        subtitle: String,
        /// 正規化した wav のキャッシュ
        path: PathBuf,
    },
    Audio {
        section: AudioSection,
        path: PathBuf,
    },
}

impl Item {
    fn path(&self) -> &Path {
        match self {
            Item::Tts { path, .. } | Item::Audio { path, .. } => path,
        }
    }

    fn subtitle(&self) -> &str {
        match self {
            Item::Tts { subtitle, .. } => subtitle,
            Item::Audio { .. } => "♪",
        }
    }
}

struct Renderer<'a> {
    dir: &'a Path,
    cache_dir: PathBuf,
    client: reqwest::Client,
    downloads: Semaphore,
}

impl Renderer<'_> {
    async fn render(&self, i: usize, item: &Item) -> Result<()> {
        match item {
            Item::Tts {
                entry,
                voice,
                text,
                path,
                ..
            } => {
                if path.exists() {
                    tracing::debug!("[{i}] cache hit: {text}");
                    return Ok(());
                }
                let bytes = {
                    let _permit = entry.permits.acquire().await?;
                    tracing::info!("[{i}] {}: {text}", voice.engine);
                    entry
                        .engine
                        .synthesize(SynthesisRequest { text, voice })
                        .await
                        .with_context(|| {
                            format!("TTS engine {:?} failed to synthesize", voice.engine)
                        })?
                };
                let raw = self.dir.join(format!("raw_{i}.bin"));
                tokio::fs::write(&raw, &bytes).await?;
                // 書きかけのファイルをキャッシュとして読まないよう、別名で作ってから置く
                let tmp = self
                    .cache_dir
                    .join(format!("{}.tmp", uuid::Uuid::new_v4().simple()));
                normalize(&raw, &tmp).await?;
                tokio::fs::rename(&tmp, path).await?;
                tokio::fs::remove_file(&raw).await?;
                Ok(())
            }
            Item::Audio { section, path } => {
                let bin = {
                    let _permit = self.downloads.acquire().await?;
                    let res = self
                        .client
                        .get(&section.url)
                        .send()
                        .await
                        .and_then(|res| res.error_for_status())
                        .with_context(|| format!("failed to download {}", section.url))?;
                    res.bytes().await?
                };
                let download = self.dir.join(format!("download_{i}"));
                tokio::fs::write(&download, &bin).await?;
                slice_and_normalize(&download, path, section.from, section.to).await?;
                tokio::fs::remove_file(&download).await?;
                Ok(())
            }
        }
    }
}

/// セクションの音声を作り、`episode.mp3` と SRT を返す。
pub async fn generate_audio(
    work_dir: &WorkDir,
    registry: &TtsRegistry,
    sections: Vec<ResolvedSection>,
) -> Result<SynthesisResult> {
    generate_audio_in_dir(work_dir.dir(), registry, sections).await
}

/// [`generate_audio`] の本体。`dir` に途中のファイルと `episode.mp3` を置く。
#[instrument(skip(registry, sections))]
pub async fn generate_audio_in_dir(
    dir: &Path,
    registry: &TtsRegistry,
    sections: Vec<ResolvedSection>,
) -> Result<SynthesisResult> {
    let cache_dir = registry
        .cache_dir()
        .cloned()
        .unwrap_or_else(|| dir.join("tts_cache"));
    tokio::fs::create_dir_all(&cache_dir)
        .await
        .with_context(|| format!("failed to create {}", cache_dir.display()))?;

    // 文に分け、各文のキャッシュの置き場を決める
    let mut items = vec![];
    for section in sections {
        match section {
            ResolvedSection::Serif { voice, text } => {
                let entry = registry.entry(&voice.engine)?.clone();
                let voice = Arc::new(voice);
                for sentence in split_text(&text, MAX_SENTENCE_CHARS) {
                    let prepared = entry.engine.prepare_text(&sentence);
                    if prepared.trim().is_empty() {
                        continue;
                    }
                    let key =
                        cache_key(&voice.engine, &entry.engine.cache_salt(), &voice, &prepared);
                    items.push(Item::Tts {
                        entry: entry.clone(),
                        voice: voice.clone(),
                        text: prepared,
                        subtitle: sentence,
                        path: cache_dir.join(format!("{key}.wav")),
                    });
                }
            }
            ResolvedSection::Audio(section) => {
                let path = dir.join(format!("audio_{}.wav", items.len()));
                items.push(Item::Audio { section, path });
            }
        }
    }
    if items.is_empty() {
        bail!("no sentence to synthesize");
    }

    // キャッシュに無い文があるエンジンだけ、届くかを確かめ、参照音声を同期する
    let pending: Vec<(&EngineEntry, &ResolvedVoice)> = items
        .iter()
        .filter_map(|item| match item {
            Item::Tts {
                entry, voice, path, ..
            } if !path.exists() => Some((entry, voice.as_ref())),
            _ => None,
        })
        .collect();
    let engines: BTreeSet<&str> = pending.iter().map(|(_, v)| v.engine.as_str()).collect();
    registry.health_check(engines).await?;
    let mut synced = HashSet::new();
    for (entry, voice) in &pending {
        let Some(ref_audio) = &voice.ref_audio else {
            continue;
        };
        if synced.insert((&voice.engine, &voice.voice_id, &ref_audio.hash)) {
            entry
                .engine
                .ensure_voice(&voice.voice_id, &ref_audio.hash, &ref_audio.bytes)
                .await
                .with_context(|| {
                    format!(
                        "failed to sync reference audio of voice {:?} to TTS engine {:?}",
                        voice.voice_id, voice.engine
                    )
                })?;
        }
    }

    let renderer = Renderer {
        dir,
        cache_dir,
        client: reqwest::Client::new(),
        downloads: Semaphore::new(MAX_CONCURRENT_DOWNLOADS),
    };
    try_join_all(
        items
            .iter()
            .enumerate()
            .map(|(i, item)| renderer.render(i, item)),
    )
    .await?;

    // 長さと SRT は正規化後の wav から測る
    let mut srt = Subtitles::new();
    let mut duration = Duration::ZERO;
    let mmss = |d: &Duration| format!("{:02}:{:02}", d.as_secs() / 60, d.as_secs() % 60);
    for (i, item) in items.iter().enumerate() {
        let (start, end) = (duration, duration + wav_duration(item.path())?);
        tracing::info!("{} -> {}: {}", mmss(&start), mmss(&end), item.subtitle());
        srt.push(Subtitle::new(
            i,
            Timestamp::from_milliseconds(start.as_millis() as u32),
            Timestamp::from_milliseconds(end.as_millis() as u32),
            item.subtitle().to_string(),
        ));
        duration = end;
    }

    let paths: Vec<PathBuf> = items.iter().map(|item| item.path().to_path_buf()).collect();
    let out_path = concat_audios(dir, &paths).await?;

    Ok(SynthesisResult {
        out_path,
        srt: srt.to_string(),
        duration_sec: duration.as_secs_f64(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::VoiceParams;

    #[test]
    fn cache_key_depends_on_every_input() {
        let voice = ResolvedVoice {
            engine: "irodori".to_string(),
            voice_id: "alice".to_string(),
            ..Default::default()
        };
        let base = cache_key("irodori", "m", &voice, "こんにちは");
        assert_eq!(
            base,
            cache_key("irodori", "m", &voice.clone(), "こんにちは")
        );

        let variants = [
            cache_key("other", "m", &voice, "こんにちは"),
            cache_key("irodori", "m2", &voice, "こんにちは"),
            cache_key("irodori", "m", &voice, "こんばんは"),
            cache_key(
                "irodori",
                "m",
                &ResolvedVoice {
                    voice_id: "bob".to_string(),
                    ..voice.clone()
                },
                "こんにちは",
            ),
            cache_key(
                "irodori",
                "m",
                &ResolvedVoice {
                    params: VoiceParams {
                        speed: Some(1.1),
                        ..Default::default()
                    },
                    ..voice.clone()
                },
                "こんにちは",
            ),
            cache_key(
                "irodori",
                "m",
                &ResolvedVoice {
                    style: Some("明るく".to_string()),
                    ..voice.clone()
                },
                "こんにちは",
            ),
            cache_key(
                "irodori",
                "m",
                &ResolvedVoice {
                    seed: Some(1),
                    ..voice.clone()
                },
                "こんにちは",
            ),
            cache_key(
                "irodori",
                "m",
                &ResolvedVoice {
                    ref_audio: Some(crate::models::RefAudio {
                        hash: "abc".to_string(),
                        bytes: Arc::from(vec![1u8]),
                    }),
                    ..voice.clone()
                },
                "こんにちは",
            ),
        ];
        let unique: HashSet<&String> = variants.iter().chain([&base]).collect();
        assert_eq!(unique.len(), variants.len() + 1);
    }

    #[test]
    fn canonical_json_sorts_keys() {
        assert_eq!(
            canonical_json(&json!({"b": 1, "a": {"d": [1, {"f": 2, "e": 3}], "c": "x"}})),
            r#"{"a":{"c":"x","d":[1,{"e":3,"f":2}]},"b":1}"#
        );
    }
}

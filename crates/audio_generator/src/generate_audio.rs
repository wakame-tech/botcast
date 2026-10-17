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
    collections::{BTreeSet, HashMap, HashSet},
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};
use tokio::sync::Semaphore;
use tracing::instrument;

/// 音声のセクションのダウンロードの同時実行数
const MAX_CONCURRENT_DOWNLOADS: usize = 2;
/// 音声のセクションのダウンロードのタイムアウト
const DOWNLOAD_TIMEOUT: Duration = Duration::from_secs(120);
/// ffmpeg（正規化・切り出し）の同時実行数
const MAX_CONCURRENT_FFMPEG: usize = 4;
/// キャッシュする wav を作る処理の版。正規化の設定など、同じ入力から違う wav ができる変更をしたら上げる
pub const PIPELINE_VERSION: u32 = 1;

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
/// `hash(処理の版, engine, エンジンの版, voice_id, ref_audio のハッシュ, params, style, styles, seed, text)`
pub fn cache_key(engine: &str, engine_version: &str, voice: &ResolvedVoice, text: &str) -> String {
    let value = json!({
        "pipeline": PIPELINE_VERSION,
        "engine": engine,
        "engine_version": engine_version,
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

/// 途中のファイル。drop で消すので、失敗したときや他の文の失敗で future が捨てられたときも残さない
struct TempFile(PathBuf);

impl TempFile {
    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TempFile {
    fn drop(&mut self) {
        if let Err(e) = std::fs::remove_file(&self.0) {
            if e.kind() != std::io::ErrorKind::NotFound {
                tracing::warn!("failed to remove {}: {e}", self.0.display());
            }
        }
    }
}

/// キャッシュの名前空間（テナント ID など）はパスに使うので、英数字・`-`・`_` だけにする
fn validate_cache_namespace(namespace: &str) -> Result<()> {
    if namespace.is_empty()
        || !namespace
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        bail!("invalid TTS cache namespace {namespace:?} (use ASCII letters, digits, '-' or '_')");
    }
    Ok(())
}

struct Renderer<'a> {
    dir: &'a Path,
    /// 名前空間（参照音声のサーバー側の声の ID に使う）
    namespace: &'a str,
    cache_dir: PathBuf,
    client: reqwest::Client,
    downloads: Semaphore,
    ffmpeg: Semaphore,
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
                        .synthesize(SynthesisRequest {
                            text,
                            voice,
                            namespace: self.namespace,
                        })
                        .await
                        .with_context(|| {
                            format!("TTS engine {:?} failed to synthesize", voice.engine)
                        })?
                };
                let raw = TempFile(self.dir.join(format!("raw_{i}.bin")));
                // 書きかけのファイルをキャッシュとして読まないよう、別名で作ってから置く
                let tmp = TempFile(
                    self.cache_dir
                        .join(format!("{}.tmp", uuid::Uuid::new_v4().simple())),
                );
                tokio::fs::write(raw.path(), &bytes).await?;
                {
                    let _permit = self.ffmpeg.acquire().await?;
                    normalize(raw.path(), tmp.path()).await?;
                }
                tokio::fs::rename(tmp.path(), path).await?;
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
                let download = TempFile(self.dir.join(format!("download_{i}")));
                tokio::fs::write(download.path(), &bin).await?;
                let _permit = self.ffmpeg.acquire().await?;
                slice_and_normalize(download.path(), path, section.from, section.to).await
            }
        }
    }
}

/// セクションの音声を作り、`episode.mp3` と SRT を返す。
///
/// 合成結果のキャッシュは `cache_namespace`（worker はテナント ID）ごとに分ける。
pub async fn generate_audio(
    work_dir: &WorkDir,
    registry: &TtsRegistry,
    cache_namespace: &str,
    sections: Vec<ResolvedSection>,
) -> Result<SynthesisResult> {
    generate_audio_in_dir(work_dir.dir(), registry, cache_namespace, sections).await
}

/// [`generate_audio`] の本体。`dir` の下に実行ごとのディレクトリを作って途中のファイルと `episode.mp3` を置き、
/// キャッシュは `<TTS_CACHE_DIR（既定は dir/tts_cache）>/<cache_namespace>` に置く。
#[instrument(skip(registry, sections))]
pub async fn generate_audio_in_dir(
    dir: &Path,
    registry: &TtsRegistry,
    cache_namespace: &str,
    sections: Vec<ResolvedSection>,
) -> Result<SynthesisResult> {
    validate_cache_namespace(cache_namespace)?;
    let cache_dir = registry
        .cache_dir()
        .cloned()
        .unwrap_or_else(|| dir.join("tts_cache"))
        .join(cache_namespace);
    tokio::fs::create_dir_all(&cache_dir)
        .await
        .with_context(|| format!("failed to create {}", cache_dir.display()))?;
    // 途中のファイルと結果は実行ごとのディレクトリに置く（KEEP_WORKDIR で dir を共有しても衝突しない）
    let run_dir = dir.join(format!("run_{}", uuid::Uuid::new_v4().simple()));
    tokio::fs::create_dir_all(&run_dir)
        .await
        .with_context(|| format!("failed to create {}", run_dir.display()))?;
    let dir = run_dir.as_path();

    // 文に分け、各文のキャッシュの置き場を決める
    let mut items = vec![];
    // エンジンの版（キャッシュのキーに入れる）はジョブごとにエンジン 1 つにつき 1 回だけ取る
    let mut versions: HashMap<String, String> = HashMap::new();
    for section in sections {
        match section {
            ResolvedSection::Serif { voice, text } => {
                let entry = registry.entry(&voice.engine)?.clone();
                if !versions.contains_key(&voice.engine) {
                    versions.insert(voice.engine.clone(), entry.engine.version().await);
                }
                let engine_version = &versions[&voice.engine];
                let voice = Arc::new(voice);
                for sentence in split_text(&text, MAX_SENTENCE_CHARS) {
                    let prepared = entry.engine.prepare_text(&sentence);
                    if prepared.trim().is_empty() {
                        continue;
                    }
                    let key = cache_key(&voice.engine, engine_version, &voice, &prepared);
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
                .ensure_voice(cache_namespace, &voice.voice_id, ref_audio)
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
        namespace: cache_namespace,
        dir,
        cache_dir,
        client: reqwest::Client::builder()
            .timeout(DOWNLOAD_TIMEOUT)
            .build()?,
        downloads: Semaphore::new(MAX_CONCURRENT_DOWNLOADS),
        ffmpeg: Semaphore::new(MAX_CONCURRENT_FFMPEG),
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
                        content_type: "audio/wav".to_string(),
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
    fn cache_namespace_must_be_path_safe() {
        for ok in ["botcast", "tenant-1", "a_b"] {
            validate_cache_namespace(ok).unwrap();
        }
        for ng in ["", "..", "a/b", "a\\b", ".", "テナント"] {
            assert!(validate_cache_namespace(ng).is_err(), "{ng:?}");
        }
    }

    #[test]
    fn temp_file_is_removed_on_drop() {
        let path = std::env::temp_dir().join(format!("tempfile_{}", uuid::Uuid::new_v4()));
        std::fs::write(&path, b"x").unwrap();
        drop(TempFile(path.clone()));
        assert!(!path.exists());
        // 無いファイルでも panic しない
        drop(TempFile(path));
    }

    #[test]
    fn canonical_json_sorts_keys() {
        assert_eq!(
            canonical_json(&json!({"b": 1, "a": {"d": [1, {"f": 2, "e": 3}], "c": "x"}})),
            r#"{"a":{"c":"x","d":[1,{"e":3,"f":2}]},"b":1}"#
        );
    }
}

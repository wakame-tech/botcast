//! 声の試聴（`POST /voices/{voice_id}/previews`）。
//!
//! 声のエンジン・caption・params で同じ本文を何回か合成し（テイクごとに seed を変える）、音声を返す。
//! web は聞き比べて選んだテイクを声の参照音声（`ref_audio`）・試聴用の音声（`sample`）にアップロードし、
//! seed をそのテイクの seed にする。テイクはサーバーに保存しない。
//!
//! - 呼べるのは声の持ち主だけ（`X-On-Behalf-Of` で呼び出し元として読み、`owner_id` も確かめる）
//! - テイクは順番に合成する。エンジンの同時実行数（`TTS_ENGINE_*_MAX_CONCURRENCY`）はジョブと共有する
//! - 参照音声にするテイクは長いほうがよい（Irodori-TTS は 30 秒ほどの参照音声で安定する）ので、
//!   既定の本文は数文にし、本文は [`MAX_PREVIEW_TEXT_CHARS`] 文字まで受け付ける

use super::cms_client::CmsClient;
use super::voices::{Engines, RefAudioCache, SpeakerResolver, Voice, VoiceStore, VOICES};
use async_trait::async_trait;
use audio_generator::registry::TtsRegistry;
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeSet, time::Duration};
use tracing::{instrument, Instrument};

/// 本文を省略したときの試聴の本文（絵文字・記号を含まない、20〜30 秒ほどの数文）
pub(crate) const DEFAULT_PREVIEW_TEXT: &str =
    "こんにちは。今日も番組を聞いてくださって、ありがとうございます。\
最近は朝晩がすっかり涼しくなって、温かい飲み物がおいしい季節になりました。\
この番組では、気になったニュースや身の回りの小さな発見を、のんびりとお話ししていきます。\
どうぞ肩の力を抜いて、最後までゆっくりお付き合いください。";

/// 試聴の本文の最大文字数
pub(crate) const MAX_PREVIEW_TEXT_CHARS: usize = 600;
/// テイク数の既定と上限
pub(crate) const DEFAULT_PREVIEW_TAKES: u32 = 3;
pub(crate) const MAX_PREVIEW_TAKES: u32 = 4;
/// 1 回の呼び出し全体のタイムアウト（セルフホストの nginx の `proxy_read_timeout 300s` より短くする）
pub(crate) const PREVIEW_TIMEOUT: Duration = Duration::from_secs(280);

/// `POST /voices/{voice_id}/previews` の body
#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct PreviewRequest {
    #[serde(default)]
    pub text: Option<String>,
    #[serde(default)]
    pub count: Option<u32>,
}

/// 1 つのテイク
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PreviewTake {
    /// このテイクの seed（参照音声にするときに声の seed にする）
    pub seed: i64,
    pub content_type: String,
    /// 音声（Base64）
    pub audio_base64: String,
    /// 合成にかかった時間
    pub elapsed_ms: u64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PreviewResponse {
    pub voice_id: String,
    pub engine: String,
    pub text: String,
    pub takes: Vec<PreviewTake>,
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum PreviewError {
    /// 400
    #[error("{0}")]
    BadRequest(String),
    /// 404（無い・呼び出し元が読めない）
    #[error("voice not found")]
    NotFound,
    /// 403（読めるが持ち主ではない）
    #[error("only the owner of the voice can generate previews")]
    Forbidden,
    /// 503（エンジンが設定されていない・声が使えない）
    #[error("{0}")]
    Unavailable(String),
    /// 504
    #[error("generating previews timed out")]
    Timeout,
    /// 500 / 502（CMS・エンジンの失敗）
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

/// 試聴に使う声の取得元（本番は CMS、テストはフェイク）
#[async_trait]
pub(crate) trait PreviewVoiceStore: VoiceStore {
    /// 呼び出し元 `caller` として声のレコードを読む。無い・読めなければ `None`
    async fn voice_as(&self, voice_id: &str, caller: &str) -> anyhow::Result<Option<Voice>>;
}

#[async_trait]
impl PreviewVoiceStore for CmsClient {
    async fn voice_as(&self, voice_id: &str, caller: &str) -> anyhow::Result<Option<Voice>> {
        match self.get_record_as(VOICES, voice_id, caller).await? {
            Some(record) => Ok(Some(Voice::from_record(&record)?)),
            None => Ok(None),
        }
    }
}

/// パスに入れるレコード ID として妥当か（英数字・`_`・`-`・`:`、64 文字まで）
fn is_valid_record_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 64
        && id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == ':')
}

/// 本文とテイク数を検証して決める
pub(crate) fn validate_request(req: &PreviewRequest) -> Result<(String, u32), PreviewError> {
    let text = match req.text.as_deref().map(str::trim) {
        None | Some("") => DEFAULT_PREVIEW_TEXT.to_string(),
        Some(text) => text.to_string(),
    };
    let chars = text.chars().count();
    if chars > MAX_PREVIEW_TEXT_CHARS {
        return Err(PreviewError::BadRequest(format!(
            "text is too long ({chars} chars; limit {MAX_PREVIEW_TEXT_CHARS})"
        )));
    }
    let count = req.count.unwrap_or(DEFAULT_PREVIEW_TAKES);
    if !(1..=MAX_PREVIEW_TAKES).contains(&count) {
        return Err(PreviewError::BadRequest(format!(
            "count must be between 1 and {MAX_PREVIEW_TAKES}"
        )));
    }
    Ok((text, count))
}

/// テイクの seed（0 〜 2^31-1。エンジンが 32 ビットの符号付き整数で受けても溢れない）
fn random_seed() -> i64 {
    (uuid::Uuid::new_v4().as_u128() as u32 & 0x7fff_ffff) as i64
}

/// 重ならない seed を `count` 個作る
fn distinct_seeds(count: u32, mut next: impl FnMut() -> i64) -> Vec<i64> {
    let mut seen = BTreeSet::new();
    let mut seeds = vec![];
    while seeds.len() < count as usize {
        let seed = next();
        if seen.insert(seed) {
            seeds.push(seed);
        }
    }
    seeds
}

/// 声の試聴を作る
pub(crate) struct VoicePreviewer<'a> {
    pub store: &'a dyn PreviewVoiceStore,
    pub tts: &'a TtsRegistry,
    pub engines: &'a Engines,
    /// TTS の名前空間（CMS のテナント ID）
    pub namespace: &'a str,
    pub ref_audios: &'a RefAudioCache,
}

impl VoicePreviewer<'_> {
    #[instrument(
        name = "voice.preview",
        skip(self, req),
        fields(
            voice.id = %voice_id,
            voice.engine = tracing::field::Empty,
            preview.takes = tracing::field::Empty,
            preview.text_chars = tracing::field::Empty,
            otel.status_code = tracing::field::Empty,
        )
    )]
    pub(crate) async fn generate(
        &self,
        caller: &str,
        voice_id: &str,
        req: &PreviewRequest,
    ) -> Result<PreviewResponse, PreviewError> {
        let result =
            tokio::time::timeout(PREVIEW_TIMEOUT, self.generate_inner(caller, voice_id, req))
                .await
                .unwrap_or(Err(PreviewError::Timeout));
        if let Err(e) = &result {
            let span = tracing::Span::current();
            span.record("otel.status_code", "ERROR");
            tracing::warn!("voice preview failed: {e}");
        }
        result
    }

    async fn generate_inner(
        &self,
        caller: &str,
        voice_id: &str,
        req: &PreviewRequest,
    ) -> Result<PreviewResponse, PreviewError> {
        let span = tracing::Span::current();
        if !is_valid_record_id(voice_id) {
            return Err(PreviewError::BadRequest(format!(
                "{voice_id:?} is not a valid voice id"
            )));
        }
        let (text, count) = validate_request(req)?;
        span.record("preview.takes", count);
        span.record("preview.text_chars", text.chars().count());

        let voice = self
            .store
            .voice_as(voice_id, caller)
            .await?
            .ok_or(PreviewError::NotFound)?;
        // CMS の read: owner に加えて、所有者を確かめる（管理者・サービスでも他人の声は使わない）
        if voice.owner_id.as_deref() != Some(caller) {
            return Err(PreviewError::Forbidden);
        }
        span.record("voice.engine", voice.engine.as_str());
        let engine = voice.engine.clone();

        let cast = Default::default();
        let mut resolver = SpeakerResolver::new(
            self.store,
            &cast,
            self.engines,
            Some(caller),
            self.ref_audios,
        );
        let resolved = resolver
            .resolve_record(voice)
            .await?
            .map_err(PreviewError::Unavailable)?;

        let mut takes = vec![];
        for (i, seed) in distinct_seeds(count, random_seed).into_iter().enumerate() {
            let voice = audio_generator::models::ResolvedVoice {
                seed: Some(seed),
                ..resolved.clone()
            };
            let take_span = tracing::info_span!(
                "voice.preview.take",
                preview.take = i,
                preview.seed = seed,
                preview.elapsed_ms = tracing::field::Empty,
                preview.bytes = tracing::field::Empty,
            );
            let started = std::time::Instant::now();
            let bytes = self
                .tts
                .synthesize_text(self.namespace, &voice, &text)
                .instrument(take_span.clone())
                .await
                .map_err(|e| e.context(format!("take {} of {count}", i + 1)))?;
            let elapsed_ms = started.elapsed().as_millis() as u64;
            take_span.record("preview.elapsed_ms", elapsed_ms);
            take_span.record("preview.bytes", bytes.len());
            takes.push(PreviewTake {
                seed,
                // VOICEVOX 互換・OpenAI 互換（response_format: wav）とも wav が返る
                content_type: "audio/wav".to_string(),
                audio_base64: BASE64.encode(&bytes),
                elapsed_ms,
            });
        }
        Ok(PreviewResponse {
            voice_id: voice_id.to_string(),
            engine,
            text,
            takes,
        })
    }
}

#[cfg(test)]
pub(crate) mod test_support {
    use super::*;
    use crate::usecase::cms_client::FileContent;
    use anyhow::Result;
    use audio_generator::{
        models::RefAudio,
        tts::{SynthesisRequest, TtsEngine},
    };
    use serde_json::json;
    use std::sync::{Arc, Mutex};

    pub(crate) const OWNER: &str = "user:me";

    pub(crate) struct FakeStore {
        pub voices: Vec<Voice>,
    }

    #[async_trait]
    impl VoiceStore for FakeStore {
        async fn find_voices(&self, _slug: &str) -> Result<Vec<Voice>> {
            Ok(vec![])
        }
        async fn fetch_ref_audio(&self, _voice: &Voice) -> Result<FileContent> {
            Ok(FileContent {
                bytes: b"RIFF-take".to_vec(),
                content_type: "audio/wav".to_string(),
            })
        }
    }

    #[async_trait]
    impl PreviewVoiceStore for FakeStore {
        /// CMS の `read: owner` を真似る: 持ち主以外には見えない。ただし `admin` は全部読める
        async fn voice_as(&self, voice_id: &str, caller: &str) -> Result<Option<Voice>> {
            Ok(self
                .voices
                .iter()
                .find(|v| v.id == voice_id)
                .filter(|v| caller == "user:admin" || v.owner_id.as_deref() == Some(caller))
                .cloned())
        }
    }

    /// エンジンが受けた依頼: (seed, caption, voice_id, 参照音声の有無, 本文)
    pub(crate) type Recorded = (Option<i64>, Option<String>, String, bool, String);

    /// 受けた依頼（seed・caption・voice）を記録するエンジン
    #[derive(Debug, Default)]
    pub(crate) struct RecordingEngine {
        pub requests: Mutex<Vec<Recorded>>,
    }

    #[async_trait]
    impl TtsEngine for RecordingEngine {
        async fn synthesize(&self, req: SynthesisRequest<'_>) -> Result<Vec<u8>> {
            self.requests.lock().unwrap().push((
                req.voice.seed,
                req.voice.style.clone(),
                req.voice.voice_id.clone(),
                req.voice.ref_audio.is_some(),
                req.text.to_string(),
            ));
            Ok(b"RIFF-fake".to_vec())
        }
        async fn ensure_voice(&self, _: &str, _: &str, _: &RefAudio) -> Result<()> {
            Ok(())
        }
        async fn health(&self) -> Result<()> {
            Ok(())
        }
    }

    pub(crate) fn voice(id: &str, owner: &str, data: serde_json::Value) -> Voice {
        Voice::from_record(&json!({ "id": id, "owner_id": owner, "version": 1, "data": data }))
            .unwrap()
    }

    pub(crate) fn store() -> FakeStore {
        FakeStore {
            voices: vec![
                voice(
                    "v1",
                    OWNER,
                    json!({ "slug": "alice", "name": "A", "engine": "irodori",
                            "voice_id": "none", "caption": "明るい声", "seed": 1 }),
                ),
                voice(
                    "v2",
                    "user:other",
                    json!({ "slug": "bob", "name": "B", "engine": "irodori", "voice_id": "none" }),
                ),
                voice(
                    "v3",
                    OWNER,
                    json!({ "slug": "carol", "name": "C", "engine": "missing", "voice_id": "1" }),
                ),
                voice(
                    "v4",
                    OWNER,
                    json!({ "slug": "dave", "name": "D", "engine": "irodori", "voice_id": "none",
                            "caption": "低い声",
                            "ref_audio": { "content_type": "audio/wav", "size": 9 } }),
                ),
            ],
        }
    }

    pub(crate) fn engines() -> Engines {
        Engines {
            names: ["irodori"].map(str::to_string).into(),
            openai: ["irodori"].map(str::to_string).into(),
            default: None,
        }
    }

    /// フェイクの声・エンジンで組み立てた試聴の部品
    pub(crate) struct Fixture {
        pub engine: Arc<RecordingEngine>,
        pub tts: TtsRegistry,
        pub store: FakeStore,
        pub engines: Engines,
        pub ref_audios: RefAudioCache,
    }

    impl Fixture {
        pub(crate) fn new() -> Self {
            let engine = Arc::new(RecordingEngine::default());
            Self {
                tts: TtsRegistry::default().with_engine("irodori", engine.clone(), 1),
                engine,
                store: store(),
                engines: engines(),
                ref_audios: RefAudioCache::default(),
            }
        }

        pub(crate) fn previewer(&self) -> VoicePreviewer<'_> {
            VoicePreviewer {
                store: &self.store,
                tts: &self.tts,
                engines: &self.engines,
                namespace: "botcast",
                ref_audios: &self.ref_audios,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::test_support::*;
    use super::*;

    async fn run(
        caller: &str,
        voice_id: &str,
        req: PreviewRequest,
    ) -> (
        Result<PreviewResponse, PreviewError>,
        std::sync::Arc<RecordingEngine>,
    ) {
        let fixture = Fixture::new();
        let res = fixture.previewer().generate(caller, voice_id, &req).await;
        (res, fixture.engine.clone())
    }

    #[tokio::test]
    async fn owner_gets_takes_with_distinct_seeds() {
        let (res, engine) = run(OWNER, "v1", PreviewRequest::default()).await;
        let res = res.unwrap();
        assert_eq!(res.takes.len(), DEFAULT_PREVIEW_TAKES as usize);
        assert_eq!(res.engine, "irodori");
        assert_eq!(res.text, DEFAULT_PREVIEW_TEXT);
        let seeds: BTreeSet<i64> = res.takes.iter().map(|t| t.seed).collect();
        assert_eq!(seeds.len(), res.takes.len(), "seeds must differ per take");
        assert!(seeds.iter().all(|s| (0..=i32::MAX as i64).contains(s)));
        let take = &res.takes[0];
        assert_eq!(take.content_type, "audio/wav");
        assert_eq!(BASE64.decode(&take.audio_base64).unwrap(), b"RIFF-fake");

        // テイクの seed で合成し（声の seed 1 は使わない）、caption と声の設定はそのまま
        let requests = engine.requests.lock().unwrap().clone();
        assert_eq!(requests.len(), 3);
        for ((seed, caption, voice_id, has_ref, text), take) in requests.iter().zip(&res.takes) {
            assert_eq!(*seed, Some(take.seed));
            assert_eq!(caption.as_deref(), Some("明るい声"));
            assert_eq!(voice_id, "none");
            assert!(!has_ref);
            assert_eq!(text, DEFAULT_PREVIEW_TEXT);
        }
    }

    #[tokio::test]
    async fn voice_with_ref_audio_is_previewed_with_ref_audio_and_without_caption() {
        let (res, engine) = run(
            OWNER,
            "v4",
            PreviewRequest {
                text: Some("  こんにちは。  ".to_string()),
                count: Some(1),
            },
        )
        .await;
        assert_eq!(res.unwrap().text, "こんにちは。");
        let requests = engine.requests.lock().unwrap().clone();
        assert_eq!(requests.len(), 1);
        let (_, caption, voice_id, has_ref, _) = &requests[0];
        assert_eq!(caption, &None);
        assert_eq!(voice_id, "dave");
        assert!(has_ref);
    }

    #[tokio::test]
    async fn other_users_cannot_preview() {
        // CMS が隠す（404）
        let (res, engine) = run(OWNER, "v2", PreviewRequest::default()).await;
        assert!(matches!(res, Err(PreviewError::NotFound)), "{res:?}");
        // 読めても持ち主でなければ 403
        let (res2, _) = run("user:admin", "v1", PreviewRequest::default()).await;
        assert!(matches!(res2, Err(PreviewError::Forbidden)), "{res2:?}");
        let (res3, _) = run(OWNER, "missing", PreviewRequest::default()).await;
        assert!(matches!(res3, Err(PreviewError::NotFound)));
        assert!(engine.requests.lock().unwrap().is_empty());
    }

    #[tokio::test]
    async fn count_and_text_bounds() {
        for count in [0, MAX_PREVIEW_TAKES + 1] {
            let (res, _) = run(
                OWNER,
                "v1",
                PreviewRequest {
                    text: None,
                    count: Some(count),
                },
            )
            .await;
            assert!(matches!(res, Err(PreviewError::BadRequest(_))), "{count}");
        }
        let (res, _) = run(
            OWNER,
            "v1",
            PreviewRequest {
                text: None,
                count: Some(MAX_PREVIEW_TAKES),
            },
        )
        .await;
        assert_eq!(res.unwrap().takes.len(), MAX_PREVIEW_TAKES as usize);

        let long = "あ".repeat(MAX_PREVIEW_TEXT_CHARS + 1);
        let (res, _) = run(
            OWNER,
            "v1",
            PreviewRequest {
                text: Some(long),
                count: Some(1),
            },
        )
        .await;
        assert!(matches!(res, Err(PreviewError::BadRequest(_))));
        let (res, _) = run(
            OWNER,
            "v1",
            PreviewRequest {
                text: Some("あ".repeat(MAX_PREVIEW_TEXT_CHARS)),
                count: Some(1),
            },
        )
        .await;
        assert!(res.is_ok());

        let (res, _) = run(OWNER, "v/../x", PreviewRequest::default()).await;
        assert!(matches!(res, Err(PreviewError::BadRequest(_))));
    }

    #[tokio::test]
    async fn unconfigured_engine_is_unavailable() {
        let (res, _) = run(OWNER, "v3", PreviewRequest::default()).await;
        match res {
            Err(PreviewError::Unavailable(msg)) => assert!(msg.contains("not configured"), "{msg}"),
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn distinct_seeds_skip_duplicates() {
        let mut values = [5, 5, 7, 5, 9].into_iter();
        assert_eq!(distinct_seeds(3, || values.next().unwrap()), vec![5, 7, 9]);
    }

    #[test]
    fn default_text_fits_the_limit() {
        assert!(DEFAULT_PREVIEW_TEXT.chars().count() <= MAX_PREVIEW_TEXT_CHARS);
        // 参照音声にするので数文（20〜30 秒ほど）
        assert!(DEFAULT_PREVIEW_TEXT.chars().count() >= 120);
    }
}

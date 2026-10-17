//! 台本の `speaker` を声（[`ResolvedVoice`]）に解決する。
//!
//! 解決の順番: 番組の配役（`podcasts.cast`）の別名 → `voices` の slug → 数字だけなら旧形式
//! （既定のエンジンの VOICEVOX のスタイル ID）。参照音声（`voices.ref_audio`）は CMS から取得してハッシュを付ける。
//! エンジンへの同期は audio_generator が行う。

use super::cms_client::{CmsClient, ListQuery};
use anyhow::{bail, Context};
use async_trait::async_trait;
use audio_generator::{
    config::{EngineKind, TtsConfig},
    models::{RefAudio, ResolvedSection, ResolvedVoice, Section, VoiceParams},
};
use serde::Deserialize;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet, HashMap},
    sync::Arc,
};

pub(crate) const VOICES: &str = "voices";

/// 番組の配役: 台本の `speaker` に書く別名 → voices の slug
pub(crate) type Cast = BTreeMap<String, String>;

/// 番組のレコード全体から配役を取り出す（無ければ空）
pub(crate) fn podcast_cast(podcast: &Value) -> anyhow::Result<Cast> {
    match &podcast["data"]["cast"] {
        Value::Null => Ok(Cast::new()),
        cast => serde_json::from_value(cast.clone()).context("invalid podcast cast"),
    }
}

/// `voices` のレコード
#[derive(Debug, Clone, Deserialize)]
pub(crate) struct Voice {
    /// レコード ID（参照音声の取得に使う）
    #[serde(skip)]
    pub id: String,
    pub slug: String,
    pub name: String,
    pub engine: String,
    #[serde(default)]
    pub voice_id: Option<String>,
    #[serde(default)]
    pub styles: BTreeMap<String, String>,
    /// CMS のファイルのフィールド（`pointer` / `content_type` / `size` / `url`）
    #[serde(default)]
    pub ref_audio: Option<Value>,
    #[serde(default)]
    pub params: VoiceParams,
    #[serde(default)]
    pub seed: Option<i64>,
    #[serde(default)]
    pub caption: Option<String>,
}

impl Voice {
    /// レコード全体（`id` / `data`）から作る
    pub(crate) fn from_record(record: &Value) -> anyhow::Result<Self> {
        let mut voice: Voice =
            serde_json::from_value(record["data"].clone()).context("invalid voice record")?;
        voice.id = record["id"].as_str().unwrap_or_default().to_string();
        Ok(voice)
    }
}

/// 声の取得元（本番は CMS、テストはフェイク）
#[async_trait]
pub(crate) trait VoiceStore: Send + Sync {
    /// slug の声を探す。無ければ `None`
    async fn find_voice(&self, slug: &str) -> anyhow::Result<Option<Voice>>;
    /// 声の参照音声のバイト列を取得する
    async fn fetch_ref_audio(&self, voice: &Voice) -> anyhow::Result<Vec<u8>>;
}

#[async_trait]
impl VoiceStore for CmsClient {
    async fn find_voice(&self, slug: &str) -> anyhow::Result<Option<Voice>> {
        let records = self
            .list_records(
                VOICES,
                &ListQuery {
                    filter: Some(format!("slug:eq:{slug}")),
                    limit: Some(1),
                    ..Default::default()
                },
            )
            .await
            .with_context(|| format!("Failed to fetch voice {slug:?}"))?;
        records.first().map(Voice::from_record).transpose()
    }

    async fn fetch_ref_audio(&self, voice: &Voice) -> anyhow::Result<Vec<u8>> {
        self.get_file(VOICES, &voice.id, "ref_audio")
            .await
            .with_context(|| format!("Failed to fetch reference audio of voice {:?}", voice.slug))
    }
}

/// 解決に使う TTS エンジンの設定
#[derive(Debug, Clone, Default)]
pub(crate) struct Engines {
    pub names: BTreeSet<String>,
    /// 参照音声と caption を使うエンジン（OpenAI 互換）
    pub openai: BTreeSet<String>,
    /// 旧形式の speaker に使うエンジン
    pub default: Option<String>,
}

impl Engines {
    pub(crate) fn from_config(config: &TtsConfig) -> Self {
        Self {
            names: config.engines.iter().map(|e| e.name.clone()).collect(),
            openai: config
                .engines
                .iter()
                .filter(|e| e.kind == EngineKind::OpenAi)
                .map(|e| e.name.clone())
                .collect(),
            default: config.default_engine.clone(),
        }
    }
}

/// voices の slug として探す価値がある文字列か（スキーマの `slug` の pattern と同じ）
fn looks_like_slug(s: &str) -> bool {
    let mut chars = s.chars();
    chars.next().is_some_and(|c| c.is_ascii_lowercase())
        && chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' || c == '-')
}

fn is_legacy_id(s: &str) -> bool {
    !s.is_empty() && s.chars().all(|c| c.is_ascii_digit())
}

fn ref_audio_hash(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}

/// 台本の `speaker` を声に解決する。1 回のジョブで使い、同じ声は 1 度だけ取得する
pub(crate) struct SpeakerResolver<'a> {
    store: &'a dyn VoiceStore,
    cast: &'a Cast,
    engines: &'a Engines,
    /// slug → 声（style は未設定）。見つからなかった slug は `None`
    voices: HashMap<String, Option<Arc<ResolvedVoice>>>,
    /// slug → 声の caption
    captions: HashMap<String, Option<String>>,
}

impl<'a> SpeakerResolver<'a> {
    pub(crate) fn new(store: &'a dyn VoiceStore, cast: &'a Cast, engines: &'a Engines) -> Self {
        Self {
            store,
            cast,
            engines,
            voices: HashMap::new(),
            captions: HashMap::new(),
        }
    }

    /// slug の声を取得し（ジョブ内でキャッシュ）、参照音声があれば取得してハッシュを付ける
    async fn voice(&mut self, slug: &str) -> anyhow::Result<Option<Arc<ResolvedVoice>>> {
        if let Some(voice) = self.voices.get(slug) {
            return Ok(voice.clone());
        }
        let resolved = match self.store.find_voice(slug).await? {
            None => None,
            Some(voice) => {
                // 参照音声と caption は OpenAI 互換のエンジンだけが使う（VOICEVOX 互換では取得しない）
                let openai = self.engines.openai.contains(&voice.engine);
                let ref_audio = match &voice.ref_audio {
                    Some(_) if openai => {
                        let bytes = self.store.fetch_ref_audio(&voice).await?;
                        Some(RefAudio {
                            hash: ref_audio_hash(&bytes),
                            bytes: bytes.into(),
                        })
                    }
                    _ => None,
                };
                // 参照音声を使う声は voice_id を省略でき、slug をエンジンの voice 名にする
                let voice_id = match (&voice.voice_id, &ref_audio) {
                    (Some(id), _) => id.clone(),
                    (None, Some(_)) => voice.slug.clone(),
                    (None, None) => bail!("voice {slug:?} has neither voice_id nor ref_audio"),
                };
                self.captions
                    .insert(slug.to_string(), voice.caption.clone().filter(|_| openai));
                Some(Arc::new(ResolvedVoice {
                    engine: voice.engine,
                    voice_id,
                    params: voice.params,
                    style: None,
                    styles: voice.styles,
                    seed: voice.seed,
                    ref_audio,
                }))
            }
        };
        self.voices.insert(slug.to_string(), resolved.clone());
        Ok(resolved)
    }

    /// 1 つの speaker を解決する。解決できなければ理由を `Err(String)` で返す（CMS のエラーは `anyhow`）
    async fn resolve_speaker(
        &mut self,
        speaker: &str,
        style: Option<String>,
    ) -> anyhow::Result<Result<ResolvedVoice, String>> {
        let (slug, via_cast) = match self.cast.get(speaker) {
            Some(slug) => (slug.clone(), true),
            None => (speaker.to_string(), false),
        };
        if via_cast || looks_like_slug(&slug) {
            if let Some(voice) = self.voice(&slug).await? {
                if !self.engines.names.contains(&voice.engine) {
                    return Ok(Err(format!(
                        "TTS engine {:?} of voice {slug:?} is not configured (configured: {:?})",
                        voice.engine, self.engines.names
                    )));
                }
                let caption = self.captions.get(&slug).cloned().flatten();
                return Ok(Ok(ResolvedVoice {
                    style: style.or(caption),
                    ..(*voice).clone()
                }));
            }
            if via_cast {
                return Ok(Err(format!(
                    "cast maps it to voice {slug:?}, which does not exist"
                )));
            }
        }
        if is_legacy_id(speaker) {
            return Ok(match &self.engines.default {
                Some(engine) => Ok(ResolvedVoice {
                    engine: engine.clone(),
                    voice_id: speaker.to_string(),
                    style,
                    ..Default::default()
                }),
                None => Err(
                    "no TTS engine is configured (set TTS_ENGINES or VOICEVOX_ENDPOINT)"
                        .to_string(),
                ),
            });
        }
        Ok(Err(
            "not an alias in the podcast cast, a voice slug, nor a legacy style ID".to_string(),
        ))
    }

    /// セクションの speaker を解決する。解決できない speaker があれば、全部をまとめたエラーにする
    pub(crate) async fn resolve_sections(
        &mut self,
        sections: Vec<Section>,
    ) -> anyhow::Result<Vec<ResolvedSection>> {
        let mut resolved = Vec::with_capacity(sections.len());
        let mut errors = vec![];
        for (i, section) in sections.into_iter().enumerate() {
            match section {
                Section::SerifSection(serif) => {
                    match self.resolve_speaker(&serif.speaker, serif.style).await? {
                        Ok(voice) => resolved.push(ResolvedSection::Serif {
                            voice,
                            text: serif.text,
                        }),
                        Err(reason) => errors.push(format!(
                            "sections[{i}] speaker {:?}: {reason}",
                            serif.speaker
                        )),
                    }
                }
                Section::AudioSection(audio) => resolved.push(ResolvedSection::Audio(*audio)),
            }
        }
        if !errors.is_empty() {
            bail!("cannot resolve speakers: {}", errors.join("; "));
        }
        Ok(resolved)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use audio_generator::models::{AudioSection, SerifSection};
    use serde_json::json;
    use std::sync::Mutex;

    #[derive(Default)]
    struct FakeStore {
        voices: Vec<Voice>,
        ref_audio: HashMap<String, Vec<u8>>,
        finds: Mutex<Vec<String>>,
        fetches: Mutex<Vec<String>>,
    }

    #[async_trait]
    impl VoiceStore for FakeStore {
        async fn find_voice(&self, slug: &str) -> anyhow::Result<Option<Voice>> {
            self.finds.lock().unwrap().push(slug.to_string());
            Ok(self.voices.iter().find(|v| v.slug == slug).cloned())
        }

        async fn fetch_ref_audio(&self, voice: &Voice) -> anyhow::Result<Vec<u8>> {
            self.fetches.lock().unwrap().push(voice.slug.clone());
            self.ref_audio
                .get(&voice.id)
                .cloned()
                .context("ref audio not found")
        }
    }

    fn voice(id: &str, data: Value) -> Voice {
        Voice::from_record(&json!({ "id": id, "data": data })).unwrap()
    }

    fn serif(speaker: &str, style: Option<&str>) -> Section {
        Section::SerifSection(Box::new(SerifSection {
            speaker: speaker.to_string(),
            text: format!("{speaker} の台詞"),
            style: style.map(str::to_string),
            ..Default::default()
        }))
    }

    fn engines() -> Engines {
        Engines {
            names: ["voicevox", "irodori"].map(str::to_string).into(),
            openai: ["irodori"].map(str::to_string).into(),
            default: Some("voicevox".to_string()),
        }
    }

    fn store() -> FakeStore {
        FakeStore {
            voices: vec![
                voice(
                    "v1",
                    json!({
                        "slug": "zundamon", "name": "ずんだもん", "engine": "voicevox", "voice_id": "3",
                        "styles": { "ささやき": "22" }, "params": { "speed": 1.2 }
                    }),
                ),
                voice(
                    "v2",
                    json!({
                        "slug": "narrator", "name": "語り", "engine": "irodori", "seed": 7,
                        "caption": "落ち着いた声",
                        "ref_audio": { "pointer": "images:/c/v2/ref_audio", "content_type": "audio/wav", "size": 3, "url": "/x" }
                    }),
                ),
                voice(
                    "v3",
                    json!({ "slug": "ghost", "name": "幽霊", "engine": "missing", "voice_id": "1" }),
                ),
            ],
            ref_audio: HashMap::from([("v2".to_string(), b"wav".to_vec())]),
            ..Default::default()
        }
    }

    fn voice_of(section: &ResolvedSection) -> &ResolvedVoice {
        match section {
            ResolvedSection::Serif { voice, .. } => voice,
            ResolvedSection::Audio(_) => panic!("not a serif"),
        }
    }

    #[test]
    fn podcast_cast_is_optional() {
        assert!(podcast_cast(&json!({ "data": { "title": "t" } }))
            .unwrap()
            .is_empty());
        let cast = podcast_cast(&json!({ "data": { "cast": { "ホスト": "zundamon" } } })).unwrap();
        assert_eq!(cast.get("ホスト").map(String::as_str), Some("zundamon"));
        assert!(podcast_cast(&json!({ "data": { "cast": ["x"] } })).is_err());
    }

    #[tokio::test]
    async fn resolves_alias_then_slug_then_legacy_id() {
        let store = store();
        let cast = Cast::from([("ホスト".to_string(), "zundamon".to_string())]);
        let engines = engines();
        let mut resolver = SpeakerResolver::new(&store, &cast, &engines);
        let resolved = resolver
            .resolve_sections(vec![
                serif("ホスト", Some("ささやき")),
                serif("zundamon", None),
                serif("2", Some("元気に")),
                Section::AudioSection(Box::new(AudioSection {
                    url: "https://example.com/a.mp3".to_string(),
                    ..Default::default()
                })),
            ])
            .await
            .unwrap();

        let host = voice_of(&resolved[0]);
        assert_eq!(
            (host.engine.as_str(), host.voice_id.as_str()),
            ("voicevox", "3")
        );
        assert_eq!(host.style.as_deref(), Some("ささやき"));
        assert_eq!(host.styles.get("ささやき").map(String::as_str), Some("22"));
        assert_eq!(host.params.speed, Some(1.2));

        let by_slug = voice_of(&resolved[1]);
        assert_eq!(by_slug.voice_id, "3");
        assert_eq!(by_slug.style, None);

        // 数字だけの speaker は既定のエンジンのスタイル ID（voices は引かない）
        assert_eq!(
            voice_of(&resolved[2]),
            &ResolvedVoice {
                engine: "voicevox".to_string(),
                voice_id: "2".to_string(),
                style: Some("元気に".to_string()),
                ..Default::default()
            }
        );
        assert!(matches!(&resolved[3], ResolvedSection::Audio(a) if a.url.ends_with("a.mp3")));
        // 別名と slug で同じ声は 1 度だけ取得する
        assert_eq!(*store.finds.lock().unwrap(), vec!["zundamon".to_string()]);
    }

    #[tokio::test]
    async fn legacy_sections_without_cast_or_voices_keep_working() {
        let store = FakeStore::default();
        let cast = Cast::new();
        let engines = engines();
        let resolved = SpeakerResolver::new(&store, &cast, &engines)
            .resolve_sections(vec![serif("3", None), serif("1", None)])
            .await
            .unwrap();
        assert_eq!(voice_of(&resolved[0]).voice_id, "3");
        assert_eq!(voice_of(&resolved[1]).engine, "voicevox");
        assert!(store.finds.lock().unwrap().is_empty());
    }

    #[tokio::test]
    async fn ref_audio_is_fetched_once_and_hashed() {
        let store = store();
        let cast = Cast::from([("語り".to_string(), "narrator".to_string())]);
        let engines = engines();
        let resolved = SpeakerResolver::new(&store, &cast, &engines)
            .resolve_sections(vec![
                serif("語り", None),
                serif("narrator", Some("ささやき声で")),
                serif("語り", None),
            ])
            .await
            .unwrap();
        let narrator = voice_of(&resolved[0]);
        // voice_id を省略した参照音声の声は slug を voice 名にする
        assert_eq!(
            (narrator.engine.as_str(), narrator.voice_id.as_str()),
            ("irodori", "narrator")
        );
        assert_eq!(narrator.seed, Some(7));
        let ref_audio = narrator.ref_audio.as_ref().unwrap();
        assert_eq!(&*ref_audio.bytes, b"wav");
        assert_eq!(ref_audio.hash, hex::encode(Sha256::digest(b"wav")));
        // style が無いセリフは声の caption、あればセリフの style
        assert_eq!(narrator.style.as_deref(), Some("落ち着いた声"));
        assert_eq!(
            voice_of(&resolved[1]).style.as_deref(),
            Some("ささやき声で")
        );
        assert_eq!(*store.fetches.lock().unwrap(), vec!["narrator".to_string()]);
    }

    #[tokio::test]
    async fn unresolvable_speakers_fail_with_section_and_reason() {
        let store = store();
        let cast = Cast::from([("ゲスト".to_string(), "nobody".to_string())]);
        let engines = engines();
        let err = SpeakerResolver::new(&store, &cast, &engines)
            .resolve_sections(vec![
                serif("zundamon", None),
                serif("ゲスト", None),
                serif("unknown", None),
                serif("ghost", None),
            ])
            .await
            .unwrap_err()
            .to_string();
        assert!(
            err.contains(r#"sections[1] speaker "ゲスト": cast maps it to voice "nobody""#),
            "{err}"
        );
        assert!(
            err.contains(r#"sections[2] speaker "unknown": not an alias"#),
            "{err}"
        );
        assert!(
            err.contains(r#"sections[3] speaker "ghost": TTS engine "missing""#),
            "{err}"
        );
        assert!(!err.contains("sections[0]"), "{err}");
    }

    #[tokio::test]
    async fn legacy_id_without_engines_fails() {
        let store = FakeStore::default();
        let cast = Cast::new();
        let engines = Engines::default();
        let err = SpeakerResolver::new(&store, &cast, &engines)
            .resolve_sections(vec![serif("3", None)])
            .await
            .unwrap_err()
            .to_string();
        assert!(err.contains("no TTS engine is configured"), "{err}");
    }

    #[tokio::test]
    async fn voice_without_voice_id_and_ref_audio_is_an_error() {
        let store = FakeStore {
            voices: vec![voice(
                "v",
                json!({ "slug": "mute", "name": "m", "engine": "voicevox" }),
            )],
            ..Default::default()
        };
        let cast = Cast::new();
        let engines = engines();
        let err = SpeakerResolver::new(&store, &cast, &engines)
            .resolve_sections(vec![serif("mute", None)])
            .await
            .unwrap_err()
            .to_string();
        assert!(err.contains("neither voice_id nor ref_audio"), "{err}");
    }

    #[tokio::test]
    async fn voicevox_voice_ignores_ref_audio_and_caption() {
        let store = FakeStore {
            voices: vec![voice(
                "v",
                json!({
                    "slug": "zundamon", "name": "ずんだもん", "engine": "voicevox", "voice_id": "3",
                    "caption": "元気な声",
                    "ref_audio": { "pointer": "p", "content_type": "audio/wav", "size": 1, "url": "/x" }
                }),
            )],
            ..Default::default()
        };
        let cast = Cast::new();
        let engines = engines();
        let resolved = SpeakerResolver::new(&store, &cast, &engines)
            .resolve_sections(vec![serif("zundamon", None)])
            .await
            .unwrap();
        let v = voice_of(&resolved[0]);
        assert_eq!((v.style.as_ref(), v.ref_audio.as_ref()), (None, None));
        assert!(store.fetches.lock().unwrap().is_empty());
    }

    #[test]
    fn engines_from_config_marks_openai_engines() {
        let config = TtsConfig::from_lookup(|key| {
            match key {
                "TTS_ENGINES" => Some("voicevox,irodori"),
                "TTS_ENGINE_VOICEVOX_KIND" => Some("voicevox"),
                "TTS_ENGINE_VOICEVOX_ENDPOINT" => Some("http://vv:50021"),
                "TTS_ENGINE_IRODORI_KIND" => Some("openai"),
                "TTS_ENGINE_IRODORI_ENDPOINT" => Some("http://gpu:8088/v1"),
                _ => None,
            }
            .map(str::to_string)
        })
        .unwrap();
        let engines = Engines::from_config(&config);
        assert_eq!(engines.names.len(), 2);
        assert!(engines.openai.contains("irodori") && !engines.openai.contains("voicevox"));
        assert_eq!(engines.default.as_deref(), Some("voicevox"));
    }

    #[test]
    fn slug_detection_matches_schema_pattern() {
        assert!(looks_like_slug("zundamon"));
        assert!(looks_like_slug("a-1_b"));
        assert!(!looks_like_slug("3"));
        assert!(!looks_like_slug("ホスト"));
        assert!(!looks_like_slug("Zundamon"));
        assert!(!looks_like_slug(""));
    }
}

//! 台本の `speaker` を声（[`ResolvedVoice`]）に解決する。
//!
//! 解決の順番: 番組の配役（`podcasts.cast`）の別名 → `voices` の slug → 数字だけなら旧形式
//! （既定のエンジンの VOICEVOX のスタイル ID）。使える声は番組の持ち主（podcast の `owner_id`）の声だけ。
//! 参照音声（`voices.ref_audio`）は CMS から取得してハッシュを付け、声の版が同じならジョブをまたいで使い回す。
//! エンジンへの同期は audio_generator が行う。

use super::cms_client::{CmsClient, FileContent, FileField, ListQuery, MAX_FILE_SIZE};
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
    collections::{BTreeMap, BTreeSet, HashMap, VecDeque},
    sync::{Arc, Mutex},
};

pub(crate) const VOICES: &str = "voices";

/// プロセス内に置く参照音声の数の上限
const REF_AUDIO_CACHE_CAPACITY: usize = 16;

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
    /// レコードの所有者（`sub`）
    #[serde(skip)]
    pub owner_id: Option<String>,
    /// レコードの版（更新ごとに増える。参照音声のキャッシュのキーに使う）
    #[serde(skip)]
    pub version: i64,
    pub slug: String,
    pub name: String,
    pub engine: String,
    #[serde(default)]
    pub voice_id: Option<String>,
    #[serde(default)]
    pub styles: BTreeMap<String, String>,
    #[serde(default)]
    pub ref_audio: Option<FileField>,
    #[serde(default)]
    pub params: VoiceParams,
    #[serde(default)]
    pub seed: Option<i64>,
    #[serde(default)]
    pub caption: Option<String>,
}

impl Voice {
    /// レコード全体（`id` / `owner_id` / `version` / `data`）から作る
    pub(crate) fn from_record(record: &Value) -> anyhow::Result<Self> {
        let mut voice: Voice =
            serde_json::from_value(record["data"].clone()).context("invalid voice record")?;
        voice.id = record["id"].as_str().unwrap_or_default().to_string();
        voice.owner_id = record["owner_id"].as_str().map(str::to_string);
        voice.version = record["version"].as_i64().unwrap_or_default();
        Ok(voice)
    }
}

/// 声の取得元（本番は CMS、テストはフェイク）
#[async_trait]
pub(crate) trait VoiceStore: Send + Sync {
    /// slug の声を最大 2 件探す（重複を検出するため）
    async fn find_voices(&self, slug: &str) -> anyhow::Result<Vec<Voice>>;
    /// 声の参照音声を取得する
    async fn fetch_ref_audio(&self, voice: &Voice) -> anyhow::Result<FileContent>;
}

#[async_trait]
impl VoiceStore for CmsClient {
    async fn find_voices(&self, slug: &str) -> anyhow::Result<Vec<Voice>> {
        // filter に混ぜる値なので、スキーマの slug の形でなければ問い合わせない
        if !looks_like_slug(slug) {
            bail!("{slug:?} is not a valid voice slug");
        }
        let records = self
            .list_records(
                VOICES,
                &ListQuery {
                    filter: Some(format!("slug:eq:{slug}")),
                    limit: Some(2),
                    ..Default::default()
                },
            )
            .await
            .with_context(|| format!("Failed to fetch voice {slug:?}"))?;
        records.iter().map(Voice::from_record).collect()
    }

    async fn fetch_ref_audio(&self, voice: &Voice) -> anyhow::Result<FileContent> {
        let file = self
            .get_file(VOICES, &voice.id, "ref_audio", MAX_FILE_SIZE)
            .await
            .with_context(|| {
                format!("Failed to fetch reference audio of voice {:?}", voice.slug)
            })?;
        ensure_audio(&file.content_type)
            .with_context(|| format!("reference audio of voice {:?}", voice.slug))?;
        Ok(file)
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

/// 参照音声の MIME タイプが音声（`audio/*`）か
fn ensure_audio(content_type: &str) -> anyhow::Result<()> {
    if !content_type.starts_with("audio/") {
        bail!("content type {content_type:?} is not audio");
    }
    Ok(())
}

/// slug の声を探し、番組の持ち主 `owner` の声であることを確かめる。
/// 見つからなければ `Ok(None)`、使えなければ理由を `Err(String)` で返す（CMS のエラーは `anyhow`）
pub(crate) async fn find_owned_voice(
    store: &dyn VoiceStore,
    slug: &str,
    owner: Option<&str>,
) -> anyhow::Result<Result<Option<Voice>, String>> {
    if !looks_like_slug(slug) {
        return Ok(Err(format!("{slug:?} is not a valid voice slug")));
    }
    let mut voices = store.find_voices(slug).await?;
    if voices.len() > 1 {
        return Ok(Err(format!("multiple voices have slug {slug:?}")));
    }
    let Some(voice) = voices.pop() else {
        return Ok(Ok(None));
    };
    let Some(owner) = owner else {
        return Ok(Err(format!(
            "voice {slug:?} cannot be used because the podcast owner is unknown"
        )));
    };
    if voice.owner_id.as_deref() != Some(owner) {
        return Ok(Err(format!(
            "voice {slug:?} belongs to another user (only the podcast owner's voices can be used)"
        )));
    }
    Ok(Ok(Some(voice)))
}

/// 参照音声のキャッシュのキー: 声のレコード ID・版・参照音声の大きさ
type RefAudioKey = (String, i64, u64);

/// 参照音声のプロセス内のキャッシュ。声のレコードの版と参照音声の大きさが同じなら、ジョブをまたいで使い回す
#[derive(Default)]
pub(crate) struct RefAudioCache {
    /// 古い順
    entries: Mutex<VecDeque<(RefAudioKey, RefAudio)>>,
}

impl RefAudioCache {
    fn get(&self, key: &RefAudioKey) -> Option<RefAudio> {
        let entries = self.entries.lock().unwrap();
        entries
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.clone())
    }

    fn insert(&self, key: RefAudioKey, ref_audio: RefAudio) {
        let mut entries = self.entries.lock().unwrap();
        // 同じ声の古い版は捨てる
        entries.retain(|(k, _)| k.0 != key.0);
        entries.push_back((key, ref_audio));
        while entries.len() > REF_AUDIO_CACHE_CAPACITY {
            entries.pop_front();
        }
    }
}

/// 声の解決結果: 見つからなければ `Ok(None)`、使えなければ理由
type Lookup = Result<Option<Arc<ResolvedVoice>>, String>;

/// 台本の `speaker` を声に解決する。1 回のジョブで使い、同じ声は 1 度だけ取得する
pub(crate) struct SpeakerResolver<'a> {
    store: &'a dyn VoiceStore,
    cast: &'a Cast,
    engines: &'a Engines,
    /// 番組の持ち主（`owner_id`）。この人の声だけを使う
    owner: Option<&'a str>,
    ref_audios: &'a RefAudioCache,
    /// slug → 声（style は未設定）
    voices: HashMap<String, Lookup>,
    /// slug → 声の caption
    captions: HashMap<String, Option<String>>,
}

impl<'a> SpeakerResolver<'a> {
    pub(crate) fn new(
        store: &'a dyn VoiceStore,
        cast: &'a Cast,
        engines: &'a Engines,
        owner: Option<&'a str>,
        ref_audios: &'a RefAudioCache,
    ) -> Self {
        Self {
            store,
            cast,
            engines,
            owner,
            ref_audios,
            voices: HashMap::new(),
            captions: HashMap::new(),
        }
    }

    /// 参照音声を取得してハッシュを付ける。声の版と大きさが同じならキャッシュを使う
    async fn ref_audio(
        &self,
        voice: &Voice,
        file: &FileField,
    ) -> anyhow::Result<Result<RefAudio, String>> {
        if ensure_audio(&file.content_type).is_err() {
            return Ok(Err(format!(
                "reference audio of voice {:?} is not audio ({:?})",
                voice.slug, file.content_type
            )));
        }
        if file.size > MAX_FILE_SIZE as u64 {
            return Ok(Err(format!(
                "reference audio of voice {:?} is too large ({} bytes; limit {MAX_FILE_SIZE} bytes)",
                voice.slug, file.size
            )));
        }
        let key = (voice.id.clone(), voice.version, file.size);
        if let Some(ref_audio) = self.ref_audios.get(&key) {
            return Ok(Ok(ref_audio));
        }
        let file = self.store.fetch_ref_audio(voice).await?;
        let ref_audio = RefAudio {
            hash: ref_audio_hash(&file.bytes),
            bytes: file.bytes.into(),
            content_type: file.content_type,
        };
        self.ref_audios.insert(key, ref_audio.clone());
        Ok(Ok(ref_audio))
    }

    /// 声を解決し、参照音声があれば取得してハッシュを付ける
    async fn resolve_voice(
        &mut self,
        voice: Voice,
    ) -> anyhow::Result<Result<ResolvedVoice, String>> {
        // 参照音声と caption は OpenAI 互換のエンジンだけが使う（VOICEVOX 互換では取得しない）
        let openai = self.engines.openai.contains(&voice.engine);
        let ref_audio = match &voice.ref_audio {
            Some(file) if openai => match self.ref_audio(&voice, file).await? {
                Ok(ref_audio) => Some(ref_audio),
                Err(reason) => return Ok(Err(reason)),
            },
            _ => None,
        };
        // 参照音声を使う声は voice_id を省略でき、slug をエンジンの voice 名にする
        let voice_id = match (&voice.voice_id, &ref_audio) {
            (Some(id), _) => id.clone(),
            (None, Some(_)) => voice.slug.clone(),
            (None, None) => bail!(
                "voice {:?} has neither voice_id nor ref_audio: upload a reference audio or set voice_id \
                 (for irodori without a reference audio, use \"none\")",
                voice.slug
            ),
        };
        self.captions
            .insert(voice.slug.clone(), voice.caption.clone().filter(|_| openai));
        Ok(Ok(ResolvedVoice {
            engine: voice.engine,
            voice_id,
            params: voice.params,
            style: None,
            styles: voice.styles,
            seed: voice.seed,
            ref_audio,
        }))
    }

    /// slug の声を取得する（ジョブ内でキャッシュ）
    async fn voice(&mut self, slug: &str) -> anyhow::Result<Lookup> {
        if let Some(voice) = self.voices.get(slug) {
            return Ok(voice.clone());
        }
        let lookup = match find_owned_voice(self.store, slug, self.owner).await? {
            Ok(Some(voice)) => self.resolve_voice(voice).await?.map(|v| Some(Arc::new(v))),
            Ok(None) => Ok(None),
            Err(reason) => Err(reason),
        };
        self.voices.insert(slug.to_string(), lookup.clone());
        Ok(lookup)
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
            match self.voice(&slug).await? {
                Err(reason) => return Ok(Err(reason)),
                Ok(Some(voice)) => {
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
                Ok(None) if via_cast => {
                    return Ok(Err(format!(
                        "cast maps it to voice {slug:?}, which does not exist"
                    )));
                }
                Ok(None) => {}
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
        async fn find_voices(&self, slug: &str) -> anyhow::Result<Vec<Voice>> {
            self.finds.lock().unwrap().push(slug.to_string());
            Ok(self
                .voices
                .iter()
                .filter(|v| v.slug == slug)
                .take(2)
                .cloned()
                .collect())
        }

        async fn fetch_ref_audio(&self, voice: &Voice) -> anyhow::Result<FileContent> {
            self.fetches.lock().unwrap().push(voice.slug.clone());
            let bytes = self
                .ref_audio
                .get(&voice.id)
                .cloned()
                .context("ref audio not found")?;
            Ok(FileContent {
                bytes,
                content_type: "audio/wav".to_string(),
            })
        }
    }

    /// 番組の持ち主
    const OWNER: &str = "user:me";

    /// 番組の持ち主の声
    fn voice(id: &str, data: Value) -> Voice {
        owned_voice(id, OWNER, data)
    }

    fn owned_voice(id: &str, owner: &str, data: Value) -> Voice {
        Voice::from_record(&json!({ "id": id, "owner_id": owner, "version": 1, "data": data }))
            .unwrap()
    }

    /// 番組の持ち主 [`OWNER`] として、ジョブごとのキャッシュで解決する
    async fn resolve(
        store: &FakeStore,
        cast: &Cast,
        engines: &Engines,
        sections: Vec<Section>,
    ) -> anyhow::Result<Vec<ResolvedSection>> {
        SpeakerResolver::new(store, cast, engines, Some(OWNER), &RefAudioCache::default())
            .resolve_sections(sections)
            .await
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
        let resolved = resolve(
            &store,
            &cast,
            &engines,
            vec![
                serif("ホスト", Some("ささやき")),
                serif("zundamon", None),
                serif("2", Some("元気に")),
                Section::AudioSection(Box::new(AudioSection {
                    url: "https://example.com/a.mp3".to_string(),
                    ..Default::default()
                })),
            ],
        )
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
        let resolved = resolve(
            &store,
            &cast,
            &engines,
            vec![serif("3", None), serif("1", None)],
        )
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
        let resolved = resolve(
            &store,
            &cast,
            &engines,
            vec![
                serif("語り", None),
                serif("narrator", Some("ささやき声で")),
                serif("語り", None),
            ],
        )
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
        let err = resolve(
            &store,
            &cast,
            &engines,
            vec![
                serif("zundamon", None),
                serif("ゲスト", None),
                serif("unknown", None),
                serif("ghost", None),
            ],
        )
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
        let err = resolve(&store, &cast, &engines, vec![serif("3", None)])
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
        let err = resolve(&store, &cast, &engines, vec![serif("mute", None)])
            .await
            .unwrap_err()
            .to_string();
        assert!(err.contains("neither voice_id nor ref_audio"), "{err}");
        // どうすれば直るかも書く
        assert!(err.contains("set voice_id"), "{err}");
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
        let resolved = resolve(&store, &cast, &engines, vec![serif("zundamon", None)])
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

    #[tokio::test]
    async fn cast_value_that_is_not_a_slug_fails_without_querying() {
        let store = store();
        let cast = Cast::from([
            ("ホスト".to_string(), "zundamon,slug:eq:x".to_string()),
            ("ゲスト".to_string(), "Narrator".to_string()),
        ]);
        let engines = engines();
        let err = resolve(
            &store,
            &cast,
            &engines,
            vec![serif("ホスト", None), serif("ゲスト", None)],
        )
        .await
        .unwrap_err()
        .to_string();
        assert!(
            err.contains(
                r#"sections[0] speaker "ホスト": "zundamon,slug:eq:x" is not a valid voice slug"#
            ),
            "{err}"
        );
        assert!(
            err.contains(r#"sections[1] speaker "ゲスト": "Narrator" is not a valid voice slug"#),
            "{err}"
        );
        assert!(store.finds.lock().unwrap().is_empty());
    }

    #[tokio::test]
    async fn duplicated_slug_is_an_error() {
        let data = json!({ "slug": "twin", "name": "双子", "engine": "voicevox", "voice_id": "1" });
        let store = FakeStore {
            voices: vec![voice("a", data.clone()), voice("b", data)],
            ..Default::default()
        };
        let cast = Cast::from([("ホスト".to_string(), "twin".to_string())]);
        let engines = engines();
        let err = resolve(
            &store,
            &cast,
            &engines,
            vec![serif("ホスト", None), serif("twin", None)],
        )
        .await
        .unwrap_err()
        .to_string();
        assert!(
            err.contains(r#"sections[0] speaker "ホスト": multiple voices have slug "twin""#),
            "{err}"
        );
        assert!(
            err.contains(r#"sections[1] speaker "twin": multiple voices have slug "twin""#),
            "{err}"
        );
    }

    #[tokio::test]
    async fn voices_of_other_users_cannot_be_used() {
        let store = FakeStore {
            voices: vec![owned_voice(
                "o",
                "user:other",
                json!({ "slug": "stranger", "name": "他人", "engine": "voicevox", "voice_id": "1" }),
            )],
            ..Default::default()
        };
        let cast = Cast::from([("ゲスト".to_string(), "stranger".to_string())]);
        let engines = engines();
        let err = resolve(
            &store,
            &cast,
            &engines,
            vec![serif("ゲスト", None), serif("stranger", None)],
        )
        .await
        .unwrap_err()
        .to_string();
        for i in 0..2 {
            assert!(
                err.contains(&format!(
                    r#"sections[{i}] speaker "{}": voice "stranger" belongs to another user"#,
                    ["ゲスト", "stranger"][i]
                )),
                "{err}"
            );
        }
    }

    #[tokio::test]
    async fn without_podcast_owner_only_legacy_ids_resolve() {
        // 番組が消えたエピソード（持ち主が分からない）は、旧形式の speaker だけを解決する
        let store = store();
        let cast = Cast::new();
        let engines = engines();
        let cache = RefAudioCache::default();
        let mut resolver = SpeakerResolver::new(&store, &cast, &engines, None, &cache);
        let resolved = resolver
            .resolve_sections(vec![serif("3", None)])
            .await
            .unwrap();
        assert_eq!(voice_of(&resolved[0]).voice_id, "3");
        let err = resolver
            .resolve_sections(vec![serif("zundamon", None)])
            .await
            .unwrap_err()
            .to_string();
        assert!(err.contains("podcast owner is unknown"), "{err}");
    }

    #[tokio::test]
    async fn ref_audio_is_reused_across_jobs_while_the_version_is_the_same() {
        let mut store = store();
        let cast = Cast::new();
        let engines = engines();
        let cache = RefAudioCache::default();
        async fn job(
            store: &FakeStore,
            cast: &Cast,
            engines: &Engines,
            cache: &RefAudioCache,
        ) -> Vec<ResolvedSection> {
            SpeakerResolver::new(store, cast, engines, Some(OWNER), cache)
                .resolve_sections(vec![serif("narrator", None)])
                .await
                .unwrap()
        }
        let first = job(&store, &cast, &engines, &cache).await;
        let second = job(&store, &cast, &engines, &cache).await;
        assert_eq!(voice_of(&first[0]), voice_of(&second[0]));
        assert_eq!(*store.fetches.lock().unwrap(), vec!["narrator".to_string()]);
        assert_eq!(
            voice_of(&first[0]).ref_audio.as_ref().unwrap().content_type,
            "audio/wav"
        );

        // 声が更新される（版が上がる）と取り直す
        store.voices[1].version = 2;
        store.ref_audio.insert("v2".to_string(), b"wav2".to_vec());
        let third = job(&store, &cast, &engines, &cache).await;
        let ref_audio = voice_of(&third[0]).ref_audio.as_ref().unwrap();
        assert_eq!(&*ref_audio.bytes, b"wav2");
        assert_eq!(store.fetches.lock().unwrap().len(), 2);
    }

    #[test]
    fn ref_audio_cache_keeps_a_bounded_number_of_entries() {
        let cache = RefAudioCache::default();
        let ref_audio = |hash: &str| RefAudio {
            hash: hash.to_string(),
            bytes: Arc::from(&b""[..]),
            content_type: "audio/wav".to_string(),
        };
        for i in 0..REF_AUDIO_CACHE_CAPACITY + 1 {
            cache.insert((format!("v{i}"), 1, 1), ref_audio(&i.to_string()));
        }
        assert!(cache.get(&("v0".to_string(), 1, 1)).is_none());
        assert!(cache.get(&("v1".to_string(), 1, 1)).is_some());
        // 同じ声の新しい版は古い版を置き換える
        cache.insert(("v1".to_string(), 2, 1), ref_audio("new"));
        assert!(cache.get(&("v1".to_string(), 1, 1)).is_none());
        assert_eq!(cache.get(&("v1".to_string(), 2, 1)).unwrap().hash, "new");
    }

    #[tokio::test]
    async fn ref_audio_must_be_small_audio() {
        let file = |content_type: &str, size: usize| json!({ "pointer": "p", "content_type": content_type, "size": size, "url": "/x" });
        let store = FakeStore {
            voices: vec![
                voice(
                    "a",
                    json!({ "slug": "text", "name": "t", "engine": "irodori", "ref_audio": file("text/plain", 3) }),
                ),
                voice(
                    "b",
                    json!({ "slug": "huge", "name": "h", "engine": "irodori", "ref_audio": file("audio/wav", MAX_FILE_SIZE + 1) }),
                ),
            ],
            ..Default::default()
        };
        let cast = Cast::new();
        let engines = engines();
        let err = resolve(
            &store,
            &cast,
            &engines,
            vec![serif("text", None), serif("huge", None)],
        )
        .await
        .unwrap_err()
        .to_string();
        assert!(
            err.contains(
                r#"sections[0] speaker "text": reference audio of voice "text" is not audio"#
            ),
            "{err}"
        );
        assert!(
            err.contains(
                r#"sections[1] speaker "huge": reference audio of voice "huge" is too large"#
            ),
            "{err}"
        );
        // ダウンロードしない
        assert!(store.fetches.lock().unwrap().is_empty());
        assert!(ensure_audio("audio/mpeg").is_ok());
        assert!(ensure_audio("application/octet-stream").is_err());
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

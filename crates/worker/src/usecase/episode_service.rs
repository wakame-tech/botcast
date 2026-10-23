use super::cms_client::CmsClient;
use super::voices::{
    find_owned_voice, podcast_cast, Engines, RefAudioCache, SpeakerResolver, Voice,
};
use crate::error::Error;
use anyhow::Context;
use audio_generator::{
    generate_audio::{generate_audio, SynthesisResult},
    models::Section,
    registry::TtsRegistry,
    workdir::WorkDir,
};
use std::sync::Arc;
use tracing::instrument;

const EPISODES: &str = "episodes";

/// CMS からエピソードを取得し音声を生成するサービス。
///
/// # 音声生成フロー (`GenerateAudio`)
///
/// ```mermaid
/// sequenceDiagram
///   participant kafru as kafru
///   participant worker as worker
///   participant cms as botcast-cms
///   participant tts as TTS エンジン
///
///   kafru->>worker: execute job (generateAudio)
///   worker->>cms: GET /records/{episodes の collection ID}/{id}
///   cms-->>worker: sections
///   worker->>cms: GET /records/{podcasts}/{podcast_id} (配役 cast)
///   worker->>cms: GET /records/{voices}?filter=slug:eq:... (声。ジョブ内で 1 度だけ)
///   worker->>cms: GET /records/{voices}/{id}/images/ref_audio (参照音声があれば)
///   worker->>tts: 参照音声の同期 (OpenAI 互換。ハッシュが違うときだけ)
///   worker->>tts: 音声合成 (各文。エンジンごとの同時実行数まで並列)
///   tts-->>worker: 音声
///   worker->>worker: 正規化 (44.1kHz・s16・2ch) → 結合 → mp3 / SRT 生成
///   worker->>cms: POST .../images/audio, .../images/srt (Base64)
///   worker->>cms: PUT /records/{episodes の collection ID}/{id} (audio_url, srt_url, duration_sec)
/// ```
#[cfg_attr(doc, aquamarine::aquamarine)]
#[derive(Clone)]
pub(crate) struct EpisodeService {
    cms: Arc<CmsClient>,
    tts: Arc<TtsRegistry>,
    engines: Arc<Engines>,
    /// TTS のキャッシュの名前空間（CMS のテナント ID）
    tenant: String,
    /// 参照音声（ジョブをまたいで使い回す）
    ref_audios: Arc<RefAudioCache>,
}

impl EpisodeService {
    pub(crate) fn new(
        cms: Arc<CmsClient>,
        tts: Arc<TtsRegistry>,
        engines: Arc<Engines>,
        tenant: String,
    ) -> Self {
        Self {
            cms,
            tts,
            engines,
            tenant,
            ref_audios: Default::default(),
        }
    }

    /// エピソードの番組のレコード全体
    pub(crate) async fn podcast_of_episode(
        &self,
        episode_id: &str,
    ) -> anyhow::Result<serde_json::Value> {
        let data = self.cms.get_record_data(EPISODES, episode_id).await?;
        let podcast_id = data["podcast_id"]
            .as_str()
            .context("episode has no podcast_id")?;
        self.cms.get_record("podcasts", podcast_id).await
    }

    /// 番組の配役（別名 → 声。番組の持ち主の声が見つからなければ `None`）
    pub(crate) async fn cast_of_podcast(
        &self,
        podcast: &serde_json::Value,
    ) -> anyhow::Result<Vec<(String, Option<Voice>)>> {
        let owner = podcast["owner_id"].as_str();
        let mut voices = vec![];
        for (alias, slug) in podcast_cast(podcast)? {
            let voice = find_owned_voice(self.cms.as_ref(), &slug, owner).await?;
            voices.push((alias, voice.ok().flatten()));
        }
        Ok(voices)
    }

    #[instrument(skip(self, work_dir), ret)]
    pub(crate) async fn generate_audio(
        &self,
        work_dir: &WorkDir,
        episode_id: &str,
    ) -> anyhow::Result<(), Error> {
        let data = self
            .cms
            .get_record_data(EPISODES, episode_id)
            .await
            .context("Failed to fetch episode")
            .map_err(Error::Other)?;

        let sections: Vec<Section> = serde_json::from_value(data["sections"].clone())
            .context("Failed to parse sections")
            .map_err(Error::Other)?;

        // speaker を 配役の別名 → voices の slug → 旧形式（既定のエンジンのスタイル ID）の順で解決する
        let podcast_id = data["podcast_id"]
            .as_str()
            .context("episode has no podcast_id")
            .map_err(Error::Other)?;
        let podcast = match self.cms.get_record("podcasts", podcast_id).await {
            Ok(podcast) => podcast,
            // 番組が消えたエピソードも、旧形式の speaker なら今までどおり作れるようにする
            Err(e) if e.to_string().contains("CMS returned 404") => {
                tracing::warn!("podcast {podcast_id} not found; resolving speakers without cast");
                serde_json::Value::Null
            }
            Err(e) => {
                return Err(Error::Other(e.context("Failed to fetch podcast")));
            }
        };
        let cast = podcast_cast(&podcast).map_err(Error::Other)?;
        // 声は番組の持ち主のものだけを使う
        let owner = podcast["owner_id"].as_str();
        let sections = SpeakerResolver::new(
            self.cms.as_ref(),
            &cast,
            &self.engines,
            owner,
            &self.ref_audios,
        )
        .resolve_sections(sections)
        .await
        .map_err(Error::Other)?;

        let SynthesisResult {
            out_path,
            srt,
            duration_sec,
        } = generate_audio(work_dir, &self.tts, &self.tenant, sections)
            .await
            .context("Failed to generate audio")
            .map_err(Error::Other)?;

        let audio = tokio::fs::read(&out_path)
            .await
            .context("Failed to read audio file")
            .map_err(Error::Other)?;

        let audio_url = self
            .cms
            .upload_file(EPISODES, episode_id, "audio", &audio, "audio/mpeg")
            .await
            .context("Failed to upload audio")
            .map_err(Error::Other)?;
        let srt_url = self
            .cms
            .upload_file(EPISODES, episode_id, "srt", srt.as_bytes(), "text/plain")
            .await
            .context("Failed to upload srt")
            .map_err(Error::Other)?;

        // アップロードでレコードにファイル情報がマージされるため、取り直してから更新する
        let mut data = self
            .cms
            .get_record_data(EPISODES, episode_id)
            .await
            .context("Failed to refetch episode")
            .map_err(Error::Other)?;
        data["audio_url"] = serde_json::Value::String(audio_url);
        data["srt_url"] = serde_json::Value::String(srt_url);
        data["duration_sec"] =
            serde_json::Value::Number(serde_json::Number::from(duration_sec.round() as i64));

        self.cms
            .update_record_data(EPISODES, episode_id, data)
            .await
            .context("Failed to update episode")
            .map_err(Error::Other)?;

        Ok(())
    }
}

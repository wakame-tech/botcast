use super::cms_client::CmsClient;
use crate::error::Error;
use anyhow::Context;
use audio_generator::{
    generate_audio::{generate_audio, SynthesisResult},
    models::Section,
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
///   participant vv as VoiceVox
///
///   kafru->>worker: execute job (generateAudio)
///   worker->>cms: GET /records/{episodes の collection ID}/{id}
///   cms-->>worker: sections
///   worker->>vv: 音声合成 (各セクション)
///   vv-->>worker: wav
///   worker->>worker: wav 結合 → mp3 / SRT 生成
///   worker->>cms: POST .../images/audio, .../images/srt (Base64)
///   worker->>cms: PUT /records/{episodes の collection ID}/{id} (audio_url, srt_url, duration_sec)
/// ```
#[cfg_attr(doc, aquamarine::aquamarine)]
#[derive(Clone)]
pub(crate) struct EpisodeService {
    cms: Arc<CmsClient>,
}

impl EpisodeService {
    pub(crate) fn new(cms: Arc<CmsClient>) -> Self {
        Self { cms }
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

        let SynthesisResult {
            out_path,
            srt,
            duration_sec,
        } = generate_audio(work_dir, sections)
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

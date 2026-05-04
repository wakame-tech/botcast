use crate::error::Error;
use anyhow::Context;
use audio_generator::{
    generate_audio::{generate_audio, SynthesisResult},
    workdir::WorkDir,
};
use openapi_client::models::Section;
use repos::storage::Storage;
use std::{fs::File, io::Read, sync::Arc};
use tracing::instrument;
use uuid::Uuid;

#[derive(serde::Deserialize)]
struct CmsRecord {
    data: serde_json::Value,
}

#[derive(Clone)]
pub(crate) struct EpisodeService {
    storage: Arc<dyn Storage>,
    cms_url: String,
}

impl EpisodeService {
    pub(crate) fn new(storage: Arc<dyn Storage>) -> Self {
        let cms_url =
            std::env::var("CMS_URL").unwrap_or_else(|_| "http://localhost:3002".to_string());
        Self { storage, cms_url }
    }

    #[instrument(skip(self, work_dir), ret)]
    pub(crate) async fn generate_audio(
        &self,
        work_dir: &WorkDir,
        episode_id: &Uuid,
    ) -> anyhow::Result<(), Error> {
        let client = reqwest::Client::new();
        let episode_id_str = episode_id.hyphenated().to_string();

        let response = client
            .get(format!(
                "{}/records/episodes/{}",
                self.cms_url, episode_id_str
            ))
            .send()
            .await
            .map_err(|e| Error::Other(anyhow::anyhow!("Failed to fetch episode: {}", e)))?;

        if !response.status().is_success() {
            return Err(Error::Other(anyhow::anyhow!(
                "CMS returned {}",
                response.status()
            )));
        }

        let record: CmsRecord = response
            .json()
            .await
            .map_err(|e| Error::Other(anyhow::anyhow!("Failed to parse episode: {}", e)))?;

        let sections: Vec<Section> = serde_json::from_value(record.data["sections"].clone())
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

        let mut file = File::open(&out_path)
            .context("Failed to open audio file")
            .map_err(Error::Other)?;
        let mut audio = vec![];
        file.read_to_end(&mut audio)
            .context("Failed to read audio file")
            .map_err(Error::Other)?;

        let audio_path = format!("episodes/{}.mp3", episode_id_str);
        self.storage
            .upload(&audio_path, &audio, "audio/mp3")
            .await
            .context("Failed to upload audio")
            .map_err(Error::Other)?;

        let srt_path = format!("episodes/{}.srt", episode_id_str);
        self.storage
            .upload(&srt_path, srt.as_bytes(), "text/plain")
            .await
            .context("Failed to upload srt")
            .map_err(Error::Other)?;

        let mut data = record.data;
        data["audio_url"] = serde_json::Value::String(audio_path);
        data["srt_url"] = serde_json::Value::String(srt_path);
        data["duration_sec"] = serde_json::Value::Number(
            serde_json::Number::from(duration_sec.round() as i64),
        );

        let update_response = client
            .put(format!(
                "{}/records/episodes/{}",
                self.cms_url, episode_id_str
            ))
            .json(&serde_json::json!({ "data": data }))
            .send()
            .await
            .map_err(|e| Error::Other(anyhow::anyhow!("Failed to update episode: {}", e)))?;

        if !update_response.status().is_success() {
            return Err(Error::Other(anyhow::anyhow!(
                "CMS update returned {}",
                update_response.status()
            )));
        }

        Ok(())
    }
}

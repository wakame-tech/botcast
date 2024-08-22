use super::{Speaker, Synthesizer};
use serde_json::Value;
use std::path::PathBuf;
use tokio::{fs, io::AsyncWriteExt};

static ORIGIN: &str = "http://localhost:50021";

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub(crate) enum VoiceVoxSpeaker {
    ZundaNormal,
}

impl Speaker for VoiceVoxSpeaker {
    fn id(&self) -> &str {
        match self {
            Self::ZundaNormal => "3",
        }
    }
}

#[derive(Debug)]
pub(crate) struct VoiceVox {
    client: reqwest::Client,
}

impl VoiceVox {
    pub(crate) fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }

    async fn query(&self, text: &str, speaker: &impl Speaker) -> anyhow::Result<Value> {
        let url = format!(
            "{}/audio_query?text={}&speaker={}",
            ORIGIN,
            urlencoding::encode(text),
            speaker.id()
        );
        let res = self.client.post(url).send().await?;
        if res.status() != reqwest::StatusCode::OK {
            anyhow::bail!(
                "Failed to query: {} {}",
                res.status(),
                res.json::<Value>().await?.to_string()
            );
        }
        let res = res.json().await?;
        Ok(res)
    }

    async fn synthesis_(
        &self,
        query: Value,
        speaker: &impl Speaker,
        out: &PathBuf,
    ) -> anyhow::Result<()> {
        let url = format!("{}/synthesis?speaker={}", ORIGIN, speaker.id());
        let res = self
            .client
            .post(url)
            .header("Content-Type", "application/json")
            .json(&query)
            .send()
            .await?;
        if res.status() != reqwest::StatusCode::OK {
            anyhow::bail!(
                "Failed to synthesis: {} {}",
                res.status(),
                res.text().await?
            );
        }
        let res = res.bytes().await?;
        let mut f = fs::File::create(out).await?;
        f.write_all(&res).await?;
        Ok(())
    }
}

impl Synthesizer for VoiceVox {
    async fn synthesis(
        &self,
        text: &str,
        speaker: &impl Speaker,
        out: &PathBuf,
    ) -> anyhow::Result<()> {
        let query = self.query(text, speaker).await?;
        self.synthesis_(query, speaker, out).await
    }
}

#[cfg(test)]
mod tests {
    use crate::synthesizer::voicevox::{VoiceVox, VoiceVoxSpeaker};
    use std::path::PathBuf;
    use tokio::fs;

    #[tokio::test]
    async fn test_synthesis() -> anyhow::Result<()> {
        let voicevox = VoiceVox::new();
        let speaker = VoiceVoxSpeaker::ZundaNormal;
        let query = voicevox.query("こんにちは", &speaker).await?;
        let out = PathBuf::from("test.wav");
        voicevox.synthesis_(query, &speaker, &out).await?;
        fs::remove_file(out).await?;
        Ok(())
    }
}

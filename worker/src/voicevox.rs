use serde_json::Value;
use std::path::PathBuf;
use tokio::{fs, io::AsyncWriteExt};

static ORIGIN: &str = "http://localhost:50021";

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum Speaker {
    ZundaNormal,
}

impl Speaker {
    fn as_str(&self) -> &str {
        match self {
            Self::ZundaNormal => "3",
        }
    }
}

#[derive(Debug)]
struct VoiceVox {
    client: reqwest::Client,
}

impl VoiceVox {
    fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }

    async fn query(&self, text: &str, speaker: &Speaker) -> anyhow::Result<Value> {
        let url = format!(
            "{}/audio_query?text={}&speaker={}",
            ORIGIN,
            urlencoding::encode(text),
            speaker.as_str()
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

    async fn synthesis(
        &self,
        query: Value,
        speaker: &Speaker,
        out: &PathBuf,
    ) -> anyhow::Result<()> {
        let url = format!("{}/synthesis?speaker={}", ORIGIN, speaker.as_str());
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

#[cfg(test)]
mod tests {
    use crate::voicevox::{Speaker, VoiceVox};
    use std::path::PathBuf;
    use tokio::fs;

    #[tokio::test]
    async fn test_synthesis() -> anyhow::Result<()> {
        let voicevox = VoiceVox::new();
        let speaker = Speaker::ZundaNormal;
        let query = voicevox.query("こんにちは", &speaker).await?;
        let out = PathBuf::from("test.wav");
        voicevox.synthesis(query, &speaker, &out).await?;
        fs::remove_file(out).await?;
        Ok(())
    }
}

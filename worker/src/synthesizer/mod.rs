use crate::script::Episode;
use std::{fs::OpenOptions, io::Write, path::PathBuf};
use tokio::process::Command;

pub(crate) mod voicevox;

pub(crate) trait Speaker {
    fn id(&self) -> &str;
}

trait Synthesizer {
    async fn synthesis(
        &self,
        text: &str,
        speaker: &impl Speaker,
        out: &PathBuf,
    ) -> anyhow::Result<()>;
}

async fn concat_wavs(paths: &[PathBuf], text_path: &PathBuf, out: &PathBuf) -> anyhow::Result<()> {
    let text = paths
        .iter()
        .map(|p| format!("file '{}'", p.file_name().unwrap().to_str().unwrap()))
        .collect::<Vec<_>>()
        .join("\n");
    let mut f = OpenOptions::new()
        .write(true)
        .create(true)
        .open(text_path)?;
    f.write_all(text.as_bytes())?;

    let mut cmd = Command::new("ffmpeg");
    cmd.args([
        "-y",
        "-f",
        "concat",
        "-i",
        text_path.display().to_string().as_str(),
        out.display().to_string().as_str(),
    ]);
    // cmd.spawn()?.wait();
    let res = cmd.output().await?;
    if !res.status.success() {
        anyhow::bail!("Failed to concat wavs: {}", String::from_utf8(res.stderr)?);
    }
    for path in paths {
        std::fs::remove_file(path)?;
    }
    std::fs::remove_file(text_path)?;
    Ok(())
}

async fn synthesis_episode<S: Synthesizer>(
    synthesizer: S,
    episode: &Episode,
    speaker: &impl Speaker,
    out: &PathBuf,
) -> anyhow::Result<()> {
    let temp_dir = PathBuf::from("temp");
    if !temp_dir.exists() {
        std::fs::create_dir(&temp_dir)?;
    }
    let mut paths = vec![];
    for (i, serif) in episode.serifs.iter().enumerate() {
        let out = temp_dir.join(format!("{}-{}.wav", episode.id.id.to_raw(), i));
        println!("Synthesis: ({}) {}", out.display(), serif.text);
        synthesizer.synthesis(&serif.text, speaker, &out).await?;
        paths.push(out);
    }
    let text_path = temp_dir.join("text.txt");
    concat_wavs(&paths, &text_path, out).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::{
        scrape::{narou::Narou, tests::read_html, EpisodeScraper},
        synthesizer::{
            synthesis_episode,
            voicevox::{VoiceVox, VoiceVoxSpeaker},
        },
    };
    use std::path::PathBuf;

    #[tokio::test]
    async fn synthesis_episodes() -> anyhow::Result<()> {
        let html = read_html("narou.html")?;
        let episode = Narou.scrape(&html)?;
        let voicevox = VoiceVox::new();
        let speaker = VoiceVoxSpeaker::ZundaNormal;
        let out = PathBuf::from("out.wav");
        synthesis_episode(voicevox, &episode, &speaker, &out).await?;
        Ok(())
    }
}

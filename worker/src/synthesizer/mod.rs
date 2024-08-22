use std::{fs::OpenOptions, io::Write, path::PathBuf};
use tokio::process::Command;

pub(crate) mod voicevox;

pub(crate) trait Speaker {
    fn id(&self) -> &str;
}

pub(crate) trait Synthesizer {
    async fn synthesis(
        &self,
        text: &str,
        speaker: &impl Speaker,
        out: &PathBuf,
    ) -> anyhow::Result<()>;
}

pub(crate) async fn concat_wavs(
    paths: &[PathBuf],
    text_path: &PathBuf,
    out: &PathBuf,
) -> anyhow::Result<()> {
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

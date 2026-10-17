//! ffmpeg による正規化・切り出し・結合と、正規化後の wav の長さ。

use anyhow::{bail, Context, Result};
use std::{
    fs::File,
    path::{Path, PathBuf},
    time::Duration,
};
use tokio::process::Command;
use tracing::instrument;
use wavers::Wav;

/// 各文・各音声をそろえる形式（44.1kHz・s16・2ch の wav）
pub const SAMPLE_RATE: u32 = 44100;
pub const CHANNELS: u16 = 2;

/// 正規化の出力の引数
fn normalized_wav_args(output: &Path) -> Vec<String> {
    [
        "-vn",
        "-map_metadata",
        "-1",
        "-fflags",
        "+bitexact",
        "-ar",
        &SAMPLE_RATE.to_string(),
        "-ac",
        &CHANNELS.to_string(),
        "-c:a",
        "pcm_s16le",
        "-f",
        "wav",
    ]
    .into_iter()
    .map(String::from)
    .chain([output.display().to_string()])
    .collect()
}

async fn run_ffmpeg(args: Vec<String>, what: &str) -> Result<()> {
    let res = Command::new("ffmpeg")
        .args(["-y", "-hide_banner", "-loglevel", "error"])
        .args(&args)
        .output()
        .await
        .with_context(|| format!("failed to run ffmpeg ({what})"))?;
    if !res.status.success() {
        bail!(
            "ffmpeg failed ({what}): {}",
            String::from_utf8_lossy(&res.stderr)
        );
    }
    Ok(())
}

/// 任意の形式の音声を 44.1kHz・s16・2ch の wav にする。
pub async fn normalize(input: &Path, output: &Path) -> Result<()> {
    let mut args = vec!["-i".to_string(), input.display().to_string()];
    args.extend(normalized_wav_args(output));
    run_ffmpeg(args, "normalize").await
}

/// 音声を `from_sec` から `to_sec` まで（どちらも元の音声の位置。無ければ先頭・末尾）切り出し、
/// 正規化した wav にする。
pub async fn slice_and_normalize(
    input: &Path,
    output: &Path,
    from_sec: Option<f64>,
    to_sec: Option<f64>,
) -> Result<()> {
    let mut args = vec!["-i".to_string(), input.display().to_string()];
    // 出力のオプションの -ss / -to はどちらも入力の位置を指す
    if let Some(from_sec) = from_sec {
        args.extend(["-ss".to_string(), from_sec.to_string()]);
    }
    if let Some(to_sec) = to_sec {
        args.extend(["-to".to_string(), to_sec.to_string()]);
    }
    args.extend(normalized_wav_args(output));
    run_ffmpeg(args, "slice").await
}

/// 正規化後（s16）の wav の長さ。
pub fn wav_duration(path: &Path) -> Result<Duration> {
    let file = Box::new(File::open(path).with_context(|| format!("open {}", path.display()))?);
    let wav = Wav::<i16>::new(file).with_context(|| format!("read wav {}", path.display()))?;
    let data_size = wav.header().data().size as f64;
    let sample_rate = wav.sample_rate() as f64;
    let n_channels = wav.n_channels() as f64;
    let bytes_per_sample = (wav.header().fmt_chunk.bits_per_sample / 8) as f64;
    Ok(Duration::from_secs_f64(
        data_size / (sample_rate * n_channels * bytes_per_sample),
    ))
}

/// concat demuxer の一覧の 1 行（パスの `'` をエスケープする）
fn concat_line(path: &Path) -> String {
    format!(
        "file '{}'",
        path.display().to_string().replace('\'', r"'\''")
    )
}

/// 正規化済みの wav を順に結合し、`out_dir/episode.mp3` を作る。
#[instrument(skip(paths))]
pub async fn concat_audios(out_dir: &Path, paths: &[PathBuf]) -> Result<PathBuf> {
    let inputs_path = out_dir.join("inputs.txt");
    let mut lines = vec![];
    for path in paths {
        lines.push(concat_line(&std::path::absolute(path)?));
    }
    tokio::fs::write(&inputs_path, lines.join("\n")).await?;

    let episode_audio_path = out_dir.join("episode.mp3");
    run_ffmpeg(
        [
            "-f",
            "concat",
            "-safe",
            "0",
            "-i",
            &inputs_path.display().to_string(),
            "-vn",
            "-ar",
            &SAMPLE_RATE.to_string(),
            "-ac",
            &CHANNELS.to_string(),
            "-b:a",
            "192k",
            &episode_audio_path.display().to_string(),
        ]
        .into_iter()
        .map(String::from)
        .collect(),
        "concat",
    )
    .await?;
    Ok(episode_audio_path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn concat_line_escapes_quote() {
        assert_eq!(
            concat_line(Path::new("/tmp/it's.wav")),
            r"file '/tmp/it'\''s.wav'"
        );
    }
}

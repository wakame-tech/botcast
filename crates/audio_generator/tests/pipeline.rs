//! 共通の層（正規化・長さと SRT・キャッシュ・同時実行数・参照音声の同期・音声のセクション）のテスト。
//! ffmpeg が無い環境では skip する。

use anyhow::Result;
use async_trait::async_trait;
use audio_generator::{
    ffmpeg::wav_duration,
    generate_audio::generate_audio_in_dir,
    models::{AudioSection, RefAudio, ResolvedSection, ResolvedVoice},
    registry::TtsRegistry,
    tts::{SynthesisRequest, TtsEngine},
};
use axum::{routing::get, Router};
use std::{
    fs::File,
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc, Mutex,
    },
    time::Duration,
};
use wavers::Wav;

/// キャッシュの名前空間（テナント ID 相当）
const NS: &str = "test";

fn has_ffmpeg() -> bool {
    let ok = std::process::Command::new("ffmpeg")
        .arg("-version")
        .output()
        .is_ok_and(|o| o.status.success());
    if !ok {
        eprintln!("ffmpeg is not installed; skipped");
    }
    ok
}

/// テストごとの一時ディレクトリ（終わったら消す）
struct TempDir(PathBuf);

impl TempDir {
    fn new() -> Self {
        let dir =
            std::env::temp_dir().join(format!("audio_generator_test_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        Self(dir)
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// モノラル・s16 の無音でない wav
fn wav_bytes(sample_rate: u32, secs: f64) -> Vec<u8> {
    let samples = (sample_rate as f64 * secs).round() as u32;
    let data_size = samples * 2;
    let mut buf = Vec::with_capacity(44 + data_size as usize);
    buf.extend_from_slice(b"RIFF");
    buf.extend_from_slice(&(36 + data_size).to_le_bytes());
    buf.extend_from_slice(b"WAVEfmt ");
    buf.extend_from_slice(&16u32.to_le_bytes());
    buf.extend_from_slice(&1u16.to_le_bytes()); // PCM
    buf.extend_from_slice(&1u16.to_le_bytes()); // mono
    buf.extend_from_slice(&sample_rate.to_le_bytes());
    buf.extend_from_slice(&(sample_rate * 2).to_le_bytes());
    buf.extend_from_slice(&2u16.to_le_bytes());
    buf.extend_from_slice(&16u16.to_le_bytes());
    buf.extend_from_slice(b"data");
    buf.extend_from_slice(&data_size.to_le_bytes());
    for i in 0..samples {
        let v = ((i as f64 * 0.05).sin() * 8000.0) as i16;
        buf.extend_from_slice(&v.to_le_bytes());
    }
    buf
}

/// 呼ばれた回数と同時に動いた数を数えるフェイクのエンジン
#[derive(Debug)]
struct FakeEngine {
    sample_rate: u32,
    secs: f64,
    delay: Duration,
    calls: AtomicUsize,
    health_calls: AtomicUsize,
    in_flight: AtomicUsize,
    max_in_flight: AtomicUsize,
    texts: Mutex<Vec<String>>,
    ensured: Mutex<Vec<(String, String, usize)>>,
    version: Mutex<String>,
    /// この本文を受けたら失敗する
    fail_on: Mutex<Option<String>>,
}

impl FakeEngine {
    fn new(sample_rate: u32, secs: f64) -> Arc<Self> {
        Arc::new(Self {
            sample_rate,
            secs,
            delay: Duration::from_millis(50),
            calls: AtomicUsize::new(0),
            health_calls: AtomicUsize::new(0),
            in_flight: AtomicUsize::new(0),
            max_in_flight: AtomicUsize::new(0),
            texts: Mutex::default(),
            ensured: Mutex::default(),
            version: Mutex::new("1.0".to_string()),
            fail_on: Mutex::default(),
        })
    }
}

#[async_trait]
impl TtsEngine for FakeEngine {
    async fn synthesize(&self, req: SynthesisRequest<'_>) -> Result<Vec<u8>> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        self.texts.lock().unwrap().push(req.text.to_string());
        let now = self.in_flight.fetch_add(1, Ordering::SeqCst) + 1;
        self.max_in_flight.fetch_max(now, Ordering::SeqCst);
        tokio::time::sleep(self.delay).await;
        self.in_flight.fetch_sub(1, Ordering::SeqCst);
        if self.fail_on.lock().unwrap().as_deref() == Some(req.text) {
            anyhow::bail!("fake failure");
        }
        Ok(wav_bytes(self.sample_rate, self.secs))
    }

    async fn version(&self) -> String {
        self.version.lock().unwrap().clone()
    }

    async fn ensure_voice(
        &self,
        namespace: &str,
        voice_id: &str,
        ref_audio: &RefAudio,
    ) -> Result<()> {
        assert_eq!(namespace, NS);
        self.ensured.lock().unwrap().push((
            voice_id.to_string(),
            ref_audio.hash.clone(),
            ref_audio.bytes.len(),
        ));
        Ok(())
    }

    async fn health(&self) -> Result<()> {
        self.health_calls.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }

    fn prepare_text(&self, text: &str) -> String {
        text.replace('🎉', "").trim().to_string()
    }
}

fn serif(engine: &str, voice_id: &str, text: &str) -> ResolvedSection {
    ResolvedSection::Serif {
        voice: ResolvedVoice {
            engine: engine.to_string(),
            voice_id: voice_id.to_string(),
            ..Default::default()
        },
        text: text.to_string(),
    }
}

fn assert_normalized(path: &Path) {
    let wav = Wav::<i16>::new(Box::new(File::open(path).unwrap())).unwrap();
    assert_eq!(wav.sample_rate(), 44100);
    assert_eq!(wav.n_channels(), 2);
    assert_eq!(wav.header().fmt_chunk.bits_per_sample, 16);
}

#[tokio::test]
async fn normalized_duration_and_srt() {
    if !has_ffmpeg() {
        return;
    }
    let dir = TempDir::new();
    // 24kHz の 0.5 秒（VOICEVOX 相当）と 48kHz の 1 秒（Irodori 相当）
    let voicevox = FakeEngine::new(24000, 0.5);
    let irodori = FakeEngine::new(48000, 1.0);
    let registry = TtsRegistry::default()
        .with_engine("voicevox", voicevox.clone(), 1)
        .with_engine("irodori", irodori.clone(), 1);

    let result = generate_audio_in_dir(
        dir.path(),
        &registry,
        NS,
        vec![
            serif("voicevox", "3", "こんにちは。"),
            serif("irodori", "alice", "やった🎉"),
            // 絵文字だけの文は合成しない
            serif("voicevox", "3", "🎉"),
        ],
    )
    .await
    .unwrap();

    assert!(
        (result.duration_sec - 1.5).abs() < 0.01,
        "{}",
        result.duration_sec
    );
    assert_eq!(
        result.srt.trim().replace("\r\n", "\n"),
        "0\n00:00:00,000 --> 00:00:00,500\nこんにちは。\n\n1\n00:00:00,500 --> 00:00:01,500\nやった🎉"
    );
    assert_eq!(*irodori.texts.lock().unwrap(), vec!["やった"]);
    assert!(result.out_path.ends_with("episode.mp3"));
    assert!(result.out_path.exists());

    let cached: Vec<PathBuf> = std::fs::read_dir(dir.path().join("tts_cache").join(NS))
        .unwrap()
        .map(|e| e.unwrap().path())
        .collect();
    assert_eq!(cached.len(), 2, "{cached:?}");
    for path in &cached {
        assert_eq!(path.extension().unwrap(), "wav");
        assert_normalized(path);
    }
    // 一時ファイルは残さない
    let leftovers: Vec<_> = std::fs::read_dir(result.out_path.parent().unwrap())
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .filter(|n| n.starts_with("raw_") || n.ends_with(".tmp"))
        .collect();
    assert!(leftovers.is_empty(), "{leftovers:?}");
}

#[tokio::test]
async fn cache_is_reused() {
    if !has_ffmpeg() {
        return;
    }
    let dir = TempDir::new();
    let cache = TempDir::new();
    let engine = FakeEngine::new(24000, 0.3);
    let registry = TtsRegistry::default()
        .with_engine("voicevox", engine.clone(), 1)
        .with_cache_dir(cache.path());
    let sections = || {
        vec![
            serif("voicevox", "3", "一つ目。二つ目。"),
            serif("voicevox", "1", "三つ目。"),
        ]
    };

    generate_audio_in_dir(dir.path(), &registry, NS, sections())
        .await
        .unwrap();
    assert_eq!(engine.calls.load(Ordering::SeqCst), 1 + 1);
    assert_eq!(engine.health_calls.load(Ordering::SeqCst), 1);

    // 同じ入力ならエンジンを呼ばない（ヘルスチェックもしない）
    let result = generate_audio_in_dir(dir.path(), &registry, NS, sections())
        .await
        .unwrap();
    assert_eq!(engine.calls.load(Ordering::SeqCst), 2);
    assert_eq!(engine.health_calls.load(Ordering::SeqCst), 1);
    assert!((result.duration_sec - 0.6).abs() < 0.01);

    // 声が変われば合成し直す
    generate_audio_in_dir(
        dir.path(),
        &registry,
        NS,
        vec![serif("voicevox", "2", "三つ目。")],
    )
    .await
    .unwrap();
    assert_eq!(engine.calls.load(Ordering::SeqCst), 3);

    // エンジンの版が変われば合成し直す
    *engine.version.lock().unwrap() = "2.0".to_string();
    generate_audio_in_dir(
        dir.path(),
        &registry,
        NS,
        vec![serif("voicevox", "2", "三つ目。")],
    )
    .await
    .unwrap();
    assert_eq!(engine.calls.load(Ordering::SeqCst), 4);

    // 名前空間（テナント）が違えばキャッシュを共有しない
    generate_audio_in_dir(
        dir.path(),
        &registry,
        "other",
        vec![serif("voicevox", "2", "三つ目。")],
    )
    .await
    .unwrap();
    assert_eq!(engine.calls.load(Ordering::SeqCst), 5);
    assert!(cache.path().join("other").is_dir());
    assert!(cache.path().join(NS).is_dir());

    // パスに使えない名前空間は受け付けない
    let err = generate_audio_in_dir(dir.path(), &registry, "../x", sections())
        .await
        .err()
        .unwrap();
    assert!(err.to_string().contains("namespace"), "{err}");
}

#[tokio::test]
async fn temp_files_are_removed_on_failure() {
    if !has_ffmpeg() {
        return;
    }
    let dir = TempDir::new();
    let cache = TempDir::new();
    let engine = FakeEngine::new(24000, 0.1);
    *engine.fail_on.lock().unwrap() = Some("失敗。".to_string());
    let registry = TtsRegistry::default()
        .with_engine("voicevox", engine.clone(), 4)
        .with_cache_dir(cache.path());
    let mut sections: Vec<ResolvedSection> = (0..8)
        .map(|i| serif("voicevox", &i.to_string(), "成功。"))
        .collect();
    sections.insert(4, serif("voicevox", "x", "失敗。"));

    let err = generate_audio_in_dir(dir.path(), &registry, NS, sections)
        .await
        .err()
        .unwrap();
    assert!(format!("{err:#}").contains("fake failure"), "{err:#}");

    // 実行ごとのディレクトリにもキャッシュにも途中のファイルを残さない
    let mut leftovers = vec![];
    for root in [dir.path().to_path_buf(), cache.path().join(NS)] {
        let mut stack = vec![root];
        while let Some(d) = stack.pop() {
            for e in std::fs::read_dir(&d).unwrap() {
                let path = e.unwrap().path();
                let name = path.file_name().unwrap().to_string_lossy().into_owned();
                if path.is_dir() {
                    stack.push(path);
                } else if name.starts_with("raw_") || name.ends_with(".tmp") {
                    leftovers.push(path);
                }
            }
        }
    }
    assert!(leftovers.is_empty(), "{leftovers:?}");
}

#[tokio::test]
async fn concurrency_is_limited_per_engine() {
    if !has_ffmpeg() {
        return;
    }
    let dir = TempDir::new();
    let fast = FakeEngine::new(24000, 0.1);
    let slow = FakeEngine::new(24000, 0.1);
    let registry = TtsRegistry::default()
        .with_engine("fast", fast.clone(), 2)
        .with_engine("slow", slow.clone(), 1);

    let text = (0..6).map(|i| format!("文{i}。\n")).collect::<String>();
    // 1 文ずつ合成されるよう、セクションを分ける
    let mut sections = vec![];
    for i in 0..6 {
        sections.push(serif("fast", "a", &format!("速い{i}。")));
        sections.push(serif("slow", "b", &format!("遅い{i}。")));
    }
    sections.push(serif("fast", "a", &text));

    let result = generate_audio_in_dir(dir.path(), &registry, NS, sections)
        .await
        .unwrap();
    assert_eq!(fast.calls.load(Ordering::SeqCst), 7);
    assert_eq!(slow.calls.load(Ordering::SeqCst), 6);
    assert_eq!(fast.max_in_flight.load(Ordering::SeqCst), 2);
    assert_eq!(slow.max_in_flight.load(Ordering::SeqCst), 1);
    // 並列でも順番どおりに並ぶ
    let lines: Vec<&str> = result.srt.lines().map(str::trim).collect();
    let subtitles: Vec<&str> = lines
        .iter()
        .copied()
        .filter(|l| l.starts_with("速い") || l.starts_with("遅い"))
        .collect();
    assert_eq!(subtitles[..4], ["速い0。", "遅い0。", "速い1。", "遅い1。"]);
}

#[tokio::test]
async fn ensure_voice_once_per_reference_audio() {
    if !has_ffmpeg() {
        return;
    }
    let dir = TempDir::new();
    let engine = FakeEngine::new(48000, 0.2);
    let registry = TtsRegistry::default().with_engine("irodori", engine.clone(), 1);
    let voice = ResolvedVoice {
        engine: "irodori".to_string(),
        voice_id: "alice".to_string(),
        seed: Some(1),
        ref_audio: Some(RefAudio {
            hash: "hash-1".to_string(),
            bytes: Arc::from(b"RIFF-ref".to_vec()),
            content_type: "audio/wav".to_string(),
        }),
        ..Default::default()
    };
    let sections = vec![
        ResolvedSection::Serif {
            voice: voice.clone(),
            text: "一つ目。".to_string(),
        },
        serif("irodori", "bob", "二つ目。"),
        ResolvedSection::Serif {
            voice: voice.clone(),
            text: "三つ目。".to_string(),
        },
    ];
    generate_audio_in_dir(dir.path(), &registry, NS, sections.clone())
        .await
        .unwrap();
    assert_eq!(
        *engine.ensured.lock().unwrap(),
        vec![("alice".to_string(), "hash-1".to_string(), 8)]
    );

    // すべてキャッシュにあれば同期しない
    generate_audio_in_dir(dir.path(), &registry, NS, sections)
        .await
        .unwrap();
    assert_eq!(engine.ensured.lock().unwrap().len(), 1);
}

#[tokio::test]
async fn unknown_engine_fails() {
    let dir = TempDir::new();
    let registry = TtsRegistry::default().with_engine("voicevox", FakeEngine::new(24000, 0.1), 1);
    let err = generate_audio_in_dir(
        dir.path(),
        &registry,
        NS,
        vec![serif("irodori", "a", "こんにちは。")],
    )
    .await
    .err()
    .unwrap();
    assert!(
        err.to_string().contains("\"irodori\" is not configured"),
        "{err}"
    );
}

#[tokio::test]
async fn audio_section_is_sliced_and_normalized() {
    if !has_ffmpeg() {
        return;
    }
    let bgm = wav_bytes(22050, 1.0);
    let app = Router::new().route("/bgm.wav", get(move || async move { bgm.clone() }));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });

    let dir = TempDir::new();
    let engine = FakeEngine::new(24000, 0.5);
    let registry = TtsRegistry::default().with_engine("voicevox", engine, 1);
    let audio = |from, to| {
        ResolvedSection::Audio(AudioSection {
            url: format!("http://{addr}/bgm.wav"),
            from,
            to,
            ..Default::default()
        })
    };
    let result = generate_audio_in_dir(
        dir.path(),
        &registry,
        NS,
        vec![
            audio(Some(0.2), Some(0.7)),
            serif("voicevox", "3", "こんにちは。"),
            audio(None, None),
        ],
    )
    .await
    .unwrap();

    let sliced = result.out_path.parent().unwrap().join("audio_0.wav");
    assert_normalized(&sliced);
    let sliced_secs = wav_duration(&sliced).unwrap().as_secs_f64();
    assert!((sliced_secs - 0.5).abs() < 0.01, "{sliced_secs}");
    assert!(
        (result.duration_sec - 2.0).abs() < 0.02,
        "{}",
        result.duration_sec
    );
    assert!(result.srt.contains("♪"));
}

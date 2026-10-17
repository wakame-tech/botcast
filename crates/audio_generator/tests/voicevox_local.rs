//! 手元の VOICEVOX で旧形式（`VOICEVOX_ENDPOINT` だけ・`speaker` にスタイル ID）の音声生成を通す。
//!
//! ```sh
//! docker run -p 50021:50021 voicevox/voicevox_engine:cpu-latest
//! VOICEVOX_ENDPOINT=http://localhost:50021 cargo test -p audio_generator --test voicevox_local -- --ignored
//! ```

use audio_generator::{
    generate_audio::generate_audio_in_dir, models::Section, registry::TtsRegistry,
};

#[tokio::test]
#[ignore = "VOICEVOX が要る"]
async fn legacy_sections_with_local_voicevox() {
    let registry = TtsRegistry::from_env().unwrap();
    let sections: Vec<Section> = serde_json::from_value(serde_json::json!([
        {"type": "Serif", "speaker": "3", "text": "こんにちは🎉。今日はいい天気ですね。\nhttps://example.com\nまた来週。"},
        {"type": "Serif", "speaker": "1", "text": "はい、また来週！"}
    ]))
    .unwrap();
    let sections = registry.resolve_legacy_sections(sections).unwrap();

    let dir = std::env::temp_dir().join(format!("voicevox_local_{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&dir).unwrap();
    let result = generate_audio_in_dir(&dir, &registry, "local", sections)
        .await
        .unwrap();
    println!("{}", result.srt);
    println!(
        "duration: {:.2}s, out: {}",
        result.duration_sec,
        result.out_path.display()
    );
    assert!(result.duration_sec > 1.0);
    assert!(result.srt.contains("こんにちは🎉。今日はいい天気ですね。"));
    std::fs::remove_dir_all(&dir).unwrap();
}

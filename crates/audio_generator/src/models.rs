use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, sync::Arc};

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct SerifSection {
    #[serde(rename = "type")]
    pub r#type: SerifType,
    #[serde(rename = "speaker")]
    pub speaker: String,
    #[serde(rename = "text")]
    pub text: String,
}

#[derive(
    Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize, Default,
)]
pub enum SerifType {
    #[default]
    #[serde(rename = "Serif")]
    Serif,
}

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct AudioSection {
    #[serde(rename = "type")]
    pub r#type: AudioType,
    #[serde(rename = "url")]
    pub url: String,
    #[serde(rename = "from", skip_serializing_if = "Option::is_none")]
    pub from: Option<f64>,
    #[serde(rename = "to", skip_serializing_if = "Option::is_none")]
    pub to: Option<f64>,
}

#[derive(
    Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize, Default,
)]
pub enum AudioType {
    #[default]
    #[serde(rename = "Audio")]
    Audio,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Section {
    SerifSection(Box<SerifSection>),
    AudioSection(Box<AudioSection>),
}

impl Default for Section {
    fn default() -> Self {
        Self::SerifSection(Default::default())
    }
}

/// 音質のパラメータ。値が無い項目はエンジンの既定のまま。
///
/// - VOICEVOX 互換: `audio_query` の結果の `speedScale` / `pitchScale` / `intonationScale` /
///   `volumeScale` を上書きし、`extra` をそのまま足す
/// - OpenAI 互換: `speed` だけ使う（それ以外は無視して warn）。`extra` はリクエストの本文にそのまま足す
#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct VoiceParams {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub speed: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pitch: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub intonation: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub volume: Option<f64>,
    /// エンジンごとの素通しの値
    #[serde(default, skip_serializing_if = "serde_json::Map::is_empty")]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

/// 声の複製に使う参照音声。`hash` は内容のハッシュ（同期とキャッシュのキーに使う）。
#[derive(Clone, PartialEq)]
pub struct RefAudio {
    pub hash: String,
    pub bytes: Arc<[u8]>,
}

impl std::fmt::Debug for RefAudio {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RefAudio")
            .field("hash", &self.hash)
            .field("bytes", &format_args!("{} bytes", self.bytes.len()))
            .finish()
    }
}

/// エンジンと声が決まった話者。
#[derive(Clone, Default, Debug, PartialEq)]
pub struct ResolvedVoice {
    /// エンジンの名前（`TTS_ENGINES` の要素）
    pub engine: String,
    /// エンジンの声の ID（VOICEVOX 互換はスタイル ID、OpenAI 互換は voice 名）
    pub voice_id: String,
    pub params: VoiceParams,
    /// 口調・感情の指定（自由文）。VOICEVOX 互換は `styles` で引き、OpenAI 互換は caption に渡す
    pub style: Option<String>,
    /// VOICEVOX 互換で使う、style の名前 → スタイル ID の対応
    pub styles: BTreeMap<String, String>,
    /// 生成の seed（OpenAI 互換の Irodori で使う）
    pub seed: Option<i64>,
    pub ref_audio: Option<RefAudio>,
}

/// 声が解決済みのセクション。[`crate::generate_audio::generate_audio`] の入力。
// 台本のセクションは高々数百なので、大きさの差は気にしない
#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug, PartialEq)]
pub enum ResolvedSection {
    Serif { voice: ResolvedVoice, text: String },
    Audio(AudioSection),
}

/// 旧形式のセクション（`speaker` に VOICEVOX のスタイル ID）を、既定のエンジンの声として解決する。
pub fn resolve_legacy_sections(
    sections: Vec<Section>,
    default_engine: &str,
) -> Vec<ResolvedSection> {
    sections
        .into_iter()
        .map(|section| match section {
            Section::SerifSection(serif) => ResolvedSection::Serif {
                voice: ResolvedVoice {
                    engine: default_engine.to_string(),
                    voice_id: serif.speaker,
                    ..Default::default()
                },
                text: serif.text,
            },
            Section::AudioSection(audio) => ResolvedSection::Audio(*audio),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sections_keep_json_shape() {
        let json = serde_json::json!([
            {"type": "Serif", "speaker": "3", "text": "こんにちは。"},
            {"type": "Audio", "url": "https://example.com/a.mp3", "from": 1.0, "to": 2.0},
            {"type": "Audio", "url": "https://example.com/b.mp3"}
        ]);
        let sections: Vec<Section> = serde_json::from_value(json.clone()).unwrap();
        assert_eq!(serde_json::to_value(&sections).unwrap(), json);
    }

    #[test]
    fn legacy_speaker_is_resolved_to_default_engine() {
        let sections = vec![
            Section::SerifSection(Box::new(SerifSection {
                r#type: SerifType::Serif,
                speaker: "3".to_string(),
                text: "こんにちは。".to_string(),
            })),
            Section::AudioSection(Box::new(AudioSection {
                url: "https://example.com/a.mp3".to_string(),
                ..Default::default()
            })),
        ];
        let resolved = resolve_legacy_sections(sections, "voicevox");
        assert_eq!(
            resolved[0],
            ResolvedSection::Serif {
                voice: ResolvedVoice {
                    engine: "voicevox".to_string(),
                    voice_id: "3".to_string(),
                    ..Default::default()
                },
                text: "こんにちは。".to_string(),
            }
        );
        assert!(
            matches!(&resolved[1], ResolvedSection::Audio(a) if a.url == "https://example.com/a.mp3")
        );
    }
}

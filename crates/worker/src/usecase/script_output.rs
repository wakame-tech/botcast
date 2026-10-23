//! 台本生成の LLM の出力 `{ title?, sections }` のスキーマと、その検証・保存
//!
//! スキーマは `collections.rs` の episodes の定義（`title` / `sections`）から作り、二重に持たない。
//! LLM には structured outputs（`output_config.format`）で形を固定させ、保存は worker が行う。

use super::cms_client::CmsClient;
use super::collections::episodes_schema;
use anyhow::{bail, Context};
use audio_generator::models::Section;
use serde_json::{json, Map, Value};
use std::collections::BTreeSet;
use tracing::field::Empty;

const EPISODES: &str = "episodes";

/// structured outputs の JSON Schema が受け付けないキーワード（値の範囲・長さの制約と、CMS 独自の `x-ref`）。
/// これらは送るスキーマから外し、受け取ったあとに元のスキーマ（[`script_validation_schema`]）で確かめる
const UNSUPPORTED_KEYWORDS: &[&str] = &[
    "minLength",
    "maxLength",
    "minimum",
    "maximum",
    "exclusiveMinimum",
    "exclusiveMaximum",
    "multipleOf",
    "x-ref",
];

/// 台本の出力を検証するスキーマ: episodes のスキーマの `title`（任意）と `sections`（必須）
pub(crate) fn script_validation_schema() -> Value {
    // 参照先の ID は台本の出力に関係しない（`podcast_id` は含めない）
    let episodes = episodes_schema("");
    let props = &episodes["properties"];
    json!({
        "type": "object",
        "properties": {
            "title": props["title"].clone(),
            "sections": props["sections"].clone()
        },
        "required": ["sections"],
        "additionalProperties": false
    })
}

/// LLM に渡す出力のスキーマ（`output_config.format.schema`）。
/// [`script_validation_schema`] から structured outputs が受け付けない書き方を除いたもの
pub(crate) fn script_output_schema() -> Value {
    to_structured_output_schema(script_validation_schema())
}

/// `oneOf` を `anyOf` にし、[`UNSUPPORTED_KEYWORDS`] を取り除く
fn to_structured_output_schema(schema: Value) -> Value {
    match schema {
        Value::Object(obj) => Value::Object(
            obj.into_iter()
                .filter(|(k, _)| !UNSUPPORTED_KEYWORDS.contains(&k.as_str()))
                .map(|(k, v)| {
                    let k = if k == "oneOf" { "anyOf".to_string() } else { k };
                    // properties の下はフィールド名なので、キーワードとして扱わない
                    let v = if k == "properties" {
                        match v {
                            Value::Object(props) => Value::Object(
                                props
                                    .into_iter()
                                    .map(|(name, s)| (name, to_structured_output_schema(s)))
                                    .collect::<Map<_, _>>(),
                            ),
                            other => other,
                        }
                    } else {
                        to_structured_output_schema(v)
                    };
                    (k, v)
                })
                .collect(),
        ),
        Value::Array(items) => {
            Value::Array(items.into_iter().map(to_structured_output_schema).collect())
        }
        other => other,
    }
}

/// 検証済みの台本
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct GeneratedScript {
    /// 空でなければエピソードのタイトルも更新する
    pub title: Option<String>,
    /// `Section` 配列（JSON のまま CMS に保存する）
    pub sections: Value,
    /// span に出す件数（本文は入れない）
    pub stats: ScriptStats,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub(crate) struct ScriptStats {
    pub serif_sections: usize,
    pub audio_sections: usize,
    pub speakers: usize,
}

/// LLM の最終応答の本文を台本として解釈し、episodes のスキーマで検証する
pub(crate) fn parse_script_output(text: &str) -> anyhow::Result<GeneratedScript> {
    let out: Value = serde_json::from_str(text.trim())
        .context("LLM output is not JSON")?;
    let validator = jsonschema::validator_for(&script_validation_schema())
        .map_err(|e| anyhow::anyhow!("invalid script output schema: {e}"))?;
    let errors: Vec<String> = validator
        .iter_errors(&out)
        .map(|e| format!("{} ({})", e, e.instance_path))
        .collect();
    if !errors.is_empty() {
        bail!(
            "LLM output does not match the script schema: {}",
            errors.join("; ")
        );
    }
    let sections = out["sections"].clone();
    let parsed: Vec<Section> =
        serde_json::from_value(sections.clone()).context("invalid sections in LLM output")?;
    if parsed.is_empty() {
        bail!("LLM output has no sections");
    }
    let mut stats = ScriptStats::default();
    let mut speakers = BTreeSet::new();
    for section in &parsed {
        match section {
            Section::SerifSection(s) => {
                stats.serif_sections += 1;
                speakers.insert(s.speaker.as_str());
            }
            Section::AudioSection(_) => stats.audio_sections += 1,
        }
    }
    stats.speakers = speakers.len();
    let title = out["title"]
        .as_str()
        .map(str::trim)
        .filter(|t| !t.is_empty())
        .map(str::to_string);
    Ok(GeneratedScript {
        title,
        sections,
        stats,
    })
}

/// エピソードの `data` に台本を書き込む（`sections` を置き換え、`title` があれば更新する）
pub(crate) fn apply_script(data: &mut Value, script: &GeneratedScript) -> anyhow::Result<()> {
    let obj = data.as_object_mut().context("episode data is not an object")?;
    obj.insert("sections".to_string(), script.sections.clone());
    if let Some(title) = &script.title {
        obj.insert("title".to_string(), Value::String(title.clone()));
    }
    Ok(())
}

/// 台本をエピソードに保存する。span には件数だけを入れ、台本の本文は入れない
pub(crate) async fn save_script(
    cms: &CmsClient,
    episode_id: &str,
    script: &GeneratedScript,
) -> anyhow::Result<()> {
    let span = tracing::info_span!(
        "script.save",
        script.serif_sections = script.stats.serif_sections,
        script.audio_sections = script.stats.audio_sections,
        script.speakers = script.stats.speakers,
        script.title_updated = script.title.is_some(),
        otel.status_code = Empty,
        otel.status_message = Empty,
    );
    let result = async {
        let mut data = cms
            .get_record_data(EPISODES, episode_id)
            .await
            .context("Failed to fetch episode")?;
        apply_script(&mut data, script)?;
        cms.update_record_data(EPISODES, episode_id, data)
            .await
            .context("Failed to save script")
    }
    .await;
    if let Err(e) = &result {
        crate::trace_context::record_error(&span, &format!("{e:#}"));
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn walk(v: &Value, f: &mut impl FnMut(&str)) {
        match v {
            Value::Object(o) => {
                for (k, v) in o {
                    f(k);
                    walk(v, f);
                }
            }
            Value::Array(a) => a.iter().for_each(|v| walk(v, f)),
            _ => {}
        }
    }

    #[test]
    fn output_schema_is_derived_from_episodes_and_has_only_supported_keywords() {
        let schema = script_output_schema();
        let sections = &schema["properties"]["sections"];
        // episodes の Serif / Audio がそのまま入る（oneOf は anyOf に）
        assert_eq!(sections["items"]["anyOf"][0]["properties"]["type"]["const"], "Serif");
        assert_eq!(sections["items"]["anyOf"][1]["properties"]["type"]["const"], "Audio");
        assert!(schema["properties"]["title"].is_object());
        assert_eq!(schema["required"], json!(["sections"]));
        let mut keys = vec![];
        walk(&schema, &mut |k| keys.push(k.to_string()));
        for k in UNSUPPORTED_KEYWORDS.iter().chain(&["oneOf"]) {
            assert!(!keys.iter().any(|x| x == k), "{k} が残っている: {schema}");
        }
        // structured outputs は全ての object に additionalProperties: false を要求する
        fn check_objects(v: &Value) {
            if let Value::Object(o) = v {
                if o.get("type") == Some(&json!("object")) {
                    assert_eq!(o.get("additionalProperties"), Some(&json!(false)), "{v}");
                }
                o.values().for_each(check_objects);
            } else if let Value::Array(a) = v {
                a.iter().for_each(check_objects);
            }
        }
        check_objects(&schema);
    }

    #[test]
    fn output_schema_keeps_field_named_like_keyword() {
        // properties の下のフィールド名はキーワードとして扱わない
        let s = to_structured_output_schema(json!({
            "type": "object",
            "properties": { "maxLength": { "type": "string", "maxLength": 3 } }
        }));
        assert_eq!(s["properties"]["maxLength"], json!({ "type": "string" }));
    }

    #[test]
    fn parses_valid_output() {
        let script = parse_script_output(
            r#"{"title":" 第1回 ","sections":[
                {"type":"Serif","speaker":"ホスト","text":"こんにちは","style":"楽しそうに"},
                {"type":"Serif","speaker":"ゲスト","text":"どうも"},
                {"type":"Audio","url":"https://example.com/a.mp3","from":0,"to":1.5},
                {"type":"Serif","speaker":"ホスト","text":"では"}
            ]}"#,
        )
        .unwrap();
        assert_eq!(script.title.as_deref(), Some("第1回"));
        assert_eq!(
            script.stats,
            ScriptStats {
                serif_sections: 3,
                audio_sections: 1,
                speakers: 2
            }
        );
        assert_eq!(script.sections[0]["style"], "楽しそうに");
    }

    #[test]
    fn title_is_optional() {
        let script =
            parse_script_output(r#"{"sections":[{"type":"Serif","speaker":"a","text":"b"}]}"#)
                .unwrap();
        assert_eq!(script.title, None);
        let script = parse_script_output(
            r#"{"title":"","sections":[{"type":"Serif","speaker":"a","text":"b"}]}"#,
        )
        .unwrap();
        assert_eq!(script.title, None);
    }

    #[test]
    fn rejects_schema_violations() {
        for (text, want) in [
            ("ここに台本を保存しました", "is not JSON"),
            (r#"{"title":"t"}"#, "does not match"),
            (r#"{"sections":[]}"#, "no sections"),
            (
                r#"{"sections":[{"type":"Serif","speaker":"a"}]}"#,
                "does not match",
            ),
            (
                r#"{"sections":[{"type":"Serif","speaker":"a","text":"b","extra":1}]}"#,
                "does not match",
            ),
            (
                r#"{"sections":[{"type":"Video","url":"x"}]}"#,
                "does not match",
            ),
            (r#"{"sections":[],"podcast_id":"p"}"#, "does not match"),
        ] {
            let err = format!("{:#}", parse_script_output(text).unwrap_err());
            assert!(err.contains(want), "{text}: {err}");
        }
        // 送るスキーマから外した制約（style の長さ）も、受け取ったあとに確かめる
        let long = "あ".repeat(300);
        let text = format!(
            r#"{{"sections":[{{"type":"Serif","speaker":"a","text":"b","style":"{long}"}}]}}"#
        );
        let err = format!("{:#}", parse_script_output(&text).unwrap_err());
        assert!(err.contains("does not match"), "{err}");
    }

    #[test]
    fn apply_script_replaces_sections_and_keeps_other_fields() {
        let mut data = json!({
            "title": "旧", "podcast_id": "p", "audio_url": "a.mp3",
            "sections": [{"type":"Serif","speaker":"x","text":"古い"}]
        });
        let script =
            parse_script_output(r#"{"sections":[{"type":"Serif","speaker":"a","text":"新しい"}]}"#)
                .unwrap();
        apply_script(&mut data, &script).unwrap();
        assert_eq!(data["title"], "旧");
        assert_eq!(data["podcast_id"], "p");
        assert_eq!(data["sections"][0]["text"], "新しい");
        // 保存する内容は episodes のスキーマを満たす
        assert!(jsonschema::is_valid(&episodes_schema("podcasts"), &data));

        let script = parse_script_output(
            r#"{"title":"新","sections":[{"type":"Serif","speaker":"a","text":"b"}]}"#,
        )
        .unwrap();
        apply_script(&mut data, &script).unwrap();
        assert_eq!(data["title"], "新");
    }
}

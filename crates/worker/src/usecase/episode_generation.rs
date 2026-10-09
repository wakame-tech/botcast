use anyhow::{bail, Context};
use audio_generator::models::Section;
use serde_json::Value;

/// レコード全体から、スクリプトに渡す形 (`data` に `id` を足したもの) を作る
pub(crate) fn record_view(record: &Value) -> Value {
    let mut view = record["data"].clone();
    if let Some(obj) = view.as_object_mut() {
        obj.insert("id".to_string(), record["id"].clone());
    }
    view
}

/// スクリプトのコードの先頭に `context` の定義を連結する
pub(crate) fn build_script_code(context: &Value, template: &str) -> String {
    format!("const context = {context};\n{template}")
}

/// スクリプトが生成したエピソード
#[derive(Debug, Clone)]
pub(crate) struct GeneratedEpisode {
    pub title: String,
    /// 検証済みの `Section` 配列 (JSON のまま CMS に保存する)
    pub sections: Value,
}

/// スクリプトの標準出力 `{ title?, sections }` を解釈する
pub(crate) fn parse_script_output(
    stdout: &str,
    error: &str,
    default_title: &str,
) -> anyhow::Result<GeneratedEpisode> {
    if !error.trim().is_empty() {
        bail!("script failed: {}", error.trim());
    }
    let out: Value = serde_json::from_str(stdout.trim())
        .with_context(|| format!("script output is not JSON: {}", stdout.trim()))?;
    let sections = out["sections"].clone();
    let parsed: Vec<Section> =
        serde_json::from_value(sections.clone()).context("invalid sections in script output")?;
    if parsed.is_empty() {
        bail!("script output has no sections");
    }
    let title = out["title"]
        .as_str()
        .filter(|t| !t.is_empty())
        .unwrap_or(default_title)
        .to_string();
    Ok(GeneratedEpisode { title, sections })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn record_view_merges_id_into_data() {
        let record = json!({ "id": "r1", "data": { "title": "t" }, "owner_id": "user:x" });
        assert_eq!(record_view(&record), json!({ "id": "r1", "title": "t" }));
    }

    #[test]
    fn build_script_code_prepends_context() {
        let code = build_script_code(&json!({ "mails": [] }), "console.log(1);");
        assert_eq!(code, "const context = {\"mails\":[]};\nconsole.log(1);");
    }

    #[test]
    fn parse_script_output_with_title() {
        let stdout = r#"{"title":"T","sections":[{"type":"Serif","speaker":"3","text":"a"}]}"#;
        let ep = parse_script_output(&format!("{stdout}\n"), "", "第1回").unwrap();
        assert_eq!(ep.title, "T");
        assert_eq!(ep.sections.as_array().unwrap().len(), 1);
    }

    #[test]
    fn parse_script_output_uses_default_title() {
        let stdout = r#"{"sections":[{"type":"Serif","speaker":"3","text":"a"}]}"#;
        assert_eq!(parse_script_output(stdout, "", "第2回").unwrap().title, "第2回");
    }

    #[test]
    fn parse_script_output_rejects_error_output() {
        assert!(parse_script_output("{}", "ReferenceError: x", "第1回").is_err());
    }

    #[test]
    fn parse_script_output_rejects_invalid_json() {
        assert!(parse_script_output("not json", "", "第1回").is_err());
    }

    #[test]
    fn parse_script_output_rejects_empty_sections() {
        assert!(parse_script_output(r#"{"sections":[]}"#, "", "第1回").is_err());
    }

    #[test]
    fn parse_script_output_rejects_unknown_section() {
        let stdout = r#"{"sections":[{"type":"Unknown"}]}"#;
        assert!(parse_script_output(stdout, "", "第1回").is_err());
    }
}

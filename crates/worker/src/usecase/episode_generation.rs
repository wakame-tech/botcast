use super::cms_client::{CmsClient, ListQuery};
use anyhow::{bail, Context};
use audio_generator::models::Section;
use serde_json::Value;
use std::sync::Arc;
use tracing::instrument;

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

/// 番組の引数の値 `arguments` をスクリプトの JSON Schema `schema` で検証する。
///
/// スキーマが無い (`null`) か空オブジェクト `{}` のときは常に通す。
pub(crate) fn validate_arguments(schema: &Value, arguments: &Value) -> anyhow::Result<()> {
    if schema.is_null() || schema.as_object().is_some_and(|o| o.is_empty()) {
        return Ok(());
    }
    let validator = jsonschema::validator_for(schema)
        .map_err(|e| anyhow::anyhow!("invalid arguments schema: {e}"))?;
    let errors: Vec<String> = validator
        .iter_errors(arguments)
        .map(|e| format!("{} ({})", e, e.instance_path))
        .collect();
    if !errors.is_empty() {
        bail!("arguments do not match schema: {}", errors.join("; "));
    }
    Ok(())
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

/// 番組のスクリプトを実行して新しいエピソードを作るサービス。
///
/// 前回エピソード (その番組で最新) 宛てのお便りを `context.mails`、
/// 番組の `schedule.arguments` (スクリプトの `arguments` スキーマで検証済み) を
/// `context.arguments` としてスクリプトに渡し、
/// スクリプトの出力 `{ title?, sections }` からエピソードを作成する。
#[derive(Clone)]
pub(crate) struct EpisodeGenerationService {
    cms: Arc<CmsClient>,
}

impl EpisodeGenerationService {
    pub(crate) fn new(cms: Arc<CmsClient>) -> Self {
        Self { cms }
    }

    /// エピソードを作成し、その ID を返す
    #[instrument(skip(self), ret)]
    pub(crate) async fn generate(&self, podcast_id: &str) -> anyhow::Result<String> {
        let podcast = self.cms.get_record("podcasts", podcast_id).await?;
        let script_id = podcast["data"]["schedule"]["script_id"]
            .as_str()
            .context("podcast has no schedule.script_id")?;
        let script = self.cms.get_record("scripts", script_id).await?;
        let template = script["data"]["template"]
            .as_str()
            .context("script has no template")?;
        // スクリプトの `arguments` は JSON Schema、値は番組の `schedule.arguments`
        let arguments = match &podcast["data"]["schedule"]["arguments"] {
            Value::Null => serde_json::json!({}),
            v => v.clone(),
        };
        validate_arguments(&script["data"]["arguments"], &arguments)?;

        // 件数 (タイトルの番号) と前回エピソードを同じ一覧から求める
        let episodes = self
            .cms
            .list_records(
                "episodes",
                &ListQuery {
                    filter: Some(format!("podcast_id:eq:{podcast_id}")),
                    sort: Some("created_at".to_string()),
                    order_desc: true,
                    ..Default::default()
                },
            )
            .await?;
        let previous = episodes.first();
        let mails = match previous.and_then(|e| e["id"].as_str()) {
            Some(episode_id) => {
                self.cms
                    .list_records(
                        "mails",
                        &ListQuery {
                            filter: Some(format!("episode_id:eq:{episode_id}")),
                            ..Default::default()
                        },
                    )
                    .await?
            }
            None => vec![],
        };

        let context = serde_json::json!({
            "podcast": record_view(&podcast),
            "previous_episode": previous.map(record_view),
            "mails": mails.iter().map(record_view).collect::<Vec<_>>(),
            "arguments": arguments,
        });
        let output = self
            .cms
            .execute_script(&build_script_code(&context, template))
            .await?;
        let episode = parse_script_output(
            &output.stdout,
            &output.error,
            &format!("第{}回", episodes.len() + 1),
        )?;

        self.cms
            .create_record(
                "episodes",
                serde_json::json!({
                    "podcast_id": podcast_id,
                    "title": episode.title,
                    "description": "",
                    "sections": episode.sections,
                    "user_id": podcast["data"]["user_id"],
                }),
            )
            .await
    }
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

    #[test]
    fn validate_arguments_accepts_any_when_schema_is_empty() {
        assert!(validate_arguments(&json!({}), &json!({ "x": 1 })).is_ok());
        assert!(validate_arguments(&Value::Null, &json!({ "x": 1 })).is_ok());
    }

    #[test]
    fn validate_arguments_accepts_valid_values() {
        let schema = json!({
            "type": "object",
            "properties": { "speaker": { "type": "string" } },
            "required": ["speaker"],
        });
        assert!(validate_arguments(&schema, &json!({ "speaker": "3" })).is_ok());
    }

    #[test]
    fn validate_arguments_rejects_missing_required() {
        let schema = json!({ "type": "object", "required": ["speaker"] });
        assert!(validate_arguments(&schema, &json!({})).is_err());
    }

    #[test]
    fn validate_arguments_rejects_wrong_type() {
        let schema = json!({
            "type": "object",
            "properties": { "count": { "type": "integer" } },
        });
        assert!(validate_arguments(&schema, &json!({ "count": "a" })).is_err());
    }

    #[test]
    fn validate_arguments_rejects_invalid_schema() {
        assert!(validate_arguments(&json!({ "type": 1 }), &json!({})).is_err());
    }
}

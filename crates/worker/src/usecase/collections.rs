//! botcast が CMS に置くコレクションのスキーマと、その作成・更新 (seed)
//!
//! スキーマはここが唯一の情報源で、worker の起動時に CMS へ反映する。
//! `x-ref` の値は参照先コレクションの ID（環境ごとに異なる）なので、参照先から順に ensure する。

use super::cms_client::CmsClient;
use serde_json::{json, Value};

/// 全コレクション共通: 認証済みなら誰でも読め、書けるのは所有者（とサービス）
fn access() -> Value {
    json!({ "read": "public", "write": "owner" })
}

pub(crate) fn scripts_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "title": { "type": "string" },
            "description": { "type": "string" },
            "template": { "type": "string" },
            // スクリプト引数の JSON Schema
            "arguments": { "type": "object" }
        },
        "required": ["title", "template"],
        "additionalProperties": false
    })
}

pub(crate) fn podcasts_schema(scripts_id: &str) -> Value {
    json!({
        "type": "object",
        "properties": {
            "title": { "type": "string" },
            "icon": { "type": "string" },
            "description": { "type": "string" },
            "schedule": {
                "type": "object",
                "properties": {
                    "cron": { "type": "string" },
                    "script_id": { "type": "string", "x-ref": scripts_id },
                    "enabled": { "type": "boolean" },
                    // 値はスクリプトの `arguments` (JSON Schema) に従う
                    "arguments": { "type": "object" }
                },
                "required": ["cron", "script_id", "enabled"],
                "additionalProperties": false
            }
        },
        "required": ["title", "icon"],
        "additionalProperties": false
    })
}

pub(crate) fn episodes_schema(podcasts_id: &str) -> Value {
    json!({
        "type": "object",
        "properties": {
            "title": { "type": "string" },
            "podcast_id": { "type": "string", "x-ref": podcasts_id },
            "description": { "type": "string" },
            "sections": {
                "type": "array",
                "items": {
                    "oneOf": [
                        {
                            "type": "object",
                            "properties": {
                                "type": { "const": "Serif" },
                                "speaker": { "type": "string" },
                                "text": { "type": "string" }
                            },
                            "required": ["type", "speaker", "text"],
                            "additionalProperties": false
                        },
                        {
                            "type": "object",
                            "properties": {
                                "type": { "const": "Audio" },
                                "url": { "type": "string" },
                                "from": { "type": "number" },
                                "to": { "type": "number" }
                            },
                            "required": ["type", "url"],
                            "additionalProperties": false
                        }
                    ]
                }
            },
            "audio_url": { "type": "string" },
            "srt_url": { "type": "string" },
            "duration_sec": { "type": "integer" },
            // CMS がファイルのアップロード (`/images/{audio,srt}`) でここに書き込む
            // (`pointer` / `content_type` / `size` / `url`)。無いとアップロードが 400 になる
            "audio": { "type": "object" },
            "srt": { "type": "object" }
        },
        "required": ["title", "podcast_id"],
        "additionalProperties": false
    })
}

pub(crate) fn mails_schema(episodes_id: &str) -> Value {
    json!({
        "type": "object",
        "properties": {
            "episode_id": { "type": "string", "x-ref": episodes_id },
            "radio_name": { "type": "string" },
            "body": { "type": "string" }
        },
        "required": ["episode_id", "radio_name", "body"],
        "additionalProperties": false
    })
}

/// 参照先から順に作成・更新し、得た ID を参照元の `x-ref` に埋め込む
pub(crate) async fn seed(cms: &CmsClient) -> anyhow::Result<()> {
    let scripts = cms.ensure_collection("scripts", scripts_schema(), access()).await?;
    let podcasts = cms
        .ensure_collection("podcasts", podcasts_schema(&scripts), access())
        .await?;
    let episodes = cms
        .ensure_collection("episodes", episodes_schema(&podcasts), access())
        .await?;
    cms.ensure_collection("mails", mails_schema(&episodes), access())
        .await?;
    Ok(())
}

/// 環境変数の CMS 接続設定で seed する（worker の起動時に呼ぶ）
pub async fn seed_from_env() -> anyhow::Result<()> {
    seed(&CmsClient::from_env()).await
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid(schema: &Value, data: Value) -> bool {
        jsonschema::is_valid(schema, &data)
    }

    #[test]
    fn episodes_reference_podcasts_and_type_sections() {
        let s = episodes_schema("pc1");
        assert_eq!(s["properties"]["podcast_id"]["x-ref"], "pc1");
        assert!(valid(&s, json!({ "title": "t", "podcast_id": "p", "sections": [
            { "type": "Serif", "speaker": "3", "text": "a" },
            { "type": "Audio", "url": "u", "from": 1.5 }
        ]})));
        assert!(!valid(&s, json!({ "title": "t", "podcast_id": "p", "sections": [{ "type": "Serif" }] })));
        assert!(!valid(&s, json!({ "title": "t", "podcast_id": "p", "user_id": "u" })));
    }

    #[test]
    fn podcasts_reference_scripts_in_schedule() {
        let s = podcasts_schema("sc1");
        assert_eq!(s["properties"]["schedule"]["properties"]["script_id"]["x-ref"], "sc1");
        assert!(valid(&s, json!({ "title": "t", "icon": "🎙️", "schedule": {
            "cron": "0 0 0 * * ? *", "script_id": "s", "enabled": true, "arguments": { "n": 1 }
        }})));
        assert!(!valid(&s, json!({ "title": "t", "icon": "🎙️", "user_id": "u" })));
    }

    #[test]
    fn mails_reference_episodes_without_podcast_id() {
        let s = mails_schema("ep1");
        assert_eq!(s["properties"]["episode_id"]["x-ref"], "ep1");
        assert!(valid(&s, json!({ "episode_id": "e", "radio_name": "r", "body": "b" })));
        assert!(!valid(&s, json!({ "episode_id": "e", "radio_name": "r", "body": "b", "podcast_id": "p" })));
    }

    /// `CMS_URL` / `CMS_TENANT` / `API_KEY` の CMS に seed する（既存コレクションの更新も確かめる）
    #[tokio::test]
    #[ignore = "requires a running botcast-cms"]
    async fn seed_against_cms() {
        let cms = CmsClient::from_env();
        seed(&cms).await.unwrap();
        // 2 回目は差分が無いので更新しない（冪等）
        seed(&cms).await.unwrap();
    }

    #[test]
    fn episodes_accept_uploaded_file_fields() {
        let s = episodes_schema("p1");
        let file = json!({ "pointer": "images:/c/r/audio", "content_type": "audio/mpeg", "size": 1, "url": "/x" });
        assert!(valid(&s, json!({ "title": "t", "podcast_id": "p", "audio": file, "srt": file })));
    }

    #[test]
    fn scripts_reject_user_id() {
        let s = scripts_schema();
        assert!(valid(&s, json!({ "title": "t", "template": "x", "arguments": {} })));
        assert!(!valid(&s, json!({ "title": "t", "template": "x", "user_id": "u" })));
    }
}

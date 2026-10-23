//! botcast が CMS に置くコレクションのスキーマと、その作成・更新 (seed)
//!
//! スキーマはここが唯一の情報源で、worker の起動時に CMS へ反映する。
//! `x-ref` の値は参照先コレクションの ID（環境ごとに異なる）なので、参照先から順に ensure する。
//! 追加するフィールドは任意にする（既存のレコードが新しいスキーマをそのまま満たすように）。

use super::cms_client::{CmsClient, ListQuery};
use super::episode_generation::MAX_SOURCES;
use super::llm_models::{LLM_MODELS, USER_SETTINGS};
use super::safe_fetch::FETCH_ALLOWED_HOSTS;
use serde_json::{json, Value};

/// 定期実行の LLM モードの指示（`schedule.prompt`）の長さの上限
const SCHEDULE_PROMPT_MAX_LENGTH: u32 = 8000;

/// 全コレクション共通: 認証済みなら誰でも読め、書けるのは所有者（とサービス）
fn access() -> Value {
    json!({ "read": "public", "write": "owner" })
}

/// 声: 参照音声は人の声のサンプルなので、読めるのも所有者（とサービス）だけ
fn voices_access() -> Value {
    json!({ "read": "owner", "write": "owner" })
}

/// LLM のモデルの一覧: ログインしたユーザーなら誰でも読め、書けるのは管理者（とサービス）だけ
fn llm_models_access() -> Value {
    json!({ "read": "public", "write": "authenticated", "write_roles": ["admin"] })
}

/// ユーザーの設定: 本人（とサービス）だけが読み書きする
fn user_settings_access() -> Value {
    json!({ "read": "owner", "write": "owner" })
}

/// Serif の `style` の長さの上限
const STYLE_MAX_LENGTH: u32 = 200;

/// CMS がファイルのアップロード（`/images/{field}`）で書き込むフィールド
fn file_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "pointer": { "type": "string" },
            "content_type": { "type": "string" },
            "size": { "type": "integer", "minimum": 0 },
            "url": { "type": "string" }
        },
        "required": ["pointer", "content_type", "size", "url"]
    })
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

/// 声（話者）のマスタ。台本の `speaker` は番組の配役（`podcasts.cast`）の別名か、この `slug` を書く
pub(crate) fn voices_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            // 英小文字で始める（数字だけの speaker は旧形式のスタイル ID として扱うため）
            "slug": { "type": "string", "pattern": "^[a-z][a-z0-9_-]*$" },
            "name": { "type": "string" },
            // ワーカーの TTS エンジンの設定名（`TTS_ENGINES` の要素）
            "engine": { "type": "string" },
            // VOICEVOX 互換はスタイル ID、OpenAI 互換は voice 名。参照音声を使う場合は省略でき、slug を使う
            "voice_id": { "type": "string" },
            // VOICEVOX 互換で使う、style の名前 → スタイル ID
            "styles": { "type": "object", "additionalProperties": { "type": "string" } },
            // 声の複製に使う参照音声（CMS が `/images/ref_audio` のアップロードで書き込む）
            "ref_audio": file_schema(),
            "params": {
                "type": "object",
                "properties": {
                    "speed": { "type": "number" },
                    "pitch": { "type": "number" },
                    "intonation": { "type": "number" },
                    "volume": { "type": "number" },
                    // エンジンごとの素通しの値
                    "extra": { "type": "object" }
                },
                "additionalProperties": false
            },
            "seed": { "type": "integer" },
            // 声の説明（OpenAI 互換では台本に style が無いセリフの caption に使う）
            "caption": { "type": "string" },
            // 試聴用の音声（CMS が `/images/sample` のアップロードで書き込む）
            "sample": file_schema(),
            // アイコン画像（CMS が `/images/icon` のアップロードで書き込む）
            "icon": file_schema()
        },
        "required": ["slug", "name", "engine"],
        "additionalProperties": false,
        "x-unique": [["slug"]]
    })
}

/// LLM のモデルの一覧。worker はジョブの持ち主の設定（`user_settings.llm_model`）か、`default` のモデルを使う
pub(crate) fn llm_models_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            // API に渡すモデルの ID（例: `claude-opus-5-5`）
            "model_id": { "type": "string", "minLength": 1 },
            // 表示名
            "name": { "type": "string" },
            "provider": { "enum": ["anthropic"] },
            // 既定のモデル（1 件だけにする。複数あれば `order` が小さいものを使う）
            "default": { "type": "boolean" },
            // false なら選べず、設定に残っていても既定のモデルを使う
            "enabled": { "type": "boolean" },
            // 並び順（小さい順）
            "order": { "type": "integer" }
        },
        "required": ["model_id", "name", "provider", "default", "enabled"],
        "additionalProperties": false,
        "x-unique": [["model_id"]]
    })
}

/// ユーザーの設定（1 人 1 件）。API キーなどの秘密情報は置かない（CMS の `/me/secrets`）
pub(crate) fn user_settings_schema(llm_models_id: &str) -> Value {
    json!({
        "type": "object",
        "properties": {
            // 台本生成などで使う LLM のモデル（`llm_models` のレコード ID）
            "llm_model": { "type": "string", "x-ref": llm_models_id }
        },
        "additionalProperties": false
    })
}

/// `llm_models` の初期データ（コレクションが空のときだけ入れる）
pub(crate) fn initial_llm_models() -> Vec<Value> {
    vec![
        json!({ "model_id": "claude-opus-5-5", "name": "Claude Opus 5.5", "provider": "anthropic", "default": true, "enabled": true, "order": 0 }),
        json!({ "model_id": "claude-sonnet-5-5", "name": "Claude Sonnet 5.5", "provider": "anthropic", "default": false, "enabled": true, "order": 1 }),
        json!({ "model_id": "claude-haiku-5-5", "name": "Claude Haiku 5.5", "provider": "anthropic", "default": false, "enabled": true, "order": 2 }),
    ]
}

/// `llm_models` が空なら初期データを入れる（管理者が編集・削除したあとは触らない）
async fn seed_llm_models(cms: &CmsClient) -> anyhow::Result<()> {
    let existing = cms
        .list_records(LLM_MODELS, &ListQuery { limit: Some(1), ..Default::default() })
        .await?;
    if !existing.is_empty() {
        return Ok(());
    }
    tracing::info!("seeding '{LLM_MODELS}'");
    for model in initial_llm_models() {
        cms.create_record_as(LLM_MODELS, model, None).await?;
    }
    Ok(())
}

/// worker が取得してよいホストの一覧（`fetch_url` ツール・定期実行の `schedule.sources`）。
/// ホスト名と完全一致で使う（サブドメインは別に登録する）。IP アドレスは worker が拒否する
pub(crate) fn fetch_allowed_hosts_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            // 小文字のドメイン名（例: `zenn.dev`）。スキーム・ポート・パスは書かない
            "host": {
                "type": "string",
                "pattern": "^[a-z0-9]([a-z0-9-]*[a-z0-9])?(\\.[a-z0-9]([a-z0-9-]*[a-z0-9])?)+$"
            },
            // false なら取得しない
            "enabled": { "type": "boolean" },
            "description": { "type": "string" }
        },
        "required": ["host", "enabled"],
        "additionalProperties": false,
        "x-unique": [["host"]]
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
                    // 台本の作り方。無ければ script（既存のレコード）
                    "generator": { "enum": ["script", "llm"] },
                    "script_id": { "type": "string", "x-ref": scripts_id },
                    "enabled": { "type": "boolean" },
                    // 値はスクリプトの `arguments` (JSON Schema) に従う
                    "arguments": { "type": "object" },
                    // LLM モードの指示
                    "prompt": { "type": "string", "maxLength": SCHEDULE_PROMPT_MAX_LENGTH },
                    // 事前に取得して台本の材料にする URL（worker が `fetch_allowed_hosts` のホストだけ取得する）
                    "sources": {
                        "type": "array",
                        "items": { "type": "string", "pattern": "^https://" },
                        "maxItems": MAX_SOURCES
                    }
                },
                "required": ["cron", "enabled"],
                // script（既定）は script_id、llm は prompt が要る
                "anyOf": [
                    {
                        "properties": { "generator": { "const": "script" } },
                        "required": ["script_id"]
                    },
                    {
                        "properties": { "generator": { "const": "llm" } },
                        "required": ["generator", "prompt"]
                    }
                ],
                "additionalProperties": false
            },
            // 配役: 台本の `speaker` に書く別名 → voices の slug。
            // 数字だけの別名は旧形式の speaker（VOICEVOX のスタイル ID）と紛らわしいので禁止する
            "cast": {
                "type": "object",
                "propertyNames": { "minLength": 1, "not": { "pattern": "^[0-9]+$" } },
                "additionalProperties": { "type": "string" }
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
                                // 番組の配役の別名・voices の slug・旧形式の VOICEVOX のスタイル ID
                                "speaker": { "type": "string" },
                                "text": { "type": "string" },
                                // 口調・感情の指定（自由文）
                                "style": { "type": "string", "maxLength": STYLE_MAX_LENGTH }
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
    cms.ensure_collection("voices", voices_schema(), voices_access())
        .await?;
    let scripts = cms.ensure_collection("scripts", scripts_schema(), access()).await?;
    let podcasts = cms
        .ensure_collection("podcasts", podcasts_schema(&scripts), access())
        .await?;
    let episodes = cms
        .ensure_collection("episodes", episodes_schema(&podcasts), access())
        .await?;
    cms.ensure_collection("mails", mails_schema(&episodes), access())
        .await?;
    let llm_models = cms
        .ensure_collection(LLM_MODELS, llm_models_schema(), llm_models_access())
        .await?;
    seed_llm_models(cms).await?;
    cms.ensure_collection(
        USER_SETTINGS,
        user_settings_schema(&llm_models),
        user_settings_access(),
    )
    .await?;
    // 取得してよいホストは llm_models と同じく、読めるのはログインしたユーザー全員・書けるのは管理者とサービスだけ
    cms.ensure_collection(
        FETCH_ALLOWED_HOSTS,
        fetch_allowed_hosts_schema(),
        llm_models_access(),
    )
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
    fn episodes_accept_style_and_keep_legacy_sections() {
        let s = episodes_schema("pc1");
        // 旧形式（VOICEVOX のスタイル ID・style なし）はそのまま通る
        assert!(valid(&s, json!({ "title": "t", "podcast_id": "p", "sections": [
            { "type": "Serif", "speaker": "3", "text": "a" }
        ]})));
        assert!(valid(&s, json!({ "title": "t", "podcast_id": "p", "sections": [
            { "type": "Serif", "speaker": "host", "text": "a", "style": "元気に" }
        ]})));
        assert!(!valid(&s, json!({ "title": "t", "podcast_id": "p", "sections": [
            { "type": "Serif", "speaker": "host", "text": "a", "style": 1 }
        ]})));
        let style = |n: usize| json!({ "title": "t", "podcast_id": "p", "sections": [
            { "type": "Serif", "speaker": "host", "text": "a", "style": "あ".repeat(n) }
        ]});
        assert!(valid(&s, style(STYLE_MAX_LENGTH as usize)));
        assert!(!valid(&s, style(STYLE_MAX_LENGTH as usize + 1)));
    }

    #[test]
    fn podcasts_accept_cast_and_keep_legacy_records() {
        let s = podcasts_schema("sc1");
        assert!(valid(&s, json!({ "title": "t", "icon": "🎙️" })));
        assert!(valid(&s, json!({ "title": "t", "icon": "🎙️", "cast": { "ホスト": "zundamon" } })));
        assert!(!valid(&s, json!({ "title": "t", "icon": "🎙️", "cast": { "ホスト": 3 } })));
        // 数字だけの別名は旧形式の speaker と取り違えるので不可
        assert!(!valid(&s, json!({ "title": "t", "icon": "🎙️", "cast": { "3": "zundamon" } })));
        assert!(!valid(&s, json!({ "title": "t", "icon": "🎙️", "cast": { "": "zundamon" } })));
        assert!(valid(&s, json!({ "title": "t", "icon": "🎙️", "cast": { "3号": "zundamon" } })));
    }

    #[test]
    fn voices_require_slug_name_engine() {
        let s = voices_schema();
        assert_eq!(s["x-unique"], json!([["slug"]]));
        assert!(valid(&s, json!({ "slug": "zundamon", "name": "ずんだもん", "engine": "voicevox", "voice_id": "3" })));
        let file = json!({ "pointer": "images:/c/r/ref_audio", "content_type": "audio/wav", "size": 1, "url": "/x" });
        assert!(valid(&s, json!({
            "slug": "narrator-1", "name": "語り", "engine": "irodori",
            "styles": { "ささやき": "22" }, "ref_audio": file, "sample": file,
            "params": { "speed": 1.1, "extra": { "k": 1 } }, "seed": 42, "caption": "落ち着いた声"
        })));
        assert!(!valid(&s, json!({ "slug": "a", "name": "n" })));
        assert!(!valid(&s, json!({ "slug": "Bad Slug", "name": "n", "engine": "e" })));
        assert!(!valid(&s, json!({ "slug": "3", "name": "n", "engine": "e" })));
        assert!(!valid(&s, json!({ "slug": "a", "name": "n", "engine": "e", "params": { "rate": 1 } })));
        assert!(!valid(&s, json!({ "slug": "a", "name": "n", "engine": "e", "user_id": "u" })));
        // ファイルのフィールドは CMS のファイルの形だけ
        for field in ["ref_audio", "sample", "icon"] {
            assert!(!valid(&s, json!({ "slug": "a", "name": "n", "engine": "e", field: {} })));
            assert!(!valid(&s, json!({ "slug": "a", "name": "n", "engine": "e", field: { "url": "/x" } })));
        }
    }

    #[test]
    fn voices_accept_icon_and_keep_existing_records() {
        let s = voices_schema();
        let audio = json!({ "pointer": "images:/c/r/sample", "content_type": "audio/wav", "size": 1, "url": "/x" });
        // icon を足す前のレコードはそのまま通る
        let existing = json!({ "slug": "narrator", "name": "語り", "engine": "irodori", "sample": audio });
        assert!(valid(&s, existing.clone()));
        let mut with_icon = existing;
        with_icon["icon"] = json!({ "pointer": "images:/c/r/icon", "content_type": "image/png", "size": 1, "url": "/y" });
        assert!(valid(&s, with_icon));
    }

    #[test]
    fn voices_are_readable_only_by_owner() {
        let access = voices_access();
        assert_eq!(access, json!({ "read": "owner", "write": "owner" }));
        serde_json::from_value::<cms_client::models::CollectionAccess>(access).unwrap();
    }

    #[test]
    fn llm_models_are_written_only_by_admins() {
        let access = llm_models_access();
        let typed: cms_client::models::CollectionAccess =
            serde_json::from_value(access.clone()).unwrap();
        // ensure_collection は CMS が返す access と比べるので、型を通しても同じ JSON になること
        assert_eq!(serde_json::to_value(typed).unwrap(), access);
        assert_eq!(access["write_roles"], json!(["admin"]));
        let settings = user_settings_access();
        assert_eq!(settings, json!({ "read": "owner", "write": "owner" }));
        serde_json::from_value::<cms_client::models::CollectionAccess>(settings).unwrap();
    }

    #[test]
    fn initial_llm_models_have_exactly_one_default() {
        let s = llm_models_schema();
        assert_eq!(s["x-unique"], json!([["model_id"]]));
        let models = initial_llm_models();
        for m in &models {
            assert!(valid(&s, m.clone()), "{m}");
        }
        assert_eq!(models.iter().filter(|m| m["default"] == true).count(), 1);
        assert!(!valid(&s, json!({ "model_id": "x", "name": "n", "provider": "openai", "default": false, "enabled": true })));
        assert!(!valid(&s, json!({ "model_id": "", "name": "n", "provider": "anthropic", "default": false, "enabled": true })));
    }

    #[test]
    fn user_settings_reference_llm_models() {
        let s = user_settings_schema("lm1");
        assert_eq!(s["properties"]["llm_model"]["x-ref"], "lm1");
        assert!(valid(&s, json!({})));
        assert!(valid(&s, json!({ "llm_model": "m1" })));
        assert!(!valid(&s, json!({ "llm_model": "m1", "api_key": "k" })));
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
    fn podcasts_keep_legacy_schedules_valid() {
        let s = podcasts_schema("sc1");
        // generator を足す前のレコード（script_id 必須だった形）はそのまま通る
        for schedule in [
            json!({ "cron": "0 0 12 * * Fri *", "script_id": "s", "enabled": true }),
            json!({ "cron": "0 0 12 * * Fri *", "script_id": "s", "enabled": false, "arguments": {} }),
        ] {
            assert!(valid(&s, json!({ "title": "t", "icon": "🎙️", "schedule": schedule })), "{schedule}");
        }
        // script（既定）は今までどおり script_id が要る
        assert!(!valid(&s, json!({ "title": "t", "icon": "🎙️", "schedule": {
            "cron": "0 0 12 * * Fri *", "enabled": true
        }})));
        assert!(!valid(&s, json!({ "title": "t", "icon": "🎙️", "schedule": {
            "cron": "0 0 12 * * Fri *", "enabled": true, "generator": "script"
        }})));
        // script モードでも事前に取得する資料を渡せる
        assert!(valid(&s, json!({ "title": "t", "icon": "🎙️", "schedule": {
            "cron": "0 0 12 * * Fri *", "enabled": true, "script_id": "s",
            "sources": ["https://zenn.dev/api/articles?order=weekly&count=5"]
        }})));
    }

    #[test]
    fn podcasts_accept_llm_schedules() {
        let s = podcasts_schema("sc1");
        let llm = json!({
            "cron": "0 0 22 * * Sun *", "enabled": true, "generator": "llm",
            "prompt": "今週の Zenn の人気記事を紹介して",
            "sources": ["https://zenn.dev/api/articles?order=weekly&count=5"]
        });
        assert!(valid(&s, json!({ "title": "t", "icon": "🎙️", "schedule": llm })));
        // llm は script_id が要らず、prompt が要る
        let mut no_prompt = llm.clone();
        no_prompt.as_object_mut().unwrap().remove("prompt");
        assert!(!valid(&s, json!({ "title": "t", "icon": "🎙️", "schedule": no_prompt })));
        for (k, v) in [
            ("generator", json!("other")),
            ("sources", json!(["http://zenn.dev/"])),
            ("sources", json!(vec!["https://zenn.dev/"; MAX_SOURCES + 1])),
            ("prompt", json!("あ".repeat(SCHEDULE_PROMPT_MAX_LENGTH as usize + 1))),
            ("model", json!("x")),
        ] {
            let mut bad = llm.clone();
            bad[k] = v;
            assert!(!valid(&s, json!({ "title": "t", "icon": "🎙️", "schedule": bad })), "{k}");
        }
    }

    #[test]
    fn fetch_allowed_hosts_are_admin_managed_domains() {
        let s = fetch_allowed_hosts_schema();
        assert_eq!(s["x-unique"], json!([["host"]]));
        assert!(valid(&s, json!({ "host": "zenn.dev", "enabled": true, "description": "Zenn" })));
        for host in ["https://zenn.dev", "zenn.dev/api", "Zenn.dev", "localhost", "zenn.dev:443", ""] {
            assert!(!valid(&s, json!({ "host": host, "enabled": true })), "{host}");
        }
        assert!(!valid(&s, json!({ "host": "zenn.dev" })));
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

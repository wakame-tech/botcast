# CMS のデータ設計の見直し Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** botcast のコレクションに `x-ref` を宣言し、所有者を CMS の `owner_id` に一本化し、スキーマ変更を本番に反映できるようにする。

**Architecture:** botcast-cms に「コレクションのスキーマ更新」と「API キーでの代理実行 (`X-On-Behalf-Of`)」を足す。botcast の worker は seed を HTTP の `CmsClient` に移し、参照先から順にコレクションを ensure して ID を `x-ref` に埋め込む。web は `user_id` をやめ `owner_id` で編集導線を出し分ける。既存データは SurrealQL で 1 回移行する。

**Tech Stack:** Rust (Axum / reqwest / jsonschema 0.17)、TypeSpec、React + TanStack Router、SurrealDB

設計: `docs/superpowers/specs/2026-10-15-cms-ownership-design.md`

---

## 作業場所

- **botcast-cms**（Task 1〜4）: `~/dev/botcast-cms` は別セッションの checkout なので触らない。worktree を作る:
  `git -C ~/dev/botcast-cms fetch && git -C ~/dev/botcast-cms worktree add ~/dev/botcast-cms-ownership -b feat/collection-schema-update origin/develop`
  PR は `develop` 向け。
- **botcast**（Task 5〜9）: `~/dev/botcast` のブランチ `feat/cms-ownership`。PR は `main` 向け。botcast-cms のリリース後にマージする。

## ファイル構成

botcast-cms:
- Modify `spec/main.tsp` — `UpdateCollection.schema?`
- Regenerate `spec/tsp-output/`, `cms/crates/openapi-gen/`, `mcp/`
- Create `cms/crates/api/src/schema_update.rs` — スキーマ変更の検査（純粋関数）
- Modify `cms/crates/api/src/repository/{mod,surrealdb}.rs` — `update_collection` に `schema`
- Modify `cms/crates/api/src/openapi_adapter.rs` — `collection_api_update`
- Modify `cms/crates/api/src/middleware/auth.rs` — `X-On-Behalf-Of`
- Modify `docs/specres/...`（該当カード）、`CLAUDE.md`

botcast:
- Create `crates/worker/src/usecase/collections.rs` — スキーマ定義と seed
- Modify `crates/worker/src/usecase/cms_client.rs` — `ensure_collection` / `create_record_as`
- Modify `crates/worker/src/usecase/mod.rs`, `crates/worker/src/main.rs`, `crates/worker/src/usecase/mcp_client.rs`
- Modify `crates/worker/src/usecase/episode_generation.rs`
- Modify `web/src/lib/cms_client.ts`, `web/src/lib/cms_api.d.ts`（生成）、作成・編集画面
- Create `deploy/selfhost/migrations/2026-10-15-owner-id.surql`
- Modify `CLAUDE.md`

---

### Task 1: TypeSpec に `UpdateCollection.schema` を追加して再生成

**Files:** Modify `spec/main.tsp:35-38`、生成物

- [ ] **Step 1: TypeSpec を編集**

```tsp
model UpdateCollection {
  name: string;

  /**
   * New JSON Schema. Every existing record must satisfy it (otherwise 400 and nothing changes).
   * `x-unique` cannot be changed.
   */
  schema?: Record<string>;

  access?: CollectionAccess;
}
```

- [ ] **Step 2: 生成**

Run: `cd spec && npm ci && npm run build && cd ../cms && just gen_server && just gen_mcp`
Expected: `cms/crates/openapi-gen/src/models.rs` の `UpdateCollection` に `schema: Option<...>` が増える。`git diff --stat` で tsp-output / openapi-gen / mcp が変わる。

- [ ] **Step 3: ビルドが通るか確認**

Run: `cd cms && cargo check`
Expected: 生成された `UpdateCollection` を構築しているテストがあれば `schema` 不足でエラー → `schema: None` を足す。

- [ ] **Step 4: Commit** `feat(spec): コレクション更新でスキーマを受け付ける`

### Task 2: スキーマ変更の検査とリポジトリ

**Files:** Create `cms/crates/api/src/schema_update.rs`、Modify `cms/crates/api/src/main.rs`（`mod schema_update;`）、`repository/mod.rs`、`repository/surrealdb.rs`

- [ ] **Step 1: 失敗するテストを書く**（`schema_update.rs`）

```rust
// @specre <コレクション名前変更カードの ID>
//! コレクションのスキーマ変更が既存レコードと整合するかの検査

use crate::error::AppError;
use jsonschema::JSONSchema;
use serde_json::Value;

/// `old` から `new` へのスキーマ変更を許すか。
///
/// - `x-unique` は変えられない（インデックスを作り直さないため）
/// - 既存の全レコード (`(record_id, data)`) が `new` を満たすこと
pub fn check_schema_update(
    old: &Value,
    new: &Value,
    records: &[(String, Value)],
) -> Result<(), AppError> {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn schema(required: &[&str]) -> Value {
        json!({
            "type": "object",
            "properties": { "title": { "type": "string" }, "podcast_id": { "type": "string" } },
            "required": required,
            "additionalProperties": false
        })
    }

    #[test]
    fn accepts_change_all_records_satisfy() {
        let records = vec![("r1".into(), json!({ "title": "a", "podcast_id": "p" }))];
        assert!(check_schema_update(&schema(&[]), &schema(&["podcast_id"]), &records).is_ok());
    }

    #[test]
    fn rejects_change_violated_by_existing_record() {
        let records = vec![
            ("r1".into(), json!({ "title": "a", "podcast_id": "p" })),
            ("r2".into(), json!({ "title": "b" })),
        ];
        let err = check_schema_update(&schema(&[]), &schema(&["podcast_id"]), &records).unwrap_err();
        assert!(matches!(&err, AppError::Validation { message } if message.contains("r2") && !message.contains("r1")));
    }

    #[test]
    fn rejects_x_unique_change() {
        let mut new = schema(&[]);
        new["x-unique"] = json!([["title"]]);
        assert!(matches!(check_schema_update(&schema(&[]), &new, &[]), Err(AppError::Validation { .. })));
    }

    #[test]
    fn rejects_invalid_schema() {
        let new = json!({ "type": 12 });
        assert!(matches!(check_schema_update(&schema(&[]), &new, &[]), Err(AppError::Validation { .. })));
    }
}
```

`@specre` の ID は `docs/specres/コレクション管理/コレクションの一覧取得・詳細取得・名前変更・削除ができる.md` の frontmatter `id` を使う。

- [ ] **Step 2: 失敗を確認** — Run: `cd cms && cargo test -p api schema_update` → Expected: `todo!()` で panic

- [ ] **Step 3: 実装**

```rust
pub fn check_schema_update(
    old: &Value,
    new: &Value,
    records: &[(String, Value)],
) -> Result<(), AppError> {
    if crate::unique::unique_keys(old)? != crate::unique::unique_keys(new)? {
        return Err(AppError::validation("x-unique cannot be changed"));
    }
    let compiled = JSONSchema::compile(new)
        .map_err(|e| AppError::validation(format!("Invalid schema: {e}")))?;
    let violating: Vec<&str> = records
        .iter()
        .filter(|(_, data)| !compiled.is_valid(data))
        .map(|(id, _)| id.as_str())
        .collect();
    if !violating.is_empty() {
        return Err(AppError::validation(format!(
            "Existing records violate the new schema: {}",
            violating.join(", ")
        )));
    }
    Ok(())
}
```

- [ ] **Step 4: 通過を確認** — Run: `cd cms && cargo test -p api schema_update` → 4 passed

- [ ] **Step 5: リポジトリに `schema` を追加**

`repository/mod.rs` のトレイトとテスト用モック（`:456`）:

```rust
    async fn update_collection(
        &self,
        collection_id: &str,
        name: &str,
        schema: Option<Value>,
        access: Option<Access>,
    ) -> anyhow::Result<Collection, AppError>;
```

`repository/surrealdb.rs:252`:

```rust
    async fn update_collection(
        &self,
        collection_id: &str,
        name: &str,
        schema: Option<Value>,
        access: Option<Access>,
    ) -> anyhow::Result<Collection, AppError> {
        // `.merge()` なので None のフィールドは既存値に触れない
        let mut patch = serde_json::json!({ "name": name });
        if let Some(schema) = schema {
            patch["schema"] = schema;
        }
        if let Some(access) = access {
            patch["access"] = serde_json::to_value(access)
                .map_err(|e| AppError::internal(format!("Failed to convert access: {e}")))?;
        }
        // (以降は既存のまま)
```

- [ ] **Step 6: Commit** `feat(api): コレクションのスキーマ変更を既存レコードで検査する`

### Task 3: `collection_api_update` でスキーマを更新する

**Files:** Modify `cms/crates/api/src/openapi_adapter.rs:701-725`

- [ ] **Step 1: ハンドラを変更**

```rust
        let access = access_from_body(body.access.as_ref())?;
        let schema = match &body.schema {
            Some(raw) if path_params.collection_id != SCRIPTS_COLLECTION => {
                let new = serde_json::to_value(raw)
                    .map_err(|e| AppError::internal(format!("Failed to convert schema: {e}")))?;
                let repo = self.repo()?;
                reactive::validate_collection_schema(&repo, &new).await?;
                let current = repo.get_by_id(&path_params.collection_id).await?;
                let records: Vec<(String, Value)> = repo
                    .list_record(&path_params.collection_id, &ListOptions::default())
                    .await?
                    .into_iter()
                    .map(|r| (r.id.to_string(), r.data))
                    .collect();
                crate::schema_update::check_schema_update(&current.schema, &new, &records)?;
                Some(new)
            }
            // 組み込みの `_scripts` のスキーマは CMS が管理する
            Some(_) => return Err(AppError::Forbidden),
            None => None,
        };
        let collection = self.repo()?
            .update_collection(&path_params.collection_id, &body.name, schema, access)
            .await?;
```

`Record` の id / data のフィールド名・型と `current.schema` の型は `repository/mod.rs` の定義に合わせる（`schema` が `Value` 以外なら `serde_json::to_value`）。`ListOptions::default()` は limit なし（全件）であることを `query/filter.rs` で確認する。

- [ ] **Step 2: 確認**

Run: `cd cms && cargo clippy -- -D warnings && cargo test`
Expected: 全テスト通過

- [ ] **Step 3: 動作確認**（ローカル `just run` に対して）

```bash
T=localhost:3002/t/botcast
C=$(curl -s -XPOST $T/collections -H "X-Api-Key: $KEY" -H 'Content-Type: application/json' \
  -d '{"name":"t1","schema":{"type":"object","properties":{"a":{"type":"string"}}}}' | jq -r .id | sed 's/^collection://')
curl -s -XPOST $T/records/$C -H "X-Api-Key: $KEY" -H 'Content-Type: application/json' -d '{"data":{}}'
# 既存レコードが満たさない → 400
curl -s -w '%{http_code}\n' -XPUT $T/collections/$C -H "X-Api-Key: $KEY" -H 'Content-Type: application/json' \
  -d '{"name":"t1","schema":{"type":"object","required":["a"]}}'
# 満たす変更 → 200 で schema が変わる
curl -s -XPUT $T/collections/$C -H "X-Api-Key: $KEY" -H 'Content-Type: application/json' \
  -d '{"name":"t1","schema":{"type":"object","properties":{"a":{"type":"string","x-ref":"x"}}}}' | jq .schema
```

- [ ] **Step 4: Commit** `feat(api): PUT /collections でスキーマを更新できるようにする`

### Task 4: `X-On-Behalf-Of` による代理実行

**Files:** Modify `cms/crates/api/src/middleware/auth.rs`

- [ ] **Step 1: 失敗するテストを書く**（`mod tests` に追加）

```rust
    #[test]
    fn on_behalf_of_accepts_user_subject() {
        assert_eq!(on_behalf_subject(None), Ok(None));
        assert_eq!(on_behalf_subject(Some("user:abc")), Ok(Some("user:abc".to_string())));
    }

    #[test]
    fn on_behalf_of_rejects_non_user_subject() {
        for v in ["service", "user:", "", "abc"] {
            assert_eq!(on_behalf_subject(Some(v)), Err(StatusCode::BAD_REQUEST), "{v}");
        }
    }
```

- [ ] **Step 2: 失敗を確認** — Run: `cd cms && cargo test -p api on_behalf` → コンパイルエラー

- [ ] **Step 3: 実装**

```rust
/// API キー認証時に `X-On-Behalf-Of` で指定されたユーザー。
/// サービスがユーザーの代わりにレコードを作る（`owner_id` をそのユーザーにする）ために使う。
/// `user:` で始まる subject だけを受け付ける。
pub fn on_behalf_subject(header: Option<&str>) -> Result<Option<String>, StatusCode> {
    match header {
        None => Ok(None),
        Some(s) if s.strip_prefix("user:").is_some_and(|id| !id.is_empty()) => {
            Ok(Some(s.to_string()))
        }
        Some(_) => Err(StatusCode::BAD_REQUEST),
    }
}
```

API キー分岐（`auth_middleware` 内）:

```rust
    if provided.is_some() && api_key_matches(&tenant, provided) {
        let on_behalf = on_behalf_subject(
            req.headers().get("X-On-Behalf-Of").and_then(|v| v.to_str().ok()),
        )?;
        let claims = JwtClaims {
            sub: on_behalf.unwrap_or_else(|| "service".to_string()),
            tenant: tenant.id.clone(),
            exp: usize::MAX,
        };
```

JWT 分岐ではヘッダーを読まない（無視）。

- [ ] **Step 4: 通過を確認** — Run: `cd cms && cargo clippy -- -D warnings && cargo test` → 全通過

- [ ] **Step 5: 動作確認**

```bash
U=$(curl -s -XPOST $T/auth/signin -H 'Content-Type: application/json' -d '{"email":"w1@example.com","password":"123456"}' | jq -r .token | cut -d. -f2 | base64 -d 2>/dev/null | jq -r .sub)
curl -s -XPOST $T/records/$C -H "X-Api-Key: $KEY" -H "X-On-Behalf-Of: $U" -H 'Content-Type: application/json' -d '{"data":{"a":"x"}}' | jq .owner_id
# → "$U"
curl -s -w '%{http_code}\n' -XPOST $T/records/$C -H "X-Api-Key: $KEY" -H "X-On-Behalf-Of: service" -d '{}'
# → 400
```

- [ ] **Step 6: specre と CLAUDE.md**
  - `CLAUDE.md` の「認証」に 1 行: `X-Api-Key` と一緒に `X-On-Behalf-Of: user:<id>` を送ると、そのユーザーとして処理する（作成レコードの owner_id もそのユーザー）。JWT 認証では無視する
  - `docs/specres` の該当カード（API キー認証のカード、コレクション名前変更のカード）にシナリオと関連ファイル（`schema_update.rs`）を追記し、`specre index` → `specre health-check`

- [ ] **Step 7: Commit・PR** `feat(api): API キー認証で X-On-Behalf-Of による代理実行を受け付ける`
  `gh pr create --base develop`。マージ後 `just release minor` でリリースし、GHCR の `botcast-cms-{api,mcp}` が更新されるのを待つ。

### Task 5: worker — `CmsClient` に `ensure_collection` / `create_record_as`

**Files:** Modify `crates/worker/src/usecase/cms_client.rs`

- [ ] **Step 1: 失敗するテストを書く**（`mod tests`、差分判定は純粋関数にする）

```rust
    #[test]
    fn collection_needs_update_when_schema_or_access_differs() {
        let schema = json!({ "type": "object" });
        let access = json!({ "read": "public", "write": "owner" });
        let same = json!({ "name": "p", "schema": schema, "access": access });
        assert!(!collection_needs_update(&same, &schema, &access));
        let other_schema = json!({ "name": "p", "schema": { "type": "string" }, "access": access });
        assert!(collection_needs_update(&other_schema, &schema, &access));
        let no_access = json!({ "name": "p", "schema": schema });
        assert!(collection_needs_update(&no_access, &schema, &access));
    }
```

- [ ] **Step 2: 失敗を確認** — Run: `cargo test -p worker collection_needs_update` → コンパイルエラー

- [ ] **Step 3: 実装**

```rust
/// 既存コレクションのスキーマ・access が望む値と違うか
fn collection_needs_update(
    current: &serde_json::Value,
    schema: &serde_json::Value,
    access: &serde_json::Value,
) -> bool {
    &current["schema"] != schema || &current["access"] != access
}
```

`impl CmsClient` に:

```rust
    /// コレクションを作成し、既にあればスキーマ・access を合わせる。ID を返す
    pub(crate) async fn ensure_collection(
        &self,
        name: &str,
        schema: serde_json::Value,
        access: serde_json::Value,
    ) -> anyhow::Result<String> {
        let collections = Self::send(self.request(reqwest::Method::GET, "/collections")).await?;
        let existing = collections
            .as_array()
            .into_iter()
            .flatten()
            .find(|c| c["name"] == name)
            .cloned();
        let body = serde_json::json!({ "name": name, "schema": schema, "access": access });
        let collection = match existing {
            Some(c) => {
                if collection_needs_update(&c, &schema, &access) {
                    let id = parse_collection_ids(&serde_json::json!([c]))[name].clone();
                    tracing::info!("updating collection '{name}'");
                    Self::send(
                        self.request(reqwest::Method::PUT, &format!("/collections/{id}")).json(&body),
                    )
                    .await
                    .with_context(|| format!("Failed to update collection '{name}'"))?
                } else {
                    c
                }
            }
            None => {
                tracing::info!("creating collection '{name}'");
                Self::send(self.request(reqwest::Method::POST, "/collections").json(&body)).await?
            }
        };
        let id = parse_collection_ids(&serde_json::json!([collection]))
            .remove(name)
            .context("id missing in collection response")?;
        self.collection_ids.lock().await.insert(name.to_string(), id.clone());
        Ok(id)
    }
```

`create_record` を `create_record_as` に一般化（既存の `create_record` は `create_record_as(c, d, None)` を呼ぶ）:

```rust
    /// `owner` を指定すると `X-On-Behalf-Of` でそのユーザーの所有として作成する
    pub(crate) async fn create_record_as(
        &self,
        collection: &str,
        data: serde_json::Value,
        owner: Option<&str>,
    ) -> anyhow::Result<String> {
        let cid = self.collection_id(collection).await?;
        let mut req = self
            .request(reqwest::Method::POST, &format!("/records/{cid}"))
            .json(&serde_json::json!({ "data": data }));
        if let Some(owner) = owner {
            req = req.header("X-On-Behalf-Of", owner);
        }
        let res = Self::send(req).await?;
        res["id"]
            .as_str()
            .map(str::to_string)
            .context("id missing in create response")
    }
```

- [ ] **Step 4: 通過を確認** — Run: `cargo test -p worker cms_client` → 通過

- [ ] **Step 5: Commit** `feat(worker): CmsClient でコレクションの作成・スキーマ更新と代理作成をする`

### Task 6: worker — スキーマ定義と seed を `collections.rs` に移す

**Files:** Create `crates/worker/src/usecase/collections.rs`、Modify `usecase/mod.rs`（`pub mod collections;`）、`main.rs`、`mcp_client.rs`（`ensure_collection` とそのテストを削除）

- [ ] **Step 1: テストを書く**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use jsonschema::JSONSchema;

    fn valid(schema: &Value, data: Value) -> bool {
        JSONSchema::compile(schema).unwrap().is_valid(&data)
    }

    #[test]
    fn episodes_reference_podcasts_and_type_sections() {
        let s = episodes_schema("pc1");
        assert_eq!(s["properties"]["podcast_id"]["x-ref"], "pc1");
        assert!(valid(&s, json!({ "title": "t", "podcast_id": "p", "sections": [
            { "type": "Serif", "speaker": "3", "text": "a" },
            { "type": "Audio", "url": "u", "from": 1.0 }
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
    }

    #[test]
    fn mails_reference_episodes_without_podcast_id() {
        let s = mails_schema("ep1");
        assert_eq!(s["properties"]["episode_id"]["x-ref"], "ep1");
        assert!(valid(&s, json!({ "episode_id": "e", "radio_name": "r", "body": "b" })));
        assert!(!valid(&s, json!({ "episode_id": "e", "radio_name": "r", "body": "b", "podcast_id": "p" })));
    }
}
```

`jsonschema` が worker の依存に無ければ `[dev-dependencies] jsonschema = "0.17"` を足す（既に `validate_arguments` で使っていれば不要。`grep jsonschema crates/worker/Cargo.toml`）。

- [ ] **Step 2: 失敗を確認** — Run: `cargo test -p worker collections` → コンパイルエラー

- [ ] **Step 3: 実装**

```rust
//! botcast が CMS に置くコレクションのスキーマと、その作成・更新 (seed)

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
            "duration_sec": { "type": "integer" }
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

/// `x-ref` に参照先の ID を埋め込むため、参照先から順に作成・更新する
pub async fn seed(cms: &CmsClient) -> anyhow::Result<()> {
    let scripts = cms.ensure_collection("scripts", scripts_schema(), access()).await?;
    let podcasts = cms.ensure_collection("podcasts", podcasts_schema(&scripts), access()).await?;
    let episodes = cms.ensure_collection("episodes", episodes_schema(&podcasts), access()).await?;
    cms.ensure_collection("mails", mails_schema(&episodes), access()).await?;
    Ok(())
}
```

`CmsClient` が `pub(crate)` のため、`main.rs`（bin）から呼ぶ入口を lib 側に用意する: `pub async fn seed_from_env() -> anyhow::Result<()> { seed(&CmsClient::from_env()).await }` を `collections.rs` に置き、`seed` は `pub(crate)` にする。

Step 1 のテスト内 `valid(... sections Serif)` で `"from": 1.0` を使っているのは number 型の確認のため。

- [ ] **Step 4: `main.rs` を差し替え**

`seed_cms_collections()` 関数と `McpClient` の import を削除し、`main` で:

```rust
    worker::usecase::collections::seed_from_env().await?;
```

`mcp_client.rs` の `ensure_collection` と、それを使うテストを削除する（テストのレコード作成で `user_id` を入れている箇所も削除）。

- [ ] **Step 5: 確認** — Run: `cargo clippy -p worker -- -D warnings && cargo test -p worker`（MCP_SERVER_ARGS が要る統合テストは既存と同じ扱い）→ 通過

- [ ] **Step 6: Commit** `feat(worker): コレクションに x-ref・所有者アクセス・厳密なスキーマを宣言し、seed で更新する`

### Task 7: worker — エピソードを番組の所有者で作成する

**Files:** Modify `crates/worker/src/usecase/episode_generation.rs:153-165`、関連テスト

- [ ] **Step 1: テスト** — `generate` が `create_record_as` に `owner` を渡すことを確かめる。既存のテストがモック `cms` を使っていればそこに追加し、使っていなければ所有者を取り出す純粋関数をテストする:

```rust
    #[test]
    fn episode_owner_is_podcast_owner_unless_service() {
        assert_eq!(episode_owner(&json!({ "owner_id": "user:a" })), Some("user:a"));
        assert_eq!(episode_owner(&json!({ "owner_id": "service" })), None);
        assert_eq!(episode_owner(&json!({ "owner_id": null })), None);
    }
```

- [ ] **Step 2: 実装**

```rust
/// 生成するエピソードの所有者 = 番組の所有者。サービス所有の番組なら代理しない
fn episode_owner(podcast: &Value) -> Option<&str> {
    podcast["owner_id"].as_str().filter(|o| o.starts_with("user:"))
}
```

```rust
        self.cms
            .create_record_as(
                "episodes",
                serde_json::json!({
                    "podcast_id": podcast_id,
                    "title": episode.title,
                    "description": "",
                    "sections": episode.sections,
                }),
                episode_owner(&podcast),
            )
            .await
```

`task_service.rs:193` の LLM 向け説明に `user_id` が出ていれば削除。`grep -rn user_id crates/` が 0 件になること。

- [ ] **Step 3: 確認** — Run: `cargo test -p worker` → 通過

- [ ] **Step 4: Commit** `feat(worker): 生成したエピソードを番組の所有者のものにする`

### Task 8: web — `user_id` をやめ、`owner_id` で編集導線を出し分ける

**Files:** `web/src/lib/cms_client.ts`、`web/src/lib/cms_api.d.ts`（生成）、`routes/podcasts/new.lazy.tsx`、`routes/scripts/new.lazy.tsx`、`components/mail/EpisodeMails.tsx`、`routes/podcasts/$podcastId.lazy.tsx`、`routes/podcasts/$podcastId_/edit.lazy.tsx`、`routes/podcasts/$podcastId_/episodes/$episodeId.lazy.tsx`、`routes/scripts/$scriptId.lazy.tsx`、`routes/scripts/$scriptId_/edit.lazy.tsx`

- [ ] **Step 1: 型を再生成** — Run: `cd web && npm run generate`（`../botcast-cms` を参照するので、`ln -sfn ~/dev/botcast-cms-ownership ../botcast-cms` が要るなら generate スクリプトのパスを確認し、`~/dev/botcast-cms` は触らない。Task 4 のブランチの openapi.yaml を読ませる）

- [ ] **Step 2: `cms_client.ts`**
  - `Podcast` / `Episode` / `Script` / `Mail` から `user_id` を削除し、`owner_id: string | null` を追加（`recordTo*` で `record.owner_id`）
  - `Mail` から `podcast_id` を削除
  - 所有者判定ヘルパーを追加:

```ts
/** サインイン中のユーザーがレコードの所有者か（編集・削除の導線を出すか） */
export function isOwner(
	record: { owner_id: string | null },
	userId: string | null,
): boolean {
	return userId !== null && record.owner_id === userId;
}
```

- [ ] **Step 3: 作成画面** — `podcasts/new`・`scripts/new`・`EpisodeMails` の送信データから `user_id`（と mail の `podcast_id`）を削除。`useSession` が不要になれば import も削除

- [ ] **Step 4: 編集導線** — `useSession().userId` を取り、`isOwner(podcast, userId)` のときだけ
  - `podcasts/$podcastId`: 「編集」リンク
  - `episodes/$episodeId`: 「削除」ボタンと、音声生成などの書き込み操作のボタン
  - `scripts/$scriptId`: edit / delete ボタン
  - `EpisodeMails`: 自分のお便りの削除（既存にあれば）
- [ ] **Step 5: 編集画面のコメント** — `podcasts/$podcastId_/edit`・`scripts/$scriptId_/edit` の「既存の値 (user_id / schedule など) を保って」を「既存の値 (schedule など)」に直す。`scripts/$scriptId` のサンプル context から `user_id` と mail の `podcast_id` を削除

- [ ] **Step 6: 確認** — Run: `cd web && npm run check && npm run build`、`grep -rn user_id src --exclude=cms_api.d.ts` が 0 件

- [ ] **Step 7: Commit** `feat(web): 所有者を CMS の owner_id で判定し、user_id を送らない`

### Task 9: 移行 SurrealQL・ドキュメント・PR

**Files:** Create `deploy/selfhost/migrations/2026-10-15-owner-id.surql`、Modify `CLAUDE.md`

- [ ] **Step 1: 移行スクリプト**

```sql
-- 2026-10-15: 所有者を data.user_id から CMS の owner_id に移し、mails.podcast_id を消す。
-- 新しい worker を起動する前に、CMS の SurrealDB (namespace = テナント ID) に対して 1 回流す。
FOR $c IN (SELECT id, name FROM collection WHERE name INSIDE ['podcasts', 'episodes', 'scripts', 'mails']) {
    LET $t = type::table(record::id($c.id));
    UPDATE $t SET owner_id = data.user_id, data.user_id = NONE WHERE data.user_id != NONE;
    IF $c.name = 'mails' {
        UPDATE $t SET data.podcast_id = NONE;
    };
};
```

ローカルの SurrealDB で試す: botcast-cms を旧 worker で seed → レコードを数件作る → スクリプトを流す → `SELECT owner_id, data FROM <table>` で `owner_id` が入り `user_id` が消えていること → 新 worker を起動して seed が 400 にならないこと。

- [ ] **Step 2: `CLAUDE.md`**
  - 「お便りエピソードの定期生成」の mails を `episode_id` / `radio_name` / `body`（所有者は CMS の `owner_id`）に
  - 「認証」に: レコードの所有者は CMS の `owner_id`（JWT の `sub`）。コレクションは `write: owner`。worker は `X-On-Behalf-Of` で番組の所有者としてエピソードを作る
  - スキーマは `crates/worker/src/usecase/collections.rs` が唯一の情報源で、worker の起動時に CMS に反映される、と追記

- [ ] **Step 3: PR** — `git push -u origin feat/cms-ownership && gh pr create --base main`。本文に「botcast-cms vX.Y.0 のリリース後、移行手順（下記）の後にマージ」と移行手順を記載

### Task 10: 本番移行（cdfy-dev）

ユーザーの確認を取ってから行う。

- [ ] 1. botcast-cms のリリースが Watchtower で反映されたことを確認（`ssh cdfy-dev docker ps --format '{{.Image}} {{.Status}}' | grep cms-api`）
- [ ] 2. `ssh cdfy-dev 'cd ~/botcast/deploy/selfhost && docker compose stop worker'`
- [ ] 3. SurrealDB をバックアップ: `ssh cdfy-dev 'cd ~/botcast-cms/deploy && docker compose exec -T surrealdb /surreal export --ns botcast --db <DATABASE_NAME> -u root -p <pass> --endpoint http://localhost:8000 -' > backup.surql`（DB 名・認証は `~/botcast-cms/deploy/.env` を見る。値はチャットに出さない）
- [ ] 4. 移行スクリプトを `surreal sql` で流す（同じ接続情報、`--ns botcast`）
- [ ] 5. botcast PR をマージ → GHCR の worker / web が更新され Watchtower が反映（または `docker compose up -d worker web`）
- [ ] 6. worker のログで `updating collection` が 4 回出て起動していること。CMS web で episode の `podcast_id` がリンクになること。botcast web で他人の番組に編集リンクが出ないこと

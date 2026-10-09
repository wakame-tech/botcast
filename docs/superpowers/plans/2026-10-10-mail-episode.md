# お便りエピソード定期生成 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 前回エピソード宛てのお便りを CMS のスクリプトで台本にし、番組ごとの cron で新しいエピソードを生成・音声化する。

**Architecture:** worker に `GenerateEpisode` ジョブ（CMS から番組・前回エピソード・お便り・スクリプトを取得 → CMS `/scripts` で台本生成 → エピソード作成 → 既存の音声生成）を追加する。番組の `schedule` を正本として `ScheduleSync` が 60 秒ごとに kafru のスケジュールへ同期する。web はお便りの投稿・一覧と、番組編集画面でのスケジュール設定を持つ。

**Tech Stack:** Rust (axum, kafru 1.0.4, rmcp, reqwest), botcast-cms REST, React + TanStack Router + openapi-react-query

設計書: `docs/superpowers/specs/2026-10-10-mail-episode-design.md`

---

## File Structure

| ファイル | 役割 |
|---|---|
| `crates/worker/src/usecase/cms_client.rs`（変更） | 一覧取得 `list_records`・取得 `get_record`・作成 `create_record`・スクリプト実行 `execute_script` を追加 |
| `crates/worker/src/usecase/episode_generation.rs`（新規） | 純粋関数（`record_view` / `build_script_code` / `parse_script_output`）と `EpisodeGenerationService` |
| `crates/worker/src/usecase/schedule_sync.rs`（新規） | 純粋関数（`parse_cron` / `desired_schedules` / `diff_schedules`）と `ScheduleSync` |
| `crates/worker/src/usecase/task_service.rs`（変更） | `Args::GenerateEpisode`・キュー名定数 `QUEUE_NAME` |
| `crates/worker/src/usecase/provider.rs`（変更） | `episode_generation_service()` |
| `crates/worker/src/mcp_server/mod.rs`（変更） | MCP ツール `generate_episode` |
| `crates/worker/src/worker.rs`（変更） | `start_schedule_sync` |
| `crates/worker/src/main.rs`（変更） | `mails` コレクション・`podcasts` スキーマの `schedule`・同期の起動 |
| `web/src/lib/cms_client.ts`（変更） | `Mail` 型・`recordToMail`・`PodcastSchedule` |
| `web/src/components/mail/EpisodeMails.tsx`（新規） | お便り投稿フォームと一覧 |
| `web/src/components/podcast/ScheduleForm.tsx`（新規） | 定期実行の設定フォーム |
| `web/src/routes/podcasts/$podcastId_/episodes/$episodeId.lazy.tsx`（変更） | `EpisodeMails` を表示 |
| `web/src/routes/podcasts/$podcastId_/edit.lazy.tsx`（変更） | 既存 `data` を保った更新・`ScheduleForm` |
| 削除 | `web/src/components/corner/CornerList.tsx`・`web/src/components/mail/MailList.tsx`・`web/src/components/mail/MailForm.tsx`・`web/src/components/task/EvaluateScriptForm.tsx` |

コマンドはすべてリポジトリルート `/Users/kmt/dev/botcast` から実行する。`cargo` の出力は rtk フックで要約されるため、テスト結果を正確に見るときは `rtk proxy cargo ...` を使う。

---

### Task 1: CmsClient に一覧・取得・作成・スクリプト実行を追加

**Files:**
- Modify: `crates/worker/src/usecase/cms_client.rs`

- [ ] **Step 1: 失敗するテストを書く**

`cms_client.rs` の `mod tests` に追加:

```rust
    #[test]
    fn list_query_to_params() {
        let q = ListQuery {
            filter: Some("podcast_id:eq:p1".to_string()),
            sort: Some("created_at".to_string()),
            order_desc: true,
            limit: Some(1),
        };
        assert_eq!(
            q.to_params(),
            vec![
                ("filter", "podcast_id:eq:p1".to_string()),
                ("sort", "created_at".to_string()),
                ("order", "desc".to_string()),
                ("limit", "1".to_string()),
            ]
        );
        assert!(ListQuery::default().to_params().is_empty());
    }
```

- [ ] **Step 2: テストが失敗することを確認**

Run: `rtk proxy cargo test -p worker list_query_to_params`
Expected: コンパイルエラー（`ListQuery` が無い）

- [ ] **Step 3: 実装**

`cms_client.rs` の `impl CmsClient` の前に追加:

```rust
/// `GET /records/{collection}` のクエリ
#[derive(Debug, Default, Clone)]
pub(crate) struct ListQuery {
    /// `<field>:<op>:<value>` 形式 (1 件のみ)
    pub filter: Option<String>,
    pub sort: Option<String>,
    pub order_desc: bool,
    pub limit: Option<u32>,
}

impl ListQuery {
    fn to_params(&self) -> Vec<(&'static str, String)> {
        let mut params = vec![];
        if let Some(f) = &self.filter {
            params.push(("filter", f.clone()));
        }
        if let Some(s) = &self.sort {
            params.push(("sort", s.clone()));
        }
        if self.order_desc {
            params.push(("order", "desc".to_string()));
        }
        if let Some(l) = self.limit {
            params.push(("limit", l.to_string()));
        }
        params
    }
}

/// `POST /scripts` の結果
#[derive(Debug, Clone)]
pub(crate) struct ScriptOutput {
    pub stdout: String,
    pub error: String,
}
```

`impl CmsClient` 内の `get_record_data` を次の 2 メソッドに置き換え、`create_record`・`list_records`・`execute_script` を追加する:

```rust
    /// レコード全体 (`id` / `data` / `owner_id` / ...) を取得する
    pub(crate) async fn get_record(
        &self,
        collection: &str,
        record_id: &str,
    ) -> anyhow::Result<serde_json::Value> {
        let cid = self.collection_id(collection).await?;
        Self::send(self.request(reqwest::Method::GET, &format!("/records/{cid}/{record_id}")))
            .await
    }

    /// レコードの `data` を取得する
    pub(crate) async fn get_record_data(
        &self,
        collection: &str,
        record_id: &str,
    ) -> anyhow::Result<serde_json::Value> {
        Ok(self.get_record(collection, record_id).await?["data"].clone())
    }

    /// レコード一覧 (レコード全体の配列) を取得する
    pub(crate) async fn list_records(
        &self,
        collection: &str,
        query: &ListQuery,
    ) -> anyhow::Result<Vec<serde_json::Value>> {
        let cid = self.collection_id(collection).await?;
        let res = Self::send(
            self.request(reqwest::Method::GET, &format!("/records/{cid}"))
                .query(&query.to_params()),
        )
        .await?;
        Ok(res.as_array().cloned().unwrap_or_default())
    }

    /// レコードを作成し、ID を返す
    pub(crate) async fn create_record(
        &self,
        collection: &str,
        data: serde_json::Value,
    ) -> anyhow::Result<String> {
        let cid = self.collection_id(collection).await?;
        let res = Self::send(
            self.request(reqwest::Method::POST, &format!("/records/{cid}"))
                .json(&serde_json::json!({ "data": data })),
        )
        .await?;
        res["id"]
            .as_str()
            .map(str::to_string)
            .context("id missing in create response")
    }

    /// Node.js のコードを CMS (Dify Sandbox) で実行する
    pub(crate) async fn execute_script(&self, code: &str) -> anyhow::Result<ScriptOutput> {
        let res = Self::send(
            self.request(reqwest::Method::POST, "/scripts")
                .json(&serde_json::json!({ "language": "nodejs", "code": code })),
        )
        .await?;
        Ok(ScriptOutput {
            stdout: res["data"]["stdout"].as_str().unwrap_or_default().to_string(),
            error: res["data"]["error"].as_str().unwrap_or_default().to_string(),
        })
    }
```

- [ ] **Step 4: テストが通ることを確認**

Run: `rtk proxy cargo test -p worker list_query_to_params`
Expected: `test result: ok. 1 passed`

- [ ] **Step 5: Commit**

```bash
git add crates/worker/src/usecase/cms_client.rs
git commit -m "feat(worker): CmsClient に一覧・作成・スクリプト実行を追加"
```

---

### Task 2: 台本生成の純粋関数

**Files:**
- Create: `crates/worker/src/usecase/episode_generation.rs`
- Modify: `crates/worker/src/usecase/mod.rs`

- [ ] **Step 1: 失敗するテストを書く**

`episode_generation.rs` を作成（テストのみ。実装は Step 3）:

```rust
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
```

`crates/worker/src/usecase/mod.rs` に `pub(crate) mod episode_generation;` を追加する。

- [ ] **Step 2: テストが失敗することを確認**

Run: `rtk proxy cargo test -p worker episode_generation`
Expected: コンパイルエラー（`record_view` などが無い）

- [ ] **Step 3: 実装**

`episode_generation.rs` の先頭（`#[cfg(test)]` の前）に追加:

```rust
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
```

- [ ] **Step 4: テストが通ることを確認**

Run: `rtk proxy cargo test -p worker episode_generation`
Expected: `test result: ok. 8 passed`

- [ ] **Step 5: Commit**

```bash
git add crates/worker/src/usecase/episode_generation.rs crates/worker/src/usecase/mod.rs
git commit -m "feat(worker): お便りエピソード生成の純粋関数を追加"
```

---

### Task 3: EpisodeGenerationService

**Files:**
- Modify: `crates/worker/src/usecase/episode_generation.rs`
- Modify: `crates/worker/src/usecase/provider.rs`

- [ ] **Step 1: 実装**

`episode_generation.rs` の純粋関数の後ろに追加（`use` は先頭にまとめる）:

```rust
use super::cms_client::{CmsClient, ListQuery};
use std::sync::Arc;
use tracing::instrument;

/// 番組のスクリプトを実行して新しいエピソードを作るサービス。
///
/// 前回エピソード (その番組で最新) 宛てのお便りを `context.mails` としてスクリプトに渡し、
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

        let by_podcast = ListQuery {
            filter: Some(format!("podcast_id:eq:{podcast_id}")),
            ..Default::default()
        };
        let episodes = self
            .cms
            .list_records(
                "episodes",
                &ListQuery {
                    sort: Some("created_at".to_string()),
                    order_desc: true,
                    ..by_podcast.clone()
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
            "arguments": script["data"].get("arguments").cloned().unwrap_or(serde_json::json!({})),
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
```

注: エピソード件数は前回エピソードの取得と同じ一覧（`limit` なし）から数える。

`provider.rs` の `impl Provider` に追加し、先頭に `use super::episode_generation::EpisodeGenerationService;` を足す:

```rust
    pub(crate) fn episode_generation_service(&self) -> EpisodeGenerationService {
        EpisodeGenerationService::new(self.cms.clone())
    }
```

- [ ] **Step 2: ビルド確認**

Run: `rtk proxy cargo check -p worker`
Expected: `Finished`（`EpisodeGenerationService` 未使用の dead_code 警告は Task 4 で解消）

- [ ] **Step 3: Commit**

```bash
git add crates/worker/src/usecase/episode_generation.rs crates/worker/src/usecase/provider.rs
git commit -m "feat(worker): スクリプトでエピソードを作る EpisodeGenerationService を追加"
```

---

### Task 4: GenerateEpisode ジョブと MCP ツール

**Files:**
- Modify: `crates/worker/src/usecase/task_service.rs`
- Modify: `crates/worker/src/mcp_server/mod.rs`

- [ ] **Step 1: 失敗するテストを書く**

`task_service.rs` の `mod tests` に追加:

```rust
    #[test]
    fn args_generate_episode_serializes_correctly() {
        let args = Args::GenerateEpisode { podcast_id: "p1".to_string() };
        let json = serde_json::to_value(&args).unwrap();
        assert_eq!(json["type"], "generateEpisode");
        assert_eq!(json["podcastId"], "p1");
    }
```

- [ ] **Step 2: テストが失敗することを確認**

Run: `rtk proxy cargo test -p worker args_generate_episode`
Expected: コンパイルエラー（`GenerateEpisode` が無い）

- [ ] **Step 3: 実装**

`task_service.rs`:

1. `Args` に追加:

```rust
    /// 番組のスクリプトで新しいエピソードを作り、音声まで生成する
    GenerateEpisode {
        podcast_id: String,
    },
```

2. `use` の下にキュー名定数を追加し、`create_task` と `list_jobs` の `"botcast-worker-default"` を `QUEUE_NAME` に置き換える:

```rust
/// worker が処理する kafru のキュー名 (`{server}-{queue}`)
pub(crate) const QUEUE_NAME: &str = "botcast-worker-default";
```

3. `TaskService` にフィールド `episode_generation_service: EpisodeGenerationService` を追加し、`new` の引数にも足す（`use super::episode_generation::EpisodeGenerationService;`）:

```rust
    pub(crate) fn new(
        episode_service: EpisodeService,
        episode_generation_service: EpisodeGenerationService,
        kafru_queue: Arc<Queue<'static>>,
    ) -> Self {
        Self {
            episode_service,
            episode_generation_service,
            kafru_queue,
        }
    }
```

4. `execute_args` の `match` に追加:

```rust
            Args::GenerateEpisode { podcast_id } => {
                let episode_id = self
                    .episode_generation_service
                    .generate(&podcast_id)
                    .await
                    .context("Failed to generate episode")
                    .map_err(Error::Other)?;
                let work_dir = use_work_dir(&Uuid::new_v4())
                    .context("Failed to create work dir")
                    .map_err(Error::Other)?;
                self.episode_service
                    .generate_audio(&work_dir, &episode_id)
                    .await
            }
```

`provider.rs` の `task_service()` を更新:

```rust
    pub(crate) fn task_service(&self) -> TaskService {
        TaskService::new(
            self.episode_service(),
            self.episode_generation_service(),
            self.kafru_queue.clone(),
        )
    }
```

`mcp_server/mod.rs`:

```rust
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct GenerateEpisodeParams {
    /// botcast-cms の podcasts レコード ID (schedule.script_id が必要)
    pub podcast_id: String,
}
```

`impl BotcastMcpServer` に追加:

```rust
    #[tool(description = "番組のスクリプトで前回エピソード宛てのお便りを読む新しいエピソードを作り、音声まで生成するジョブをエンキューします")]
    pub async fn generate_episode(&self, params: Parameters<GenerateEpisodeParams>) -> String {
        match self
            .provider
            .task_service()
            .create_task(crate::usecase::task_service::Args::GenerateEpisode {
                podcast_id: params.0.podcast_id,
            })
            .await
        {
            Ok(()) => serde_json::json!({ "status": "enqueued" }).to_string(),
            Err(e) => serde_json::json!({ "error": e.to_string() }).to_string(),
        }
    }
```

- [ ] **Step 4: テストが通ることを確認**

Run: `rtk proxy cargo test -p worker`
Expected: すべて ok（新しいテストを含む）

- [ ] **Step 5: Commit**

```bash
git add crates/worker/src
git commit -m "feat(worker): GenerateEpisode ジョブと MCP ツール generate_episode を追加"
```

---

### Task 5: スケジュール同期の純粋関数

**Files:**
- Create: `crates/worker/src/usecase/schedule_sync.rs`
- Modify: `crates/worker/src/usecase/mod.rs`

- [ ] **Step 1: 失敗するテストを書く**

`schedule_sync.rs` を作成（テストのみ）:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn parse_cron_accepts_seven_fields() {
        let cron = parse_cron("0 0 12 * * Fri *").unwrap();
        assert_eq!(cron.get_expression(), "0 0 12 * * Fri *");
    }

    #[test]
    fn parse_cron_rejects_wrong_field_count() {
        assert!(parse_cron("0 12 * * Fri").is_err());
    }

    fn podcast(id: &str, schedule: serde_json::Value) -> serde_json::Value {
        json!({ "id": id, "data": { "title": "t", "schedule": schedule } })
    }

    #[test]
    fn desired_schedules_picks_enabled_valid_ones() {
        let podcasts = vec![
            podcast("a", json!({ "cron": "0 0 12 * * Fri *", "script_id": "s", "enabled": true })),
            podcast("b", json!({ "cron": "0 0 12 * * Fri *", "script_id": "s", "enabled": false })),
            podcast("c", json!({ "cron": "bad", "script_id": "s", "enabled": true })),
            podcast("d", json!({ "cron": "0 0 12 * * Fri *", "enabled": true })),
            json!({ "id": "e", "data": { "title": "t" } }),
        ];
        assert_eq!(
            desired_schedules(&podcasts),
            vec![DesiredSchedule {
                name: "podcast:a".to_string(),
                podcast_id: "a".to_string(),
                cron: "0 0 12 * * Fri *".to_string(),
            }]
        );
    }

    fn desired(id: &str, cron: &str) -> DesiredSchedule {
        DesiredSchedule {
            name: schedule_name(id),
            podcast_id: id.to_string(),
            cron: cron.to_string(),
        }
    }

    fn current(id: &str, cron: &str) -> CurrentSchedule {
        CurrentSchedule {
            id: format!("kafru_schedules:{id}"),
            name: schedule_name(id),
            cron: cron.to_string(),
        }
    }

    #[test]
    fn diff_schedules_creates_recreates_and_removes() {
        let changes = diff_schedules(
            &[
                desired("new", "0 0 12 * * Fri *"),
                desired("same", "0 0 12 * * Fri *"),
                desired("changed", "0 0 13 * * Fri *"),
            ],
            &[
                current("same", "0 0 12 * * Fri *"),
                current("changed", "0 0 12 * * Fri *"),
                current("gone", "0 0 12 * * Fri *"),
            ],
        );
        assert_eq!(
            changes,
            vec![
                ScheduleChange::Create(desired("new", "0 0 12 * * Fri *")),
                ScheduleChange::Recreate {
                    id: "kafru_schedules:changed".to_string(),
                    desired: desired("changed", "0 0 13 * * Fri *"),
                },
                ScheduleChange::Remove { id: "kafru_schedules:gone".to_string() },
            ]
        );
    }
}
```

`crates/worker/src/usecase/mod.rs` に `pub(crate) mod schedule_sync;` を追加する。

- [ ] **Step 2: テストが失敗することを確認**

Run: `rtk proxy cargo test -p worker schedule_sync`
Expected: コンパイルエラー

- [ ] **Step 3: 実装**

`schedule_sync.rs` の先頭に追加:

```rust
use anyhow::bail;
use kafru::cron_schedule::CronSchedule;
use serde_json::Value;

/// 番組ごとの kafru スケジュール名
pub(crate) fn schedule_name(podcast_id: &str) -> String {
    format!("podcast:{podcast_id}")
}

/// Quartz 形式 7 フィールド (`秒 分 時 日 月 曜日 年`) の cron を `CronSchedule` にする
pub(crate) fn parse_cron(expr: &str) -> anyhow::Result<CronSchedule> {
    let f: Vec<&str> = expr.split_whitespace().collect();
    if f.len() != 7 {
        bail!("cron must have 7 fields (sec min hour day month weekday year): {expr}");
    }
    let mut cron = CronSchedule::new();
    cron.set_second(f[0].to_string());
    cron.set_minute(f[1].to_string());
    cron.set_hour(f[2].to_string());
    cron.set_day_of_month(f[3].to_string());
    cron.set_month(f[4].to_string());
    cron.set_day_of_week(f[5].to_string());
    cron.set_year(f[6].to_string());
    // 次回時刻を計算できない式 (不正なフィールド値) は弾く
    cron.clone()
        .get_upcoming(None)
        .map_err(|e| anyhow::anyhow!("invalid cron {expr}: {e}"))?;
    Ok(cron)
}

/// CMS の番組設定から求められるスケジュール
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct DesiredSchedule {
    pub name: String,
    pub podcast_id: String,
    /// 正規化した cron (`CronSchedule::get_expression`)
    pub cron: String,
}

/// kafru に登録済みのスケジュール
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct CurrentSchedule {
    pub id: String,
    pub name: String,
    pub cron: String,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum ScheduleChange {
    Create(DesiredSchedule),
    Recreate { id: String, desired: DesiredSchedule },
    Remove { id: String },
}

/// `schedule.enabled` が true で `cron`・`script_id` が正しい番組のスケジュールを返す
pub(crate) fn desired_schedules(podcasts: &[Value]) -> Vec<DesiredSchedule> {
    podcasts
        .iter()
        .filter_map(|p| {
            let id = p["id"].as_str()?;
            let schedule = &p["data"]["schedule"];
            if schedule["enabled"].as_bool() != Some(true) || schedule["script_id"].as_str().is_none() {
                return None;
            }
            let cron = schedule["cron"].as_str()?;
            match parse_cron(cron) {
                Ok(c) => Some(DesiredSchedule {
                    name: schedule_name(id),
                    podcast_id: id.to_string(),
                    cron: c.get_expression(),
                }),
                Err(e) => {
                    tracing::warn!("skip schedule of podcast {id}: {e}");
                    None
                }
            }
        })
        .collect()
}

/// 求められるスケジュールと登録済みのスケジュールの差分
pub(crate) fn diff_schedules(
    desired: &[DesiredSchedule],
    current: &[CurrentSchedule],
) -> Vec<ScheduleChange> {
    let mut changes = vec![];
    for d in desired {
        match current.iter().find(|c| c.name == d.name) {
            None => changes.push(ScheduleChange::Create(d.clone())),
            Some(c) if c.cron != d.cron => changes.push(ScheduleChange::Recreate {
                id: c.id.clone(),
                desired: d.clone(),
            }),
            Some(_) => {}
        }
    }
    for c in current {
        if !desired.iter().any(|d| d.name == c.name) {
            changes.push(ScheduleChange::Remove { id: c.id.clone() });
        }
    }
    changes
}
```

- [ ] **Step 4: テストが通ることを確認**

Run: `rtk proxy cargo test -p worker schedule_sync`
Expected: `test result: ok. 4 passed`

`parse_cron_accepts_seven_fields` が `get_expression` の書式違いで落ちた場合は、`get_expression()` の実際の出力をテストの期待値にする（kafru の `get_expression` は 7 フィールドをスペース区切りで連結する）。

- [ ] **Step 5: Commit**

```bash
git add crates/worker/src/usecase/schedule_sync.rs crates/worker/src/usecase/mod.rs
git commit -m "feat(worker): スケジュール同期の差分計算を追加"
```

---

### Task 6: ScheduleSync の実行と起動

**Files:**
- Modify: `crates/worker/src/usecase/schedule_sync.rs`
- Modify: `crates/worker/src/worker.rs`
- Modify: `crates/worker/src/main.rs`

- [ ] **Step 1: 実装**

`schedule_sync.rs` の純粋関数の後ろに追加（`use` は先頭にまとめる）:

```rust
use super::cms_client::{CmsClient, ListQuery};
use super::task_service::{Args, QUEUE_NAME};
use chrono::{Duration, Utc};
use kafru::database::Db;
use kafru::schedule::{Schedule, ScheduleData, ScheduleListConditions, ScheduleStatus};
use kafru::task::RecordId;
use std::{collections::HashMap, sync::Arc};

const HANDLER: &str = "execute_task";
const SYNC_INTERVAL_SECS: u64 = 60;

/// CMS の `podcasts.data.schedule` を kafru のスケジュールへ同期する
pub(crate) struct ScheduleSync {
    cms: Arc<CmsClient>,
    schedule: Schedule<'static>,
}

impl ScheduleSync {
    pub(crate) async fn new(cms: Arc<CmsClient>, db: Arc<Db>) -> Self {
        Self {
            cms,
            schedule: Schedule::new(Some(db)).await,
        }
    }

    async fn current(&self) -> anyhow::Result<Vec<CurrentSchedule>> {
        let list = self
            .schedule
            .list(ScheduleListConditions {
                handler: Some(vec![HANDLER.to_string()]),
                upcoming: None,
                ..Default::default()
            })
            .await
            .map_err(anyhow::Error::msg)?;
        Ok(list
            .into_iter()
            .filter_map(|s| {
                let name = s.name?;
                if !name.starts_with("podcast:") {
                    return None;
                }
                Some(CurrentSchedule {
                    id: s.id?.to_string(),
                    name,
                    cron: s.cron_expression?.get_expression(),
                })
            })
            .collect())
    }

    async fn create(&self, d: &DesiredSchedule) -> anyhow::Result<()> {
        let args = serde_json::to_value(Args::GenerateEpisode {
            podcast_id: d.podcast_id.clone(),
        })?;
        self.schedule
            .create(ScheduleData {
                name: Some(d.name.clone()),
                queue: Some(QUEUE_NAME.to_string()),
                cron_expression: Some(parse_cron(&d.cron)?),
                handler: Some(HANDLER.to_string()),
                parameters: Some(HashMap::from([("args".to_string(), args)])),
                status: Some(ScheduleStatus::Enabled),
                // kafru は until_schedule >= 現在 のスケジュールだけを発火させる
                until_schedule: Some(Utc::now() + Duration::days(365 * 100)),
                ..Default::default()
            })
            .await
            .map_err(anyhow::Error::msg)?;
        Ok(())
    }

    async fn remove(&self, id: &str) -> anyhow::Result<()> {
        let id: RecordId = id.parse().map_err(|e| anyhow::anyhow!("invalid schedule id {id}: {e}"))?;
        self.schedule.remove(id).await.map_err(anyhow::Error::msg)?;
        Ok(())
    }

    /// 1 回分の同期
    pub(crate) async fn sync_once(&self) -> anyhow::Result<()> {
        let podcasts = self.cms.list_records("podcasts", &ListQuery::default()).await?;
        let changes = diff_schedules(&desired_schedules(&podcasts), &self.current().await?);
        for change in changes {
            tracing::info!("schedule change: {change:?}");
            match change {
                ScheduleChange::Create(d) => self.create(&d).await?,
                ScheduleChange::Recreate { id, desired } => {
                    self.remove(&id).await?;
                    self.create(&desired).await?;
                }
                ScheduleChange::Remove { id } => self.remove(&id).await?,
            }
        }
        Ok(())
    }

    /// 起動時と、その後 60 秒ごとに同期する
    pub(crate) async fn run(self) {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(SYNC_INTERVAL_SECS));
        loop {
            interval.tick().await;
            if let Err(e) = self.sync_once().await {
                tracing::warn!("schedule sync failed: {e:#}");
            }
        }
    }
}
```

`worker.rs` に追加:

```rust
/// CMS の番組スケジュールを kafru に同期するタスクを起動する
pub fn start_schedule_sync(provider: Arc<Provider>, kafru_db: Arc<Db>) {
    tokio::spawn(async move {
        let sync = crate::usecase::schedule_sync::ScheduleSync::new(provider.cms.clone(), kafru_db).await;
        sync.run().await;
    });
}
```

`main.rs`:

1. `use` に `start_schedule_sync` を追加（`start_worker` と同じ行）
2. `start_worker(provider.clone(), kafru_db);` を次に置き換える:

```rust
    start_worker(provider.clone(), kafru_db.clone());
    start_schedule_sync(provider.clone(), kafru_db);
```

- [ ] **Step 2: ビルド確認**

Run: `rtk proxy cargo check -p worker` / `rtk proxy cargo test -p worker`
Expected: 警告なしで `Finished`、テストすべて ok

`kafru::task::RecordId` の `parse` が使えない場合は、`task_service.rs` の `get_job_status` と同じ書き方（`job_id.parse()`）に揃える。

- [ ] **Step 3: Commit**

```bash
git add crates/worker/src
git commit -m "feat(worker): 番組スケジュールを kafru に同期する ScheduleSync を追加"
```

---

### Task 7: mails コレクションと podcasts の schedule

**Files:**
- Modify: `crates/worker/src/main.rs`（`seed_cms_collections`）

- [ ] **Step 1: 実装**

`podcasts_schema` の `properties` に追加:

```rust
            "user_id": { "type": "string" },
            "schedule": {
                "type": "object",
                "properties": {
                    "cron": { "type": "string" },
                    "script_id": { "type": "string" },
                    "enabled": { "type": "boolean" }
                }
            }
```

`scripts_schema` の後に追加し、`ensure_collection` の呼び出しにも足す:

```rust
    let mails_schema = serde_json::json!({
        "type": "object",
        "properties": {
            "podcast_id": { "type": "string" },
            "episode_id": { "type": "string" },
            "radio_name": { "type": "string" },
            "body": { "type": "string" },
            "user_id": { "type": "string" }
        },
        "required": ["podcast_id", "episode_id", "radio_name", "body", "user_id"]
    });
```

```rust
    client.ensure_collection("mails", mails_schema).await?;
```

- [ ] **Step 2: ビルド確認**

Run: `rtk proxy cargo check -p worker`
Expected: `Finished`

- [ ] **Step 3: Commit**

```bash
git add crates/worker/src/main.rs
git commit -m "feat(worker): mails コレクションと podcasts.schedule を初期化処理に追加"
```

---

### Task 8: web — お便りの投稿と一覧

**Files:**
- Modify: `web/src/lib/cms_client.ts`
- Create: `web/src/components/mail/EpisodeMails.tsx`
- Modify: `web/src/routes/podcasts/$podcastId_/episodes/$episodeId.lazy.tsx`

- [ ] **Step 1: 型と変換を追加**

`cms_client.ts` の `Script` インターフェースの後に追加:

```ts
export interface Mail {
	id: string;
	podcast_id: string;
	episode_id: string;
	radio_name: string;
	body: string;
	user_id: string;
}

export function recordToMail(record: CmsRecord): Mail {
	const d = record.data as Record<string, unknown>;
	return {
		id: record.id,
		podcast_id: d.podcast_id as string,
		episode_id: d.episode_id as string,
		radio_name: (d.radio_name as string) ?? "",
		body: (d.body as string) ?? "",
		user_id: d.user_id as string,
	};
}
```

- [ ] **Step 2: コンポーネントを作成**

`web/src/components/mail/EpisodeMails.tsx`:

```tsx
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Textarea } from "@/components/ui/textarea";
import { useSession } from "@/hooks/useSession";
import { $cms, recordToMail, toRecordData } from "@/lib/cms_client";
import { useState } from "react";

interface EpisodeMailsProps {
	podcastId: string;
	episodeId: string;
}

/** エピソード宛てのお便り一覧と投稿フォーム */
export function EpisodeMails({ podcastId, episodeId }: EpisodeMailsProps) {
	const { userId } = useSession();
	const [radioName, setRadioName] = useState("");
	const [body, setBody] = useState("");
	const listParams = {
		params: {
			path: { collectionId: "mails" },
			query: { filter: [`episode_id:eq:${episodeId}`] },
		},
	};
	const { data: records, refetch } = $cms.useQuery(
		"get",
		"/records/{collectionId}",
		listParams,
	);
	const createMail = $cms.useMutation("post", "/records/{collectionId}");
	const mails = (records ?? []).map(recordToMail);

	const handleSubmit = async (e: React.FormEvent) => {
		e.preventDefault();
		await createMail.mutateAsync({
			params: { path: { collectionId: "mails" } },
			body: {
				data: toRecordData({
					podcast_id: podcastId,
					episode_id: episodeId,
					radio_name: radioName,
					body,
					user_id: userId ?? "",
				}),
			},
		});
		setBody("");
		await refetch();
	};

	return (
		<div className="py-4">
			<h2 className="text-lg font-bold">お便り ({mails.length})</h2>
			<ul className="p-0 list-none">
				{mails.map((mail) => (
					<li key={mail.id} className="border rounded p-2 my-2">
						<p className="text-sm font-bold">{mail.radio_name}</p>
						<p className="whitespace-pre-wrap">{mail.body}</p>
					</li>
				))}
			</ul>
			<form onSubmit={handleSubmit} className="flex flex-col gap-2">
				<Input
					placeholder="ラジオネーム"
					value={radioName}
					onChange={(e) => setRadioName(e.target.value)}
					required
				/>
				<Textarea
					placeholder="この回へのお便り"
					value={body}
					onChange={(e) => setBody(e.target.value)}
					required
				/>
				<Button type="submit" disabled={createMail.isPending}>
					この回にお便りを送る
				</Button>
			</form>
		</div>
	);
}
```

- [ ] **Step 3: エピソード画面に表示**

`$episodeId.lazy.tsx` に `import { EpisodeMails } from "@/components/mail/EpisodeMails";` を追加し、`</Card>` の直後（`</>` の前）に追加:

```tsx
			<EpisodeMails podcastId={podcastId} episodeId={episodeId} />
```

- [ ] **Step 4: 型チェック・lint**

Run: `cd web && npx tsc && ./node_modules/.bin/biome check --write src`
Expected: エラーなし

`query.filter` の型が合わない場合は、`cms_api.d.ts` の `/records/{collectionId}` の `get.parameters.query.filter` の型（`string[]`）に合わせる。

- [ ] **Step 5: Commit**

```bash
git add web/src
git commit -m "feat(web): エピソードへのお便り投稿と一覧を追加"
```

---

### Task 9: web — 定期実行の設定と古い部品の削除

**Files:**
- Modify: `web/src/lib/cms_client.ts`
- Create: `web/src/components/podcast/ScheduleForm.tsx`
- Modify: `web/src/routes/podcasts/$podcastId_/edit.lazy.tsx`
- Delete: `web/src/components/corner/CornerList.tsx`, `web/src/components/mail/MailList.tsx`, `web/src/components/mail/MailForm.tsx`, `web/src/components/task/EvaluateScriptForm.tsx`
- Modify: `web/src/lib/api_client.ts`（`CornerInputSchema` / `CornerInput` / `MailInputSchema` / `MailInput` を削除）

- [ ] **Step 1: 型を追加**

`cms_client.ts` の `Podcast` インターフェースに `schedule?: PodcastSchedule;` を追加し、その前に定義:

```ts
export interface PodcastSchedule {
	/** Quartz 形式 7 フィールド (秒 分 時 日 月 曜日 年)、UTC */
	cron: string;
	script_id: string;
	enabled: boolean;
}
```

`recordToPodcast` に `schedule: d.schedule as PodcastSchedule | undefined,` を追加する。

- [ ] **Step 2: フォームを作成**

`web/src/components/podcast/ScheduleForm.tsx`:

```tsx
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { $cms, type PodcastSchedule, recordToScript } from "@/lib/cms_client";
import { useState } from "react";

interface ScheduleFormProps {
	value?: PodcastSchedule;
	onSubmit: (schedule: PodcastSchedule) => void;
}

/** 番組の定期実行 (お便りエピソードの自動生成) の設定 */
export function ScheduleForm({ value, onSubmit }: ScheduleFormProps) {
	const [cron, setCron] = useState(value?.cron ?? "0 0 12 * * Fri *");
	const [scriptId, setScriptId] = useState(value?.script_id ?? "");
	const [enabled, setEnabled] = useState(value?.enabled ?? false);
	const { data: records } = $cms.useQuery("get", "/records/{collectionId}", {
		params: { path: { collectionId: "scripts" } },
	});
	const scripts = (records ?? []).map(recordToScript);

	return (
		<form
			className="flex flex-col gap-2 py-4"
			onSubmit={(e) => {
				e.preventDefault();
				onSubmit({ cron, script_id: scriptId, enabled });
			}}
		>
			<h2 className="text-lg font-bold">定期実行</h2>
			<label className="text-sm">
				cron (秒 分 時 日 月 曜日 年、UTC)
				<Input value={cron} onChange={(e) => setCron(e.target.value)} />
			</label>
			<label className="text-sm">
				スクリプト
				<select
					className="block border rounded px-2 py-1"
					value={scriptId}
					onChange={(e) => setScriptId(e.target.value)}
					required={enabled}
				>
					<option value="">選択してください</option>
					{scripts.map((s) => (
						<option key={s.id} value={s.id}>
							{s.title}
						</option>
					))}
				</select>
			</label>
			<label className="text-sm flex items-center gap-2">
				<input
					type="checkbox"
					checked={enabled}
					onChange={(e) => setEnabled(e.target.checked)}
				/>
				有効にする
			</label>
			<Button type="submit">定期実行を保存</Button>
		</form>
	);
}
```

- [ ] **Step 3: 編集画面を更新**

`edit.lazy.tsx`:

1. `import { ScheduleForm } from "@/components/podcast/ScheduleForm";` と、`PodcastSchedule` の型 import を追加
2. 既存 `data` を保って更新する関数を追加し、`handleSubmit` をそれで書き直す:

```tsx
	const saveData = async (patch: Record<string, unknown>) => {
		const current = (podcastRecord?.data ?? {}) as Record<string, unknown>;
		await updatePodcast.mutateAsync({
			params: { path: { collectionId: "podcasts", recordId: podcastId } },
			body: { data: toRecordData({ ...current, ...patch }) },
		});
	};

	const handleSubmit = async (values: PodcastInput) => {
		await saveData({
			icon: values.icon,
			title: values.title,
			description: values.description,
		});
		navigate({ to: "/podcasts/$podcastId", params: { podcastId } });
	};

	const handleScheduleSubmit = async (schedule: PodcastSchedule) => {
		await saveData({ schedule });
		navigate({ to: "/podcasts/$podcastId", params: { podcastId } });
	};
```

3. `<PodcastForm ... />` の直後に追加:

```tsx
			<ScheduleForm
				value={podcast.schedule}
				onSubmit={handleScheduleSubmit}
			/>
```

- [ ] **Step 4: 古い部品を削除**

```bash
git rm web/src/components/corner/CornerList.tsx web/src/components/mail/MailList.tsx web/src/components/mail/MailForm.tsx web/src/components/task/EvaluateScriptForm.tsx
```

`web/src/lib/api_client.ts` から `CornerInputSchema`・`CornerInput`・`MailInputSchema`・`MailInput` を削除する。

- [ ] **Step 5: 型チェック・lint**

Run: `cd web && npx tsc && ./node_modules/.bin/biome check --write src`
Expected: エラーなし（削除した部品を参照している箇所が無いこと）

- [ ] **Step 6: Commit**

```bash
git add -A web/src
git commit -m "feat(web): 番組の定期実行設定を追加し、使われていない Corner/Mail 部品を削除"
```

---

### Task 10: ドキュメント

**Files:**
- Modify: `CLAUDE.md`
- Modify: `crates/worker/src/lib.rs`

- [ ] **Step 1: 追記**

`CLAUDE.md` の「## 音声ファイル」の後に追加:

```markdown
## お便りエピソードの定期生成

- お便りは CMS の `mails`（`podcast_id` / `episode_id` / `radio_name` / `body` / `user_id`）。web のエピソード画面から投稿する
- 番組の `data.schedule = { cron, script_id, enabled }`（cron は Quartz 7 フィールド・UTC）を worker の `ScheduleSync` が 60 秒ごとに kafru のスケジュールへ同期する
- 発火すると `GenerateEpisode` ジョブがスクリプト（Node.js）を CMS `/scripts` で実行する。スクリプトは `context = { podcast, previous_episode, mails, arguments }` を受け取り、`{ title?, sections }` を JSON で標準出力する
- 手動実行は worker の MCP ツール `generate_episode(podcast_id)`
```

`crates/worker/src/lib.rs` の「## 機能」に追加:

```rust
//! - 番組のスクリプトで前回エピソード宛てのお便りを読む新しいエピソードを定期生成 (`GenerateEpisode`・`ScheduleSync`)
```

- [ ] **Step 2: rustdoc 確認**

Run: `RUSTDOCFLAGS="-D warnings" rtk proxy cargo doc --no-deps -p worker`
Expected: 警告なしで `Generated`

- [ ] **Step 3: Commit**

```bash
git add CLAUDE.md crates/worker/src/lib.rs
git commit -m "docs: お便りエピソードの定期生成を追記"
```

---

### Task 11: ローカルでの通し確認

前回の音声生成の確認と同じ構成を使う（botcast-cms api :3002・VOICEVOX :50021・kafru SurrealDB :4030・worker :9001・web :5199）。

- [ ] **Step 1: サービスを起動**

```bash
cd /Users/kmt/dev/botcast-cms && docker compose up -d surrealdb
docker start botcast-voicevox
cd /Users/kmt/dev/botcast && just up
# botcast-cms api: cd /Users/kmt/dev/botcast-cms/cms && ./target/debug/api (バックグラウンド)
# worker: 前回と同じ環境変数 (API_KEY = botcast-cms/cms/.env の API_KEY) で ./target/debug/worker (バックグラウンド)
# web: cd web && npx vite --port 5199 --strictPort (バックグラウンド)
```

worker のログに `collection 'mails'` の作成が出ることを確認する。

- [ ] **Step 2: スクリプトを登録し、番組に紐づける**

テストユーザーの JWT で `scripts` にレコードを作る（`template` は設計書の例のスクリプト、`arguments: {}`）。ダミー番組「ずんだもんラジオ」（`fo67rv5rgddzt0eabpdv`）の編集画面で、スクリプトを選び cron を設定して保存する（有効のチェックは Step 4 で入れる）。

- [ ] **Step 3: お便りを投稿し、手動で生成**

web で「第1回 はじめまして」に 2 件のお便りを投稿する。worker の MCP で `generate_episode`（`{"podcast_id":"fo67rv5rgddzt0eabpdv"}`）を呼び、ジョブが Completed になること、新しいエピソードの `sections` にお便りの本文が含まれ `audio_url` が付くことを確認する。

- [ ] **Step 4: スケジュールで生成**

番組の定期実行を「有効」にし、cron を 2〜3 分後の時刻（UTC）にして保存する。60 秒以内に worker のログに `schedule change: Create` が出て、指定時刻にジョブが投入され、新しいエピソードができることを確認する。確認後、定期実行を無効に戻し、`schedule change: Remove` が出ることを確認する。

- [ ] **Step 5: 片付け**

起動したサービスを止める。

---

## Self-Review メモ

- 設計書の各項目 → タスク: データ構造 (7, 9)・ジョブ (1〜4)・同期 (5, 6)・MCP (4)・web (8, 9)・テスト (1, 2, 4, 5, 11)・ドキュメント (10)
- エラー時の扱い: スクリプト失敗・JSON 不正は `parse_script_output` (Task 2)、CMS エラーは `CmsClient::send` が返す。同期失敗は `run` でログのみ (Task 6)
- 型の一貫性: `ListQuery` / `ScriptOutput` (Task 1)、`GeneratedEpisode` (Task 2)、`Args::GenerateEpisode { podcast_id }` (Task 4)、`DesiredSchedule` / `CurrentSchedule` / `ScheduleChange` (Task 5) を後続タスクで同名で使う

---
id: "01KNM2BBT5YGP4741F2QK6JNKE"
name: "Kafruベースのジョブキューでバックグラウンドタスクを実行できる"
status: "stable"
---

## 関連ファイル

- `cms/crates/worker/src/lib.rs`
- `cms/crates/worker/src/jobs/mod.rs`
- `cms/crates/worker/src/jobs/test_job.rs`
- `cms/crates/api/src/openapi_adapter.rs`
- `cms/crates/openapi-gen/src/apis/job.rs`
- `spec/main.tsp`

## 機能概要

`POST /jobs` でジョブをエンキューし、`GET /jobs` でジョブ一覧とステータスを取得する。BotcastWorker が Kafru キューからジョブを取り出し、登録された TaskHandler で非同期実行する。ジョブステータスは waiting → in_progress → completed/error と遷移する。

## 意図

重い処理や非同期タスクをバックグラウンドで実行できる仕組みを提供し、API の応答性を維持する。

## 主要なメンバー

- `BotcastWorker { server, task_registry, queue, db }` — ワーカープロセス
- `TaskHandler` トレイト — ジョブ実行ハンドラの抽象化
- `TaskRegistry` — ジョブ名と TaskHandler の対応を管理
- `BotcastJob { name, params, status }` — ジョブエンティティ
- `TestJob` — テスト用ジョブハンドラ実装

## シナリオ

### ジョブをエンキューする

1. クライアントが `POST /jobs` にジョブ名とパラメータを送信する
2. BotcastWorker がジョブを Kafru キューに追加する
3. ジョブは `waiting` ステータスで登録される
4. 204 No Content を返す

### ワーカーがジョブを実行する

1. BotcastWorker がキューからジョブを取り出す
2. ステータスを `in_progress` に更新する
3. TaskRegistry から対応する TaskHandler を取得する
4. `TaskHandler::run()` を実行する
5. 成功時はステータスを `completed` に、失敗時は `error` に更新する

### ジョブ一覧を取得する

1. クライアントが `GET /jobs` を送信する
2. Kafru キューから全ジョブを取得する
3. 200 OK でジョブ配列（名前・パラメータ・ステータス）を返す

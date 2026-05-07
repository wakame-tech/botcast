# Job 実行アーキテクチャ改善設計

**日付**: 2026-05-07  
**ステータス**: 承認済み

## 背景と課題

現状、音声生成 job の実行ロジックは botcast 側にあるが、job 投入・管理 API が botcast-cms 側に分散している。具体的な問題点：

- `BotcastWorker`（kafru worker）が botcast-cms に存在するが、`testjob` しか登録されておらず実質未完成
- botcast と botcast-cms が同じ kafru DB・同じキュー名 `"botcast-worker-default"` を共有しており責務が曖昧
- Dify Sandbox スクリプト実行（`POST /scripts`）が同期 HTTP で実装されており、job として管理できない
- botcast の GenerateScript 進捗を CMS から監視できない
- ARCHITECTURE.md の "MCP ファースト" 原則に反し、job 投入が HTTP API (`POST /createTask`) 経由のみ

## 設計方針

**job 実行はビジネスロジックであるため botcast に集約する。**  
CMS の同期スクリプト実行も、botcast の job 実行中に MCP ツールとして呼び出すことで自然に包含できる。

## ターゲットアーキテクチャ

### botcast-cms：純粋なデータストア + MCP サーバー

```
botcast-cms
├── Collection / Record API（データ CRUD）
├── Script API（Dify Sandbox、同期実行のまま）
└── MCP Server: CollectionApi_* / RecordApi_* / ScriptApi_execute
```

**削除するもの：**
- `cms/crates/worker/`（`BotcastWorker`、kafru 依存）
- Job API（`GET /jobs`、`POST /jobs`）
- OpenAPI 仕様の Job リソース定義
- `cms/crates/api/src/openapi_adapter.rs` の Job 実装

### botcast：ビジネスロジック実行エンジン

```
botcast
├── MCP Server（新規追加）
│     generate_audio {episode_id}        → job エンキュー → {job_id}
│     generate_script {episode_id, prompt} → job エンキュー → {job_id}
│     list_jobs                          → [{job_id, name, status, ...}]
│     get_job_status {job_id}            → {status, ...}
├── kafru Worker（既存・拡張）
│     GenerateAudio, GenerateScript, ...
│     └── job 実行中に CMS MCP ツールを呼べる（ScriptApi_execute 含む）
└── HTTP API（/createTask は廃止 or MCP サーバー起動後に廃止）
```

**MCP サーバートランスポート：** HTTP/SSE  
既存の HTTP サーバーと同じプロセスで動作し、`rmcp` を使用して実装する。

### LLM Agent のフロー

```
LLM Agent
├── (MCP) botcast-cms → データ操作（Collection / Record / Script）
└── (MCP) botcast     → job 投入・状態確認

job 実行中（botcast kafru worker）:
  └── (MCP) botcast-cms → ScriptApi_execute（Dify スクリプト）
                        → RecordApi_update（結果の書き戻し）
```

### フロントエンド

```
Frontend
├── $cms → botcast-cms（CMS CRUD）
└── $api → botcast（認証 + job 投入 / 一覧）
```

## 変更サマリ

| コンポーネント | 変更 |
|---|---|
| botcast-cms | `BotcastWorker` 削除・Job API 削除・OpenAPI spec から Job 定義削除 |
| botcast | MCP サーバー追加（`generate_audio` / `generate_script` / `list_jobs` / `get_job_status`）、Job 一覧 HTTP エンドポイント追加 |
| botcast-cms MCP Server | 変更なし（`ScriptApi_execute` は botcast job から呼ばれる） |
| LLM Agent 設定 | botcast MCP サーバーを追加接続 |
| フロントエンド | job 投入・一覧を `$api`（botcast）経由に統一 |

## 実装ステップ（概要）

1. botcast-cms から Job API と BotcastWorker を削除
2. botcast に `rmcp` MCP サーバーを追加
3. `generate_audio` / `generate_script` / `list_jobs` / `get_job_status` ツールを実装
4. botcast HTTP API に job 一覧エンドポイント追加（フロントエンド用）
5. botcast MCP サーバーの SSE エンドポイントを既存 HTTP サーバーに統合
6. フロントエンドの job 投入を botcast API に変更
7. `POST /createTask` を廃止（フロントエンドが job 系 API を使うようになった後）

## 非機能要件

- botcast-cms の Script API（Dify Sandbox）は同期実行のまま維持。job 化は不要（botcast の job 実行中に MCP 経由で呼ぶため）
- kafru DB は botcast のみが保持し、botcast-cms との共有を解消する

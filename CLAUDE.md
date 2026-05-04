# CLAUDE.md

This file provides guidance to AI assistants when working with code in this repository.

## Response Language

**すべてのレスポンスは日本語で行うこと。**

## Project Overview

Botcast はポッドキャスト自動生成システムのモノレポ。Rust バックエンドと TypeScript/React フロントエンドで構成される。

### コンポーネント一覧

| ディレクトリ | 技術 | 役割 |
|---|---|---|
| `crates/api` | Rust / Axum / SurrealDB | CMS REST API（ポート 3002） |
| `crates/worker` | Rust / kafru / VoiceVox | バックグラウンドタスク処理 |
| `crates/audio_generator` | Rust / FFmpeg | 音声合成・処理 |
| `crates/repos` | Rust / SeaORM | PostgreSQL アクセス層 |
| `crates/openapi-gen` | Rust / rust-axum | TypeSpec から生成された API スキーマ |
| `frontend/` | React / TanStack Router / UnoCSS | ユーザー向けフロントエンド |
| `cms-dashboard/` | React / Vite | CMS 管理ダッシュボード |
| `spec/` | TypeSpec | OpenAPI YAML 生成 |
| `mcp/` | Node.js | MCP サーバー |

## Development Commands

```bash
# API サーバー起動（ポート 3002）
just api

# Worker 起動
just worker

# 型チェック
just check
# または:
cargo check

# ローカル Docker サービス起動（SurrealDB + dify-sandbox）
just up

# OpenAPI コード生成（spec → openapi-gen → mcp）
just gen

# フロントエンド開発サーバー
cd frontend && npm run dev
cd cms-dashboard && npm run dev
```

## Architecture

### Backend（`crates/`）

- **API 層**: TypeSpec → OpenAPI → Axum サーバー自動生成（`crates/openapi-gen/`）
- **Worker 層**: kafru キューによるバックグラウンドタスク処理（`crates/worker/`）
- **データ層**: SurrealDB（CMS API）+ PostgreSQL（repos）
- **ストレージ**: Cloudflare R2（音声・字幕ファイル）
- **音声処理**: FFmpeg + VoiceVox TTS（`crates/audio_generator/`）

### Frontend

- `frontend/`: React 18 / TanStack Router / UnoCSS / Supabase Auth
- `cms-dashboard/`: React / Vite / shadcn/ui

### OpenAPI Integration

スペックファースト開発:
1. `spec/main.tsp` を編集
2. `just gen_spec` で `spec/tsp-output/schema/openapi.yaml` 生成
3. `just gen_server` で `crates/openapi-gen/` を再生成
4. `just gen_mcp` で `mcp/` を再生成

## Environment Variables

`crates/api` が必要とする主な環境変数（`.env`）:
- `DATABASE_URL` / `DATABASE_NAMESPACE` / `DATABASE_NAME`
- `JWT_SECRET`
- `DIFY_SANDBOX_ENDPOINT` / `DIFY_SANDBOX_API_KEY`

`crates/worker` が必要とする主な環境変数:
- `MCP_SERVER_ARGS`（MCP サーバーの起動引数）
- `VOICEVOX_ENDPOINT`
- Cloudflare R2 認証情報

## Local Development Setup

1. `just up` で SurrealDB と dify-sandbox を起動
2. `.env` を設定（`.env.example` を参照）
3. `just api` で API サーバーを起動
4. `just worker` で Worker を起動

## Testing

- Rust: `cargo test`
- Frontend: `cd frontend && npm test` / `cd cms-dashboard && npm test`

# The Justfile automatically loads environment variables from .env file

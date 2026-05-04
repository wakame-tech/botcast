---
id: "01KNM2BBT556RV0ZZB1EHEW2V7"
name: "TypeSpecからOpenAPI仕様を生成しRustコードとMCPサーバーを自動生成する"
status: "stable"
---

## 関連ファイル

- `spec/main.tsp`
- `spec/tsp-output/schema/openapi.yaml` (Generated)
- `cms/crates/openapi-gen/src/lib.rs` (Generated)
- `cms/crates/openapi-gen/src/models.rs` (Generated)
- `cms/crates/openapi-gen/src/apis/collections.rs` (Generated)
- `cms/crates/openapi-gen/src/apis/record.rs` (Generated)
- `cms/crates/openapi-gen/src/apis/script.rs` (Generated)
- `cms/crates/openapi-gen/src/apis/job.rs` (Generated)
- `mcp/src/index.ts` (Generated)

## 機能概要

TypeSpec (`spec/main.tsp`) を唯一の API 定義として、OpenAPI YAML を生成し、そこから Rust のモデル・トレイト（openapi-gen クレート）と TypeScript の MCP サーバーを自動生成するコード生成パイプラインを構成する。

## 意図

API 定義を一箇所に集約し、Rust バックエンドと MCP サーバーの型安全性を自動的に保証する。手動でのスキーマ同期作業を排除する。

## シナリオ

### TypeSpec から OpenAPI 仕様を生成する

1. `spec/main.tsp` で API モデルとエンドポイントを定義する
2. TypeSpec コンパイラが `spec/tsp-output/schema/openapi.yaml` を生成する

### OpenAPI 仕様から Rust コードを生成する

1. OpenAPI Generator が `openapi.yaml` を入力とする
2. `cms/crates/openapi-gen/src/` にモデル型とトレイトを生成する
3. `ApiImpl` がトレイトを実装して実際のハンドラロジックを提供する

### OpenAPI 仕様から MCP サーバーを生成する

1. OpenAPI 仕様からツール定義を生成する
2. `mcp/src/index.ts` に TypeScript MCP サーバーコードを配置する
3. 各 API エンドポイントが MCP ツールとしてマッピングされる

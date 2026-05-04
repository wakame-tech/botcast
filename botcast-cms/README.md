# botcast-cms

LLM の Structured Output 等の出力結果を保存する汎用 CMS。
SurrealDB を利用して構造化データを保存し、MCP 経由で LLM Agent から操作可能。

## 機能

- **Collection 管理**: JSON Schema 付きコレクションの CRUD
- **Record 管理**: スキーマバリデーション付きレコードの CRUD
- **AI 生成**: OpenAI Structured Output によるレコード自動生成
- **画像管理**: レコードへの画像アップロード・取得
- **Canvas**: レコード間のグラフ関係（エッジ）管理
- **Script 実行**: Dify Sandbox 経由での Python/Node.js 実行
- **MCP Server**: LLM Agent 向けツールインターフェース

## プロジェクト構成

```
botcast-cms/
├── cms/                    # Rust バックエンド
│   ├── crates/
│   │   ├── api/           # REST API サーバー (Axum)
│   │   ├── worker/        # ジョブキュー (kafru)
│   │   └── openapi-gen/   # OpenAPI 生成コード
│   └── Justfile           # ビルドコマンド
├── web/                    # React フロントエンド (Vite)
├── spec/                   # TypeSpec API 仕様
├── mcp/                    # MCP Server
├── docs/                   # ドキュメント
└── Justfile               # リリースコマンド
```

## セットアップ

### 必要なツール

- Rust 1.85+
- Node.js 20+
- Docker (SurrealDB 用)
- [just](https://github.com/casey/just) (タスクランナー)

### 起動

```bash
# SurrealDB 起動
docker compose up -d

# API サーバー起動
cd cms && just run

# フロントエンド起動 (別ターミナル)
cd web && npm install && npm run dev
```

### 環境変数

`cms/.env` を作成:

```env
DATABASE_URL=localhost:8000
DATABASE_USERNAME=root
DATABASE_PASSWORD=root
DATABASE_NAMESPACE=botcast
DATABASE_NAME=cms
IMAGE_STORAGE_PATH=./data/images
```

## 開発コマンド

### API サーバー

```bash
cd cms

just run                  # サーバー起動
cargo build              # ビルド
cargo test               # テスト
cargo clippy             # Lint
cargo fmt                # フォーマット
```

### フロントエンド

```bash
cd web

npm run dev              # 開発サーバー
npm run build            # ビルド
npm run test             # テスト
npm run lint             # Lint
```

### OpenAPI 生成

```bash
cd spec && npm run build    # TypeSpec → OpenAPI YAML
cd cms && just gen_server   # OpenAPI → Rust サーバーコード
```

## リリース

Semantic Versioning でのリリース自動化:

```bash
# バージョン確認
just version              # Current version: 0.1.0

# ドライラン
just release-dry patch    # 0.1.0 → 0.1.1 をプレビュー
just release-dry minor    # 0.1.0 → 0.2.0 をプレビュー
just release-dry major    # 0.1.0 → 1.0.0 をプレビュー

# リリース実行
just release patch        # パッチリリース (バグ修正)
just release minor        # マイナーリリース (機能追加)
just release major        # メジャーリリース (破壊的変更)

# GitHub Release も作成
just release-full patch   # release + GitHub Release

# リリース履歴
just releases             # 過去タグ一覧
```

`just release` が行う処理:
1. 未コミット変更がないか確認
2. `cms/crates/api/Cargo.toml` のバージョン更新
3. `web/package.json` のバージョン更新
4. `chore: bump version to X.Y.Z` でコミット
5. `vX.Y.Z` タグ作成
6. リモートに push

## API 使用例

### Collection 作成

```bash
curl -X POST "http://localhost:3002/collections" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "users",
    "schema": {
      "type": "object",
      "properties": {
        "name": {"type": "string"},
        "avatar": {"type": "string", "format": "image"}
      },
      "required": ["name"]
    }
  }'
```

### Record 作成

```bash
# Collection ID は "collection:xxx" の "xxx" 部分を使用
curl -X POST "http://localhost:3002/records/{collectionId}" \
  -H "Content-Type: application/json" \
  -d '{"data": {"name": "Alice"}}'
```

### AI によるレコード生成

```bash
curl -X POST "http://localhost:3002/records/{collectionId}/generate" \
  -H "Content-Type: application/json" \
  -d '{"prompt": "日本の主要都市の情報を生成"}'
```

## 技術スタック

| レイヤー | 技術 |
|---------|------|
| API | Rust, Axum, SurrealDB |
| フロントエンド | React 19, Vite, TypeScript |
| API 仕様 | TypeSpec → OpenAPI |
| AI | OpenAI Structured Output |
| MCP | openapi-mcp-generator |

## ドキュメント

- [ARCHITECTURE.md](./ARCHITECTURE.md) - システム設計・アーキテクチャ
- [CLAUDE.md](./CLAUDE.md) - Claude Code 向けガイダンス
- [docs/specres/](./docs/specres/) - 仕様書

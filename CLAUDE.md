# CLAUDE.md

**すべてのレスポンスは日本語で行うこと。**

## 構成

- **crates/worker**: Rust バックエンド（worker / audio_generator / repos）
- **web**: React + TypeScript フロントエンド（TanStack Router）
- **botcast-cms**: CMS サーバー（Rust/Axum + SurrealDB）。認証は botcast-cms JWT を使用

## 開発コマンド

```bash
# worker 起動
just worker

# 型チェック
cargo check

# web 開発サーバー
cd web && npm run dev

# web 型チェック・lint
cd web && npm run check
```

## 認証

- botcast-cms の POST /auth/signin でトークン取得
- web は localStorage にトークンを保存し Authorization: Bearer <token> で送信
- 環境変数 VITE_CMS_URL に botcast-cms のエンドポイントを設定

## 環境変数

- DATABASE_URL: PostgreSQL 接続文字列
- VITE_API_URL: botcast-worker API URL（web）
- VITE_CMS_URL: botcast-cms URL（web）
- VOICEVOX_ENDPOINT: VoiceVox エンジン URL（任意）

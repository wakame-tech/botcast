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

# web 依存インストール（初回。web/node_modules が無いと上位の node_modules を拾う）
cd web && npm ci

# web 開発サーバー
cd web && npm run dev

# web 型チェック・lint
cd web && npm run check
```

## 認証

- botcast-cms の POST /auth/signin でトークン取得
- web は localStorage にトークンを保存し Authorization: Bearer <token> で送信（`web/src/lib/cms_client.ts` の middleware）
- ユーザー ID は JWT の `sub` から取得（`useSession().userId`）
- CMS の型は `cd web && npm run generate` で `../botcast-cms` の openapi.yaml から再生成
- 環境変数 VITE_CMS_URL に botcast-cms のエンドポイントを設定

## 環境変数

- VITE_CMS_URL: botcast-cms URL（web）
- CMS_URL: botcast-cms URL（worker）
- API_KEY: botcast-cms の API_KEY と同じ値。worker → CMS の `X-Api-Key` 認証に使い、MCP サーバーの子プロセスにもそのまま渡る
- MCP_SERVER_ARGS: botcast-cms MCP サーバーの起動引数（例: `../botcast-cms/mcp/build/index.js`）
- KAFRU_DB_HOST / KAFRU_DB_PORT / KAFRU_DB_USERNAME / KAFRU_DB_PASSWORD / KAFRU_DB_NAMESPACE / KAFRU_DB_NAME: ジョブキュー (kafru) の SurrealDB。ローカルは `just up` で 4030 番に起動
- VOICEVOX_ENDPOINT: VoiceVox エンジン URL（音声生成に必要。例: `docker run -p 50021:50021 voicevox/voicevox_engine:cpu-latest`）
- OTLP_COLLECTOR_ENDPOINT: OpenTelemetry collector

## 音声ファイル

生成した mp3 / srt は botcast-cms の `/records/{c}/{r}/images/{audio,srt}` に保存し、episode の `audio_url` / `srt_url` にその相対パスを入れる。web は認証付きで取得して Blob URL にして再生する（`useCmsFileUrl`）。

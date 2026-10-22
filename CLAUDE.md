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

- ログインは Firebase Auth の Google のみ（`web/src/lib/firebase.ts`）。`signInWithPopup` で得た ID トークンを botcast-cms の POST /t/{tenantId}/auth/firebase で CMS の JWT に交換する。Firebase のセッションは持たない。JWT は発行元テナントでのみ有効
- Firebase の Web config（プロジェクト `botcast-cms`）は公開値なのでコードの既定値に置き、`VITE_FIREBASE_API_KEY` / `VITE_FIREBASE_AUTH_DOMAIN` / `VITE_FIREBASE_PROJECT_ID` で上書きできる。ログインするホストは Firebase の承認済みドメインに登録が必要
- web は localStorage にトークンを保存し Authorization: Bearer <token> で送信（`web/src/lib/cms_client.ts` の middleware）
- ユーザー ID は JWT の `sub` から取得（`useSession().userId`）
- レコードの所有者は CMS の `owner_id`（作成したユーザーの `sub`）。`data` に `user_id` は持たない。コレクションは `access: {read: public, write: owner}` で、web は所有者にだけ編集・削除・ジョブの導線を出す（`isOwner`）
- worker はお便りエピソードを `X-Api-Key` + `X-On-Behalf-Of: <番組の owner_id>` で作り、番組の所有者のレコードにする
- CMS の型は `cd web && npm run generate` で `../botcast-cms` の openapi.yaml から再生成
- 環境変数 VITE_CMS_URL に botcast-cms のエンドポイント、VITE_CMS_TENANT にテナント ID を設定

## 環境変数

- VITE_CMS_URL: botcast-cms URL（web）
- VITE_CMS_TENANT: botcast-cms のテナント ID（web。既定 `botcast`）。CMS は全エンドポイントが `/t/{tenantId}/...` 配下にある
- VITE_FIREBASE_API_KEY / VITE_FIREBASE_AUTH_DOMAIN / VITE_FIREBASE_PROJECT_ID: Firebase の Web config（web。未設定時は `botcast-cms` プロジェクトの値）
- VITE_WORKER_URL: worker URL（web。未設定時 `http://localhost:9001`。セルフホストでは nginx が中継する `/api`）。web は `POST /jobs` に botcast-cms のトークンを `Authorization: Bearer` で付け、worker が CMS の `GET /t/{tenantId}/collections` で検証する
- CMS_URL: botcast-cms URL（worker）
- CMS_TENANT: botcast-cms のテナント ID（worker。既定 `botcast`）
- API_KEY: botcast-cms でテナント `botcast` の役割 `service` の機械用のユーザーに発行したキー（`api api-key issue --tenant botcast --name worker --user botcast-worker --roles service`）。worker → CMS の `X-Api-Key` 認証に使い、MCP サーバーの子プロセスにもそのまま渡る
- MCP_SERVER_ARGS: botcast-cms MCP サーバーの起動引数（例: `../botcast-cms/mcp/build/index.js`）
- KAFRU_DB_HOST / KAFRU_DB_PORT / KAFRU_DB_USERNAME / KAFRU_DB_PASSWORD / KAFRU_DB_NAMESPACE / KAFRU_DB_NAME: ジョブキュー (kafru) の SurrealDB。ローカルは `just up` で 4030 番に起動
- VOICEVOX_ENDPOINT: VoiceVox エンジン URL（互換。`TTS_ENGINES` が無いときに `voicevox` だけのエンジンとして扱う。例: `docker run -p 50021:50021 voicevox/voicevox_engine:cpu-latest`）
- TTS_ENGINES: 使う TTS エンジンの名前（カンマ区切り。例: `voicevox,irodori`）。無ければ `VOICEVOX_ENDPOINT` を読む。どちらも無ければ音声生成のジョブが失敗する
- TTS_DEFAULT_ENGINE: 旧形式の台本の `speaker`（数字だけ = VOICEVOX のスタイル ID）に使うエンジン（既定: `TTS_ENGINES` の先頭）
- TTS_ENGINE_<NAME>_KIND / _ENDPOINT: エンジンの種類（`voicevox` | `openai`）と URL（必須。`<NAME>` はエンジン名を大文字にし `-` を `_` にしたもの。OpenAI 互換は `/v1` まで含める。URL に認証情報 `user:pass@` は書けない。トークンは `_TOKEN` に書く）
- TTS_ENGINE_<NAME>_TOKEN / _MODEL: OpenAI 互換の bearer トークンと `model`（任意。model の既定は `irodori-tts`）。トークンはログに出さない
- TTS_ENGINE_<NAME>_MAX_CONCURRENCY / _TIMEOUT_SEC: 同時に投げる数（既定 1）とタイムアウト（既定 120 秒）
- TTS_CACHE_DIR: 合成結果のキャッシュの置き場（任意。既定はジョブのワークディレクトリ）。その下の `<CMS_TENANT>/` にテナントごとに置く
- OTLP_COLLECTOR_ENDPOINT: OpenTelemetry collector

## CMS のクライアント

- worker は botcast-cms の spec から生成した `crates/cms-client` で CMS を呼ぶ（`usecase/cms_client.rs` の `CmsClient` と `api/auth.rs` の `CmsTokenVerifier` が包む）。手で HTTP を組み立てない
- `crates/cms-client` は生成物なので編集しない。CMS の API が変わったら `just sync-cms-client`（`../botcast-cms` の `cms/crates/client` をコピーし、取り込んだ commit を `SOURCE_REV` に書く）
- 実物の CMS への結合テスト: `CMS_URL=... CMS_TENANT=botcast API_KEY=... cargo test -p worker -- --ignored against_cms`（`CMS_TEST_OWNER=user:xxxx` で `X-On-Behalf-Of` も確かめる）

## CMS のコレクション

- voices / podcasts / episodes / scripts / mails のスキーマは `crates/worker/src/usecase/collections.rs` が唯一の情報源。worker の起動時に CMS へ反映する（無ければ作成、差分があれば `PUT /collections` で更新。既存レコードが新スキーマを満たさなければ起動に失敗する）
- 参照は素のレコード ID で持ち、スキーマの `x-ref`（参照先コレクションの ID）で宣言する（`episodes.podcast_id` / `mails.episode_id` / `podcasts.schedule.script_id`）。コレクション ID は環境ごとに違うので、seed が参照先から順に作って埋め込む
- データ移行は `deploy/selfhost/migrations/` の SurrealQL を CMS の SurrealDB（namespace = テナント ID）に流す

## 声と配役

- 声は CMS の `voices`（`slug`（x-unique。英小文字で始める）・`name`・`engine`（`TTS_ENGINES` の名前）・`voice_id`・`styles`・`ref_audio`・`params`・`seed`・`caption`・`sample`・`icon`）。web の「声」画面で作成し、参照音声・試聴用の音声・アイコン画像をアップロードする
- 番組の配役は `podcasts.cast = { "<別名>": "<voices の slug>" }`。web の番組の編集画面で編集する。台本（スクリプト・LLM）は `speaker` に別名を書き、任意の `style`（口調・感情の自由文）を付けられる
- worker は音声生成の前に `speaker` を 配役の別名 → `voices` の slug → 数字だけなら旧形式（既定のエンジンのスタイル ID）の順で解決する（`usecase/voices.rs`）。`cast` も `voices` も無い番組は今までどおり動く
- 使える声は番組の持ち主（podcast の `owner_id`）の声だけ。`voices` は `read: owner`。配役の別名に数字だけは使えない。参照音声は `audio/*` で 20MB（`MAX_FILE_SIZE`）まで

## 音声ファイル

生成した mp3 / srt は botcast-cms の `/records/{c}/{r}/images/{audio,srt}` に保存し、episode の `audio_url` / `srt_url` にその相対パスを入れる。web は認証付きで取得して Blob URL にして再生する（`useCmsFileUrl`）。

## お便りエピソードの定期生成

- お便りは CMS の `mails`（`episode_id` / `radio_name` / `body`。所有者は投稿者）。web のエピソード画面から投稿する
- 番組の `data.schedule = { cron, script_id, enabled, arguments }`（cron は Quartz 7 フィールド・UTC）を worker の `ScheduleSync` が 60 秒ごとに kafru のスケジュールへ同期する
- スクリプトの `data.arguments` は引数の **JSON Schema**、その **値** は番組の `schedule.arguments`。web の定期実行設定でスキーマから生成したフォームで入力する
- 発火すると `GenerateEpisode` ジョブが `schedule.arguments`（無ければ `{}`）をスクリプトのスキーマで検証し（違反ならジョブ失敗）、スクリプト（Node.js）を CMS `/scripts` で実行する。スクリプトは `context = { podcast, previous_episode, mails, arguments }` を受け取り、`{ title?, sections }` を JSON で標準出力する
- web のスクリプト詳細画面で、サンプルの context と入力した引数で試し実行できる
- 手動実行は worker の MCP ツール `generate_episode(podcast_id)`

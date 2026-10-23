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
- OTLP_COLLECTOR_ENDPOINT: OpenTelemetry collector（ローカルは `just up` で起動する collector の `http://localhost:4317`）

## トレース

- worker は OTLP で collector → Jaeger（v2・badger で保存。UI はローカルが http://localhost:16686 ）にトレースを送る。設定は `deploy/selfhost/otel-collector-config.yaml` / `jaeger-config.yaml` をローカルと共用
- ジョブを積むときに W3C の `traceparent` を kafru のジョブの引数（`traceparent`）に入れ、実行側の `execute_task` の span の親にする（`src/trace_context.rs`）。`POST /jobs` と MCP（`/mcp`）のツールから実行までが 1 本のトレースになる。`traceparent` を持たないジョブ（定期実行・以前のジョブ）は実行時に新しいトレースを始め、その `traceparent` を書き足す。`GET /jobs` の `traceId` で Jaeger の `/trace/{traceId}` を開ける
- span: エージェントのループ（`agent.run` / `agent.turn`）、LLM の呼び出し（`chat {model}`。モデル・トークン数・停止理由）、MCP のツール（`execute_tool {name}`。ツール名・成否）、CMS（`{METHOD} {パスのテンプレート}`）。プロンプト・応答・ツールの引数の本文は span に入れない
- 失敗したジョブは `execute_task` の span の status を Error にし、エラーメッセージを付ける

## CMS のクライアント

- worker は botcast-cms の spec から生成した `crates/cms-client` で CMS を呼ぶ（`usecase/cms_client.rs` の `CmsClient` と `api/auth.rs` の `CmsTokenVerifier` が包む）。手で HTTP を組み立てない
- `crates/cms-client` は生成物なので編集しない。CMS の API が変わったら `just sync-cms-client`（`../botcast-cms` の `cms/crates/client` をコピーし、取り込んだ commit を `SOURCE_REV` に書く）
- 実物の CMS への結合テスト: `CMS_URL=... CMS_TENANT=botcast API_KEY=... cargo test -p worker -- --ignored against_cms`（`CMS_TEST_OWNER=user:xxxx` で `X-On-Behalf-Of` も確かめる）

## CMS のコレクション

- voices / podcasts / episodes / scripts / mails / llm_models / user_settings / fetch_allowed_hosts のスキーマは `crates/worker/src/usecase/collections.rs` が唯一の情報源。worker の起動時に CMS へ反映する（無ければ作成、差分があれば `PUT /collections` で更新。既存レコードが新スキーマを満たさなければ起動に失敗する）
- 参照は素のレコード ID で持ち、スキーマの `x-ref`（参照先コレクションの ID）で宣言する（`episodes.podcast_id` / `mails.episode_id` / `podcasts.schedule.script_id`）。コレクション ID は環境ごとに違うので、seed が参照先から順に作って埋め込む
- データ移行は `deploy/selfhost/migrations/` の SurrealQL を CMS の SurrealDB（namespace = テナント ID）に流す

## 声と配役

- 声は CMS の `voices`（`slug`（x-unique。英小文字で始める）・`name`・`engine`（`TTS_ENGINES` の名前）・`voice_id`・`styles`・`ref_audio`・`params`・`seed`・`caption`・`sample`・`icon`）。web の「声」画面で作成し、参照音声・試聴用の音声・アイコン画像をアップロードする
- 番組の配役は `podcasts.cast = { "<別名>": "<voices の slug>" }`。web の番組の編集画面で編集する。台本（スクリプト・LLM）は `speaker` に別名を書き、任意の `style`（口調・感情の自由文）を付けられる
- worker は音声生成の前に `speaker` を 配役の別名 → `voices` の slug → 数字だけなら旧形式（既定のエンジンのスタイル ID）の順で解決する（`usecase/voices.rs`）。`cast` も `voices` も無い番組は今までどおり動く
- 使える声は番組の持ち主（podcast の `owner_id`）の声だけ。`voices` は `read: owner`。配役の別名に数字だけは使えない。参照音声は `audio/*` で 20MB（`MAX_FILE_SIZE`）まで
- 参照音声が優先: OpenAI 互換のエンジンでは `ref_audio` があれば参照音声で声を複製する（`voice_id` が空でも `none` でもよく、サーバー側の声の名前は `voice_id`（`none` なら slug）。`audio_generator` の `tts/openai.rs`）。参照音声がある声には caption（声の `caption`・セリフの `style`）を送らない（参照音声と食い違うと声が不安定になる。Irodori-TTS v4.1 のモデルカード）。caption と seed は参照音声が無い声（`voice_id: none`）だけで使う
- caption だけの声は行ごと・絵文字で声色が揺れる。web の声の画面（持ち主だけ）の「試聴を作る」で同じ本文を seed を変えて何テイクか作り、選んだテイクを「この声を参照音声にする」（`ref_audio` にアップロードし、声の `seed` をそのテイクの seed にする）か「試聴用の音声にする」（`sample`）。参照音声にするなら 20〜30 秒以上読める数文がよい
- 試聴は worker の `POST /voices/{voice_id}/previews`（`{ text?（既定は数文。600 文字まで）, count?（1〜4。既定 3） }` → `{ voiceId, engine, text, takes: [{ seed, contentType: audio/wav, audioBase64, elapsedMs }] }`。`usecase/voice_preview.rs`）。`POST /jobs` と同じ CMS の JWT で、CMS の `GET /users/me` で呼び出し元を調べ、声を `X-On-Behalf-Of` で読んで `owner_id` も確かめる（他人の声は 404 / 403）。テイクは順番に合成し（エンジンの `MAX_CONCURRENCY` はジョブと共有）、保存しない。全体のタイムアウトは 280 秒（nginx の 300 秒より短く）で、web は 1 テイクずつ呼ぶ。エンジンが設定されていなければ 503。span `voice.preview` / `voice.preview.take` には声の ID・エンジン・テイク数・seed・時間・バイト数だけを入れ、音声は入れない

## 音声ファイル

生成した mp3 / srt は botcast-cms の `/records/{c}/{r}/images/{audio,srt}` に保存し、episode の `audio_url` / `srt_url` にその相対パスを入れる。web は認証付きで取得して Blob URL にして再生する（`useCmsFileUrl`）。

## LLM のモデル

- モデルの一覧は CMS の `llm_models`（`model_id`（x-unique。API に渡す ID）・`name`・`provider`（今は `anthropic`）・`default`・`enabled`・`order`）。読めるのはログインしたユーザー全員、書けるのは役割 `admin` とサービスだけ（`write_roles`）。worker の seed はコレクションが空のときだけ初期データ（`claude-opus-5-5`（既定）・`claude-sonnet-5-5`・`claude-haiku-5-5`）を入れる
- ユーザーの選択は `user_settings.llm_model`（`llm_models` のレコード ID。`access: owner`、1 人 1 件）。web のマイページ（本人のとき）の「設定」で選ぶ。API キーなどの秘密情報は `user_settings` に置かない
- worker は台本生成で、番組の `owner_id` の設定 → `llm_models` の既定（`default` かつ `enabled`、複数なら `order` が小さいもの）の順でモデルを決める（`usecase/llm_models.rs`）。設定のモデルが無効・一覧に無ければ既定にフォールバックし、warn ログと span `llm.resolve_model` の `llm.model_source = fallback` に出す。既定も無ければジョブが失敗する。モデルを環境変数では指定しない

## LLM の API キー（BYOK）

- Anthropic の API キーはユーザーの持ち込み。番組の持ち主が web のマイページ（本人のとき）の「設定」の「LLM の API キー」で CMS の `/t/{tenantId}/me/secrets/anthropic_api_key` に保存する（暗号化。web が読めるのは末尾 4 文字と更新日時だけ）
- worker は台本生成のジョブで、番組の `owner_id` を `X-On-Behalf-Of` で代理して `GET /me/secrets/anthropic_api_key/value` で値を取り出し、そのジョブの間だけ使う（`usecase/llm_api_key.rs`）。キーは `ApiKey`（Debug / Display を伏せる）で持ち、ログ・span・エラーに出さない
- キーが無い（持ち主がユーザーでない番組も）ときはジョブを「マイページで登録してください」のエラーで失敗させる。環境変数のキーは無い（`ANTHROPIC_API_KEY` は廃止）
- LLM のトークン数は span（`chat {model}` / `agent.run` の `gen_ai.usage.*`）と、ジョブの引数の `usage`（合計。`GET /jobs` の `usage`）に残す。失敗したジョブの分も残す

## 台本生成

- `GenerateScript` は LLM に CMS の MCP ツールのうち読み取りだけ（`agent_service.rs` の `READ_ONLY_CMS_TOOLS`）を渡し、台本を structured outputs（`output_config.format`）で `{ title?, sections }` の JSON として返させる。保存は worker が行い、LLM に保存を頼まない
- 出力のスキーマは `collections.rs` の episodes の `title` / `sections` から作る（`usecase/script_output.rs`。`oneOf` → `anyOf`、`maxLength` などの制約は外して送り、受け取ったあとに元のスキーマで検証する）。違反・`max_tokens`・`refusal` はジョブ失敗。`generateAudio` は保存できたときだけ音声に進む
- span `script.save` にはセリフ数・話者の数などの件数だけを入れ、台本の本文は入れない
- LLM には MCP のツールに加えて worker が実装するツール（client tool）`fetch_url` を渡す（`usecase/fetch_url_tool.rs`。1 ジョブ 20 回まで）。エージェントのループは `agent_service.rs` の `LocalTool` を MCP より先に引く。tool_choice は指定しない（forced は 5.5 系で 400）

## Web の取得（fetch_url・schedule.sources）

- CMS のスクリプトは dify-sandbox（seccomp でネットワーク禁止）で動くので Web を読めない。サンドボックスは開けず（LAN への SSRF になる）、worker が `usecase/safe_fetch.rs` の `SafeFetcher` で取得する
- 取得してよいホストは CMS の `fetch_allowed_hosts`（`host`（x-unique。小文字のドメイン名）・`enabled`・`description`）。ホスト名と完全一致（サブドメインは別に登録）。読めるのはログインしたユーザー全員、書けるのは役割 `admin` とサービスだけ（`llm_models` と同じ）。初期データは入れない（管理者が CMS で登録する。空なら何も取得できない）。環境変数では設定しない
- HTTPS（443 番）だけ。URL の認証情報・IP アドレスのホストは不可。DNS の結果を **すべて** 確かめ（ループバック・プライベート・リンクローカル（169.254.169.254 を含む）・ユニークローカル・CGNAT・予約などは拒否）、確かめたアドレスに接続を固定する（reqwest の `resolve`。プロキシの環境変数も使わない）ので DNS rebinding で内部に繋がらない
- リダイレクトは自動で追わず、1 回ごとに URL・許可リスト・DNS を確かめ直す（3 回まで）。接続 5 秒・全体 20 秒のタイムアウト、本文 2MiB まで（超えたら失敗）
- JSON はそのまま（ただしキーが `_html` で終わる文字列は本文のテキストにする。Zenn の `body_html` など）、HTML は見出し・段落のテキスト（`readable_text`）にし、20,000 文字で切る
- span `web.fetch` にはホスト（`server.address`）・状態コード・バイト数・リダイレクト数だけを入れ、URL のパス・本文は入れない
- 実物の zenn.dev への確認: `cargo test -p worker -- --ignored fetch_against_zenn`

## エピソードの定期生成（スクリプト・LLM）

- お便りは CMS の `mails`（`episode_id` / `radio_name` / `body`。所有者は投稿者）。web のエピソード画面から投稿する
- 番組の `data.schedule = { cron, enabled, generator?, script_id?, arguments?, prompt?, sources? }`（cron は Quartz 7 フィールド・UTC）を worker の `ScheduleSync` が 60 秒ごとに kafru のスケジュールへ同期する。`generator` は `script`（既定。無い既存のレコードも script）か `llm`。script は `script_id`、llm は `prompt` が必須（スキーマの `anyOf`。足りない番組は同期しない）
- `sources` は事前に取得する URL（10 個まで。`fetch_allowed_hosts` のホストだけ）。1 つでも取得に失敗したらジョブを失敗させる
- スクリプトの `data.arguments` は引数の **JSON Schema**、その **値** は番組の `schedule.arguments`。web の定期実行設定でスキーマから生成したフォームで入力する
- 発火すると `GenerateEpisode` ジョブが動く（`usecase/episode_generation.rs` の `EpisodeGenerator`）
  - script: `schedule.arguments`（無ければ `{}`）をスクリプトのスキーマで検証し（違反ならジョブ失敗）、スクリプト（Node.js）を CMS `/scripts` で実行する。スクリプトは `context = { podcast, previous_episode, mails, arguments, sources }`（`sources` は `[{ url, content_type, truncated, content }]`）を受け取り、`{ title?, sections }` を JSON で標準出力する
  - llm: 番組の持ち主のモデル・API キー（BYOK）で、`prompt`・取得した `sources`・配役（台本生成と同じプロンプト）を渡し、CMS の読み取りのツールと `fetch_url` を使わせて structured outputs で `{ title?, sections }` を受け取る。episodes のスキーマで検証してから番組の持ち主のエピソードを作る（失敗したらエピソードを作らない）。タイトルは LLM が書かなければ `第N回`。トークン数はジョブの `usage` に残す
  - どちらもエピソードを作れたら続けて音声を生成する
- web の番組の編集画面の「定期実行」で作り方・指示・資料の URL を編集し、取得を許可されたホストを表示する
- web のスクリプト詳細画面で、サンプルの context と入力した引数で試し実行できる
- 手動実行は worker の MCP ツール `generate_episode(podcast_id)`（web の番組画面の「エピソードを生成」）

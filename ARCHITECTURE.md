# アーキテクチャ

C4 モデルで記述する。

## Context

```mermaid
C4Context
  Person(user, "ユーザー", "番組・スクリプトを管理し、お便りを投稿する")
  System(botcast, "botcast", "ポッドキャストの台本・音声を生成する")
  System_Ext(cms, "botcast-cms", "データ・ファイル保存、認証、スクリプト実行")
  System_Ext(claude, "Anthropic API", "台本生成")
  System_Ext(tts, "TTS エンジン", "音声合成（VOICEVOX 互換・OpenAI 互換）")

  Rel(user, botcast, "使う")
  Rel(user, cms, "サインイン / CRUD")
  Rel(botcast, cms, "読み書き")
  Rel(botcast, claude, "台本生成")
  Rel(botcast, tts, "音声合成")
```

## Container

```mermaid
C4Container
  Person(user, "ユーザー")
  System_Ext(cms, "botcast-cms", "Rust/Axum + SurrealDB")
  System_Ext(claude, "Anthropic API")
  System_Ext(tts, "TTS エンジン", "VOICEVOX・AivisSpeech・Irodori-TTS-Server など")

  System_Boundary(b, "botcast") {
    Container(web, "web", "React + TanStack Router", "nginx で配信し /api を worker へ中継")
    Container(worker, "worker", "Rust/Axum", "HTTP API・MCP サーバー・ジョブ実行・定期実行同期")
    Container(mcp, "botcast-cms MCP", "Node.js", "worker の子プロセス")
    ContainerDb(kafru, "kafru DB", "SurrealDB", "ジョブキュー・スケジュール")
  }

  Rel(user, web, "使う")
  Rel(web, cms, "CRUD", "JWT")
  Rel(web, worker, "POST /jobs", "JWT")
  Rel(worker, cms, "トークン検証・保存・スクリプト実行", "X-Api-Key")
  Rel(worker, mcp, "ツール呼び出し", "stdio")
  Rel(mcp, cms, "CRUD", "X-Api-Key")
  Rel(worker, kafru, "enqueue / dequeue")
  Rel(worker, claude, "Messages API")
  Rel(worker, tts, "HTTP")
```

## Component (worker)

```mermaid
C4Component
  Container_Boundary(w, "worker") {
    Component(api, "api", "Axum", "/jobs, /version, /mcp")
    Component(mcpsrv, "mcp_server", "rmcp", "generate_audio / generate_script / generate_episode / list_jobs / get_job_status")
    Component(task, "TaskService", "", "ジョブ登録・一覧")
    Component(sync, "ScheduleSync", "", "番組の schedule を 60 秒ごとに kafru へ同期")
    Component(jobs, "jobs", "", "kafru のジョブ実行")
    Component(gen, "episode_generation / AgentService", "", "お便りエピソード生成・LLM 台本生成")
    Component(audio, "audio_generator", "crate", "TTS エンジン + ffmpeg で mp3 / srt 生成")
  }
  ContainerDb(kafru, "kafru DB", "SurrealDB")
  System_Ext(cms, "botcast-cms")

  Rel(api, task, "")
  Rel(mcpsrv, task, "")
  Rel(task, kafru, "enqueue")
  Rel(sync, kafru, "スケジュール登録")
  Rel(jobs, kafru, "dequeue")
  Rel(jobs, gen, "")
  Rel(jobs, audio, "")
  Rel(gen, cms, "/t/{tenantId}/scripts 実行・保存")
  Rel(audio, cms, "mp3 / srt 保存")
```

## 音声合成（audio_generator）

- TTS エンジンは「プロトコル」で 2 種類の窓口にまとめる: VOICEVOX 互換（`/audio_query` → `/synthesis`。VOICEVOX・AivisSpeech など）と OpenAI 互換（`POST /audio/speech`。Irodori-TTS-Server・OpenAI など）。エンジンは環境変数（`TTS_ENGINES` / `TTS_ENGINE_<NAME>_*`）で設定し、worker の起動時に検証してレジストリを作る
- エンジンは「文 1 つ → 音声のバイト列」だけを受け持つ。文の分割（100 文字まで）・各文の出力の正規化（ffmpeg で 44.1kHz・s16・2ch の wav）・長さと SRT・結合・キャッシュ・エンジンごとの同時実行数は共通の層で行う
- 合成結果のキャッシュは `TTS_CACHE_DIR/<テナント ID>/` に置き、キーには処理の版・エンジンの版（VOICEVOX は `GET /version`、OpenAI 互換は model）・声・本文を含める
- OpenAI 互換の参照音声は、サーバー側の声の ID を `{テナント ID}-{voice_id}-{ハッシュの先頭 12 文字}` にして版ごとに分ける（既にあれば上書きしない）
- 入力は声が解決済みのセクション（`ResolvedVoice { engine, voice_id, params, style, seed, ref_audio }`）。台本の `speaker` は worker が解決する（下の「声と配役」）。`audio_generator` は CMS を知らない

## 声と配役

- 声は CMS の `voices`（`slug` / `name` / `engine` / `voice_id` / `styles` / `ref_audio` / `params` / `seed` / `caption` / `sample`）、番組の配役は `podcasts.cast`（別名 → 声の slug）
- worker（`usecase/voices.rs` の `SpeakerResolver`）が音声生成の前に、台本の `speaker` を 配役の別名 → `voices` の slug → 数字だけなら旧形式（`TTS_DEFAULT_ENGINE` の VOICEVOX のスタイル ID）の順で `ResolvedVoice` にする。解決できなければ、どのセクションのどの speaker かを書いたエラーでジョブを失敗させる
- 使える声は番組の持ち主（podcast の `owner_id`）の声だけ（配役経由・slug 直接の両方）。他人の声・同じ slug が複数ある声・slug の形でない配役の値は、その speaker のエラーにする。`voices` は `read: owner`（参照音声は人の声のサンプルなので本人とサービス以外は読めない）
- 配役の別名に数字だけの文字列は使えない（旧形式の speaker と区別するため。スキーマと web の両方で弾く）
- 参照音声（`voices.ref_audio`）は、声のエンジンが OpenAI 互換のときだけ CMS から取得してハッシュを付け、audio_generator がエンジンに同期する。音声（`audio/*`）で 20MB 以下に限る。声のレコードの `version` と大きさが同じなら、プロセス内のキャッシュ（上限 16 件）でジョブをまたいで使い回す
- 台本生成（LLM）には配役を、改行を除き 40 字までにした名前・style の名前の JSON としてユーザーメッセージで渡す（システムプロンプトに埋め込まない）
- Serif の `style`（口調・感情の自由文）は、VOICEVOX 互換では声の `styles` で引き、OpenAI 互換では caption に渡す。OpenAI 互換で style が無いセリフは声の `caption` を使う
- 番組が見つからないエピソードは配役なしで解決する（旧形式の speaker はそのまま作れる。持ち主が分からないので声は使えない）

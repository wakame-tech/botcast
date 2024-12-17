---
title: "リリースノート"
nav_order: 4
---

# ~~日記~~ リリースノート

## Sprint 2024-08-21

- 技術選定した
- Deno Deploy でフロントエンド/APIのCDを作り、空ページをデプロイした #3
- supabaseをセットアップ #5
- Prismaセットアップ
- フロントエンドセットアップ
- 最小のワーカーを書いた #11

## Sprint 2024-08-28

- Cloudflare R2 をセットアップして音声をアップロードできるようにした
  - [s3](https://crates.io/crates/rust-s3) crateで
- compose-cd でワーカーのCDを作った
- Task APIを実装した

## Sprint 2024-09-04

- supabase でユーザー認証とAPI認証を実装した #17
  - [@supabase/auth-ui-react](https://www.npmjs.com/package/@supabase/auth-ui-react) で

## Sprint 2024-09-11

- UTF-8以外のHTMLに対応 #6
- HTML→MD変換する実装を変えた #9
- 音声のタイムスタンプ計算を実装した #30

## Sprint 2024-09-18

- PRをレビューしてくれるBotを作って導入した #32
- ARCHITECTURE.md を作った #37
- SRTファイルを生成するようにした #30
- `shadcn/ui` を使ってUIを作り直した #41

## Sprint 2024-09-25

- 進捗ないです...

## Sprint 2024-10-02

- head, styleタグを除外するように修正
- 今後のためにスクレイピングタスクと音声合成タスクを分離中 #15
- LLMで原稿を変換したい: とりあえず要約

## Sprint 2024-10-09

- feat: スクリプト機能 [#44](https://github.com/wakame-tech/botcast/issues/44)
  - スクリプト実行ランタイム [#17](https://github.com/wakame-tech/botcast-worker/issues/17)
  - 原稿生成と音声生成を分離した [#45](https://github.com/wakame-tech/botcast/issues/45)
  - スクリプト作成機能 [#49](https://github.com/wakame-tech/botcast/issues/49)
  - ポッドキャストにスクリプトを設定し、エピソードはそれを引き継ぐように [#50](https://github.com/wakame-tech/botcast/issues/50)
- feat: コメント機能 [#47](https://github.com/wakame-tech/botcast/issues/47)

## Sprint 2024-10-16

- feat: ローカルでスクリプト作成・実行するためのCLIを作成 [#24](https://github.com/wakame-tech/botcast-worker/issues/24)
- clean: worker側の猛烈なリファクタリング
- feat: エピソード作成定期実行機能 [#21](https://github.com/wakame-tech/botcast-worker/issues/21)
  - 実行予定時刻とcronを持つ
  - 実行予定時刻を過ぎたらタスクを実行する, cronがあれば次回の実行予定時刻でタスクを作成

## Sprint 2024-10-23

- chore: ドキュメントページを作成 [#52](https://github.com/wakame-tech/botcast/pull/52)
- feat: エピソード定期作成フォーム [#53](https://github.com/wakame-tech/botcast/pull/53)
- fix: mp3とsrtファイルはPreSign URLを使うように [#54](https://github.com/wakame-tech/botcast/issues/54)
- fix: ローカルのスクリプトを呼べるようにする [#32](https://github.com/wakame-tech/botcast-worker/issues/32)

## Sprint 2024-10-30

- 進捗ないです...

## Sprint 2024-11-06

- fix: スクリプトではなくタスクが評価結果を持つ [#57](https://github.com/wakame-tech/botcast/issues/57)
- WIP: 環境変数を設定出来るようにする [#60](https://github.com/wakame-tech/botcast/issues/60)
  - [supabase vault](https://supabase.com/docs/guides/database/vault) 使う?
- WIP: UIブラッシュアップ [#59](https://github.com/wakame-tech/botcast/issues/59)
  - Figmaでイメージ作成中

## Sprint 2024-11-13

- 進捗ないです...

## Sprint 2024-11-20

### Aivis Speech

- [AivisSpeech](https://github.com/Aivis-Project/AivisSpeech) 導入中
  - Nvidia + CUDA driver N回入れ直しています。コンテナ上でのみ?発生する?
  - `voicevox_engine:nvidia-latest` イメージが悪いのか、ホストが悪いのかわからない

```bash
voicevox_engine-1  |     raise CoreError(
voicevox_engine-1  | voicevox_engine.core.core_wrapper.CoreError: modelデータ読み込みに失敗しました 
(/opt/voicevox_core/model/d0.bin): Failed to create session: Error calling ONNX Runtime C function: /onnxruntime_src/onnxruntime/core/providers/cuda/cuda_call.cc:124 std::conditional_t<THRW, void, onnxruntime::common::Status> onnxruntime::CudaCall(ERRTYPE, const char*, const char*, ERRTYPE, const char*) 
[with ERRTYPE = cudaError; bool THRW = true; std::conditional_t<THRW, void, onnxruntime::common::Status> = void] /onnxruntime_src/onnxruntime/core/providers/cuda/cuda_call.cc:117 std::conditional_t<THRW, void, onnxruntime::common::Status> onnxruntime::CudaCall(ERRTYPE, const char*, const char*, ERRTYPE, const char*) 
[with ERRTYPE = cudaError; bool THRW = true; std::conditional_t<THRW, void, onnxruntime::common::Status> = void] 
CUDA failure 35: CUDA driver version is insufficient for CUDA runtime version ; GPU=0 ; hostname=c348de8374be ; expr=cudaSetDevice(info_.device_id);
```

### スクリプト機能

- OpenAI Assistants APIをサポート [#40](https://github.com/wakame-tech/botcast-worker/issues/40)
  - `llm(api_key, prompt)` 関数を追加
  - `llm_assistant(api_key, thread_id, assistant_id, prompt)` 関数を追加
    - `thread_id` はスクリプト実行毎に作成される
- スクリプトを直接JSONで書くよりTypeScriptで書いてJSONを出力すると楽なことに気づいた。

```typescript
const listener_assistant_id = "asst_xxx";
const personality_assistant_id = "asst_yyy";
const mail: Mail = {
    name: "ポメ太郎",
    body: "カレーを作るときのコツはなんですか？",
};
const context = {
    "api_key": "$OPENAI_API_KEY",
};
const template = let_in(
    { "thread_id": eval_("create_thread(api_key)") },
    let_in(
        { "mail": llm_assistant(listener_assistant_id, mail.body) },
        {
            "sections": [
                serif(`${mail.name} さんからのお便りです。`),
                serif(llm_assistant(listener_assistant_id, mail.body)),
                serif(llm_assistant(personality_assistant_id, mail.body)),
            ],
        },
    ),
);
```

## Sprint 2024-11-27

- workerにOpenTelemetryを導入 [#37](https://github.com/wakame-tech/botcast-worker/issues/37)
  - workerのトレースをOpenTelemetry CollectorにTraceを送信してJaegerで確認できるようになった。
- 【スクリプト】環境変数を設定出来るようにする [#60](https://github.com/wakame-tech/botcast/issues/60)
- 【スクリプト】コメント機能 [#42](https://github.com/wakame-tech/botcast-worker/issues/42)

## Sprint 2024-12-04

- fix: UIを整えた
  - プレイヤーの状態管理に jotai を導入しました。

![](https://private-user-images.githubusercontent.com/129728379/394549248-326accab-e9f3-4cdb-ac4f-5c00ee17a814.png?jwt=eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiJnaXRodWIuY29tIiwiYXVkIjoicmF3LmdpdGh1YnVzZXJjb250ZW50LmNvbSIsImtleSI6ImtleTUiLCJleHAiOjE3MzQ0NTYwNTYsIm5iZiI6MTczNDQ1NTc1NiwicGF0aCI6Ii8xMjk3MjgzNzkvMzk0NTQ5MjQ4LTMyNmFjY2FiLWU5ZjMtNGNkYi1hYzRmLTVjMDBlZTE3YTgxNC5wbmc_WC1BbXotQWxnb3JpdGhtPUFXUzQtSE1BQy1TSEEyNTYmWC1BbXotQ3JlZGVudGlhbD1BS0lBVkNPRFlMU0E1M1BRSzRaQSUyRjIwMjQxMjE3JTJGdXMtZWFzdC0xJTJGczMlMkZhd3M0X3JlcXVlc3QmWC1BbXotRGF0ZT0yMDI0MTIxN1QxNzE1NTZaJlgtQW16LUV4cGlyZXM9MzAwJlgtQW16LVNpZ25hdHVyZT1mZmVlOThjYjBhNDc5MTdiOGZhYzFhMDI2NTM5Nzc5ZjM1MTc4MTEzY2U0ZTAxYmVlZGZlZDE0MGQzMDBhZmE5JlgtQW16LVNpZ25lZEhlYWRlcnM9aG9zdCJ9.RYlVO42r6gz3U_EjQjU7SkL3zdpBFS0Dha9ggBIh2aA)

- fix: 定期実行機能をタスクに持たせるように
- feat: 音声ファイルのサポート
  - 合成音声に加えてジングル等のSEが流せるようになりました。BGMは流せません。
  - 区間指定だけ出来ます。裏はffmpeg使っています。
- fix: ポッドキャスト・エピソード・スクリプトにdescriptionを追加
- feat: スクリプトの機能追加
  - `function_calling(open_ai_api_key, prompt, function)`
    - スクリプトにJSON schemaを保存しておき, この関数に渡す想定です。関数は1個に制限しています。

## Sprint 2024-12-11

- どのようにして番組デモ・お便りデモを作るか考えています
  - コーナーのテンプレートを作って話題をプロンプトとして与える?
    - <http://melody-flag.com/>
    - パーソナリティ・リスナーに属性を設定して行動させたい
      - 属性はデータとしてのスクリプトで設定する
  - RSSフィードからランダムに選ぶ?
    - RSSをパースする関数と乱数を追加したので `text(fetch(choice(rss(fetch('https://zenn.dev/feed')).items).link))` と書ける
  - アニメ/音楽紹介番組
    - アニメのあらすじ等をAPIで提供しているものがあまり見つからない...
    - Spotifyで30秒のプレビューが取れなくなっている...
- [Dify](https://docs.dify.ai/ja-jp) でエピソード投稿をするワークフローを作ろうとしています

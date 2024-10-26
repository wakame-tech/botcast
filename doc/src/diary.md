---
title: "リリースノート"
nav_order: 1
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

- スクリプト機能 #44
  - スクリプト実行ランタイム #17
  - 原稿生成と音声生成を分離した #45
  - スクリプト作成機能 #49
  - ポッドキャストにスクリプトを設定し、エピソードはそれを引き継ぐように #50
- コメント機能 #47

## Sprint 2024-10-16

- ローカルでスクリプト作成・実行するためのCLIを作成
- worker側の猛烈なリファクタリング
- エピソード作成定期実行機能 [#21](https://github.com/wakame-tech/botcast-worker/issues/21)
  - 実行予定時刻とcronを持つ
  - 実行予定時刻を過ぎたらタスクを実行する, cronがあれば次回の実行予定時刻でタスクを作成
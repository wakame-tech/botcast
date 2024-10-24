---
title: "概要"
nav_order: 1
---

# 概要

Botcast の機能メモとか設計メモとか

## 技術構成

- frontend
  - React
  - TanStack Router
  - tRPC
  - unocss
  - デプロイ先: Deno Deploy(`/dist` を静的ファイルとして)
- API
  - Deno
  - Prisma + Prisma Accelerate
  - supabase
  - デプロイ先: Deno Deploy
- worker
  - Cloudflare R2
  - デプロイ先: 自宅のUbuntuマシン

## 機能

- ユーザー(`User`)作成・ログインが出来る
- ユーザーはポッドキャスト(`PodCast`) を作成できる
- ポッドキャストは複数のエピソード(`Episode`) からなる
- エピソード原稿を作成できる
  - Webページから: HTMLを取得して原稿に変換
  - アップロードする: Markdownをアップロード
- エピソード原稿をスクリプトで作成できる [スクリプト機能](/script.md)

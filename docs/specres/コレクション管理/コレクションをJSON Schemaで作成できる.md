---
id: "01KNM2BBT5Y18PC3WDQQN18KCA"
name: "コレクションをJSON Schemaで作成できる"
status: "stable"
---

## 関連ファイル

- `cms/crates/api/src/openapi_adapter.rs`
- `cms/crates/api/src/repository/mod.rs`
- `cms/crates/api/src/repository/surrealdb.rs`
- `cms/crates/openapi-gen/src/apis/collections.rs`
- `spec/main.tsp`

## 機能概要

REST API `POST /collections` でコレクション名と JSON Schema を受け取り、SurrealDB にスキーマ付きコレクションを作成する。コレクションはレコードの型定義となり、以降のレコード操作はこのスキーマに従ってバリデーションされる。

## 意図

LLM エージェントが自律的にデータモデルを定義できるようにするため、JSON Schema を入力としてコレクションを動的に作成する仕組みを提供する。

## 主要なメンバー

- `Collection { id, name, schema, created_at }` — SurrealDB `collection` テーブルに格納されるエンティティ
- `CreateOrUpdateCollection { name, schema }` — コレクション作成リクエストボディ

## シナリオ

### コレクションを新規作成する

1. クライアントが `POST /collections` に `name` と `schema`（JSON Schema）を送信する
2. API が名前の空文字チェックを行う
3. SurrealDB の `collection` テーブルにレコードを挿入する
4. 201 Created でコレクション情報を返す

### 名前が空のコレクションを作成しようとする

1. クライアントが `POST /collections` に空の `name` を送信する
2. API がバリデーションエラーを返す

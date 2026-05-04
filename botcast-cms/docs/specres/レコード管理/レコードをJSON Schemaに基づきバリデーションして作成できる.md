---
id: "01KNM2BBT558DMFWQ3DWXQR42B"
name: "レコードをJSON Schemaに基づきバリデーションして作成できる"
status: "stable"
---

## 関連ファイル

- `cms/crates/api/src/openapi_adapter.rs`
- `cms/crates/api/src/repository/mod.rs`
- `cms/crates/api/src/repository/surrealdb.rs`
- `cms/crates/openapi-gen/src/apis/record.rs`
- `spec/main.tsp`

## 機能概要

`POST /records/{collectionId}` でレコードデータを受け取り、コレクションの JSON Schema でバリデーション後、SurrealDB に格納する。スキーマに適合しないデータはバリデーションエラーとして拒否される。

## 意図

コレクションごとに定義された JSON Schema をレコード書き込みの際に強制することで、構造化データの整合性を保証する。

## 主要なメンバー

- `Record { id, data, created_at, updated_at }` — SurrealDB に格納されるレコードエンティティ
- `CreateOrUpdateRecord { data }` — レコード作成リクエストボディ
- `validate_record(schema, data)` — jsonschema クレートによるバリデーション関数

## シナリオ

### スキーマに適合するレコードを作成する

1. クライアントが `POST /records/{collectionId}` にデータを送信する
2. API がコレクションのスキーマを取得する
3. `validate_record()` でデータをスキーマに対して検証する
4. バリデーション成功後、SurrealDB にレコードを挿入する
5. 201 Created でレコード情報を返す

### スキーマに適合しないレコードを作成しようとする

1. クライアントがスキーマに適合しないデータを送信する
2. `validate_record()` がバリデーションエラーを検出する
3. 400 Bad Request で詳細なバリデーションエラーメッセージを返す

---
id: "01KNM2BBT51Q6B69XYQXPDTC0K"
name: "AppErrorがHTTPステータスコードに自動変換される"
status: "stable"
---

## 関連ファイル

- `cms/crates/api/src/error.rs`
- `cms/crates/api/src/openapi_adapter.rs`

## 機能概要

`AppError` 列挙型が thiserror で定義され、Validation(400)・NotFound(404)・Conflict(409)・Database(500)・Internal(500) のバリアントを持つ。ErrorHandler 実装により、アプリケーションエラーが自動的に適切な HTTP ステータスコードとエラーレスポンスボディに変換される。

## 意図

エラーハンドリングを型安全に一元管理し、リポジトリ層やハンドラ層で発生したエラーを HTTP レスポンスへ透過的にマッピングする。

## 主要なメンバー

- `AppError::Validation { message }` → 400 Bad Request
- `AppError::NotFound { resource }` → 404 Not Found
- `AppError::Conflict { message }` → 409 Conflict
- `AppError::Database(Error)` → 500 Internal Server Error
- `AppError::Internal { message }` → 500 Internal Server Error

## シナリオ

### バリデーションエラーが発生する

1. ハンドラがスキーマ検証に失敗する
2. `AppError::Validation` を返す
3. ErrorHandler が 400 ステータスコードとエラーメッセージを HTTP レスポンスに変換する

### 存在しないリソースにアクセスする

1. クライアントが存在しない ID を指定する
2. `AppError::NotFound` を返す
3. ErrorHandler が 404 ステータスコードを HTTP レスポンスに変換する

### データベースエラーが発生する

1. SurrealDB との通信に失敗する
2. `AppError::Database` が自動変換される
3. ErrorHandler が 500 ステータスコードで内部エラーを返す

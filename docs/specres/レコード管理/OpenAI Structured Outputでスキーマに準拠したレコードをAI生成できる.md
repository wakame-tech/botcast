---
id: "01KNM2BBT5MEY51QKNKHC4B176"
name: "OpenAI Structured Outputでスキーマに準拠したレコードをAI生成できる"
status: "stable"
---

## 関連ファイル

- `cms/crates/api/src/openapi_adapter.rs`
- `cms/crates/openapi-gen/src/apis/record.rs`
- `spec/main.tsp`

## 機能概要

`POST /records/{collectionId}/generate` でプロンプトを受け取り、OpenAI の gpt-4o-2024-08-06 を Structured Output モードで呼び出し、コレクションの JSON Schema に準拠したレコードを自動生成して保存する。

## 意図

LLM エージェントが自然言語の指示だけで構造化データを生成できるようにし、手動でのデータ入力を不要にする。

## 主要なメンバー

- `GenerateRecordRequest { prompt }` — 生成指示のプロンプト
- `ResponseFormatJsonSchema` — OpenAI API に渡すスキーマ定義
- `strict: true` — スキーマへの厳密な準拠を強制

## シナリオ

### プロンプトからレコードを生成する

1. クライアントが `POST /records/{collectionId}/generate` にプロンプトを送信する
2. API がコレクションの JSON Schema を取得する
3. OpenAI API に gpt-4o-2024-08-06 モデルで ChatCompletion リクエストを送信する
4. `ResponseFormat::JsonSchema` で strict モードを指定する
5. 生成された JSON を `validate_record()` で再検証する
6. バリデーション通過後、SurrealDB にレコードとして保存する
7. 201 Created で生成されたレコード情報を返す

### OpenAI API がコレクションスキーマに適合しないデータを返す

1. OpenAI API が strict モード不備等で不正な JSON を返す
2. `validate_record()` がバリデーションエラーを検出する
3. エラーレスポンスを返す

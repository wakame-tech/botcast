---
id: "01KNM2BBT5AY940G7M4HHBR26Q"
name: "Difyサンドボックスでスクリプトを実行できる"
status: "stable"
---

## 関連ファイル

- `cms/crates/api/src/openapi_adapter.rs`
- `cms/crates/openapi-gen/src/apis/script.rs`
- `spec/main.tsp`

## 機能概要

`POST /scripts` で Python3 または Node.js のコードを受け取り、Dify Sandbox コンテナ内で安全に実行して、標準出力とエラー出力を返す。オプションでネットワークアクセスとプリロードコードを指定できる。

## 意図

LLM エージェントが任意のコードを安全なサンドボックス環境で実行し、データ加工やスクリプト処理を行えるようにする。

## 主要なメンバー

- `DifySandboxClient { client, endpoint, api_key }` — Dify Sandbox への HTTP クライアント
- `ExecuteScriptRequest { language, code, preload, enable_network }` — 実行リクエスト
- `ExecuteScriptResponse { code, message, data: { stdout, error } }` — 実行結果

## シナリオ

### Python3 スクリプトを実行する

1. クライアントが `POST /scripts` に `language: "python3"` とコードを送信する
2. `DifySandboxClient` が Dify Sandbox の API にリクエストを転送する
3. Sandbox がコードを実行し、stdout と stderr を返す
4. 200 OK で実行結果を返す

### Node.js スクリプトを実行する

1. クライアントが `POST /scripts` に `language: "nodejs"` とコードを送信する
2. `DifySandboxClient` が Dify Sandbox の API にリクエストを転送する
3. Sandbox がコードを実行し、結果を返す
4. 200 OK で実行結果を返す

### スクリプト実行でエラーが発生する

1. クライアントが構文エラーを含むコードを送信する
2. Sandbox がエラーを検出する
3. レスポンスの `data.error` にエラー内容が格納される

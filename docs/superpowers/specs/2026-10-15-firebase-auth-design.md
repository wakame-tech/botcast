# ログインを Firebase Auth (Google) のみにする — 設計

## 目的

botcast-cms と botcast のメールアドレス + パスワード認証をやめ、Firebase Auth 経由の Google ログインのみにする。将来ログイン手段（GitHub など）を足すときは Firebase のコンソール設定だけで済むようにする。

## 決定事項

| 項目 | 決定 |
|---|---|
| IdP | Firebase Auth（初期は Google プロバイダのみ）。全テナントで 1 プロジェクトを共有 |
| CMS の認証方式 | クライアントが得た Firebase ID トークンを CMS が検証し、既存の CMS JWT に交換する |
| 新規登録 | 誰でも（初回ログインでユーザーを作成） |
| 既存ユーザー | 引き継がない。既存レコードの `owner_id` は移行で新ユーザーに付け替え、旧ユーザーは削除 |
| 追加ライブラリ | CMS: なし（`jsonwebtoken` 9 / `reqwest` を使う）。web: `firebase` |

変えないもの: CMS JWT（24h、`sub` / `tenant` / `exp`）、`owner_id`、`X-Api-Key` + `X-On-Behalf-Of`、worker。

## botcast-cms

### `POST /t/{tenantId}/auth/firebase`

- リクエスト `{ id_token: string }`、レスポンス `{ token: string }`（既存の `AuthResponse`）
- ID トークンの検証
  - ヘッダー `alg = RS256`、`kid` で JWKS の鍵を選ぶ
  - JWKS: `https://www.googleapis.com/service_accounts/v1/jwk/securetoken@system.gserviceaccount.com`。`Cache-Control: max-age` の間メモリにキャッシュし、未知の `kid` なら 1 回だけ取り直す
  - `iss = https://securetoken.google.com/<FIREBASE_PROJECT_ID>`、`aud = <FIREBASE_PROJECT_ID>`、`exp`、`sub` が空でない
- 検証後、テナントの `user` を `uid = sub` で upsert（`email` / `name` はトークンのクレームで更新）し、`issue_jwt(user:<id>)` を返す
- 検証失敗は 401。`FIREBASE_PROJECT_ID` 未設定は 500

### 削除

- `POST /auth/signup` / `/auth/signin` / `/auth/change-password` とそのハンドラー
- `DEFINE ACCESS user ...`、`user.pass`

### `user` テーブル

`SCHEMAFULL`: `uid: string`（UNIQUE インデックス）、`email: option<string>`、`name: option<string>`。旧フィールド（`pass`、`email` の UNIQUE インデックス）は移行で消す。

### 設定

- 環境変数 `FIREBASE_PROJECT_ID`（api。deploy の compose で既定 `botcast-cms`）。テスト用に JWKS の URL を `FIREBASE_JWKS_URL` で差し替えられる
- web の Firebase config（`apiKey` / `authDomain` / `projectId`）は公開値なので、`VITE_FIREBASE_*` で上書きできる既定値としてコードに置く。ビルド引数・リポジトリ変数は増やさない
- `user` のスキーマ変更（`pass` / `idx_user_email` / `DEFINE ACCESS` の削除、`uid` の追加）は起動時の `init_auth_schema` で冪等に行う

### CMS web

- サインイン画面を「Google でログイン」ボタンのみにする（`signInWithPopup(GoogleAuthProvider)` → `getIdToken()` → `/auth/firebase` → 既存の `signIn(token)`）
- Firebase のセッションは保持しない（`inMemoryPersistence`）。以後は CMS JWT のみ
- パスワード変更画面・ナビのリンクを削除

### その他

認証エンドポイントは TypeSpec の外（手書きのルート）なので openapi / MCP の再生成は不要。spec カード（認証・パスワード変更）、README / ARCHITECTURE / CLAUDE.md を更新。

## botcast

- `web/src/routes/signin.lazy.tsx` を Google ログインボタンのみにする（流れは CMS web と同じ。最後は既存の `saveToken`）
- `web/src/components/user/ChangePasswordForm.tsx` とその導線を削除
- ユーザー設定画面（`users/$userId`）はユーザー ID の表示のみにする
- CLAUDE.md と `crates/worker/src/lib.rs` の説明を更新

## 本番移行（cdfy-dev、1 回）

1. ユーザーが Firebase プロジェクトを作り、Google プロバイダを有効化し、承認済みドメインに cdfy-dev の CMS web / botcast web のホストを追加。Web アプリの config を共有する
2. cdfy-dev の CMS の compose に `FIREBASE_PROJECT_ID` を反映し、新 CMS / web をリリース
3. SurrealDB をバックアップ
4. ユーザーが botcast web で Google ログインし、新しい `user:<id>` を作る
5. 移行 SurrealQL（`deploy/selfhost/migrations/`）を流す
   - podcasts / episodes / scripts / mails の各テーブルで、旧ユーザー（`uid` が無い user）が持つレコードの `owner_id` を新ユーザーに付け替える
   - `DELETE user WHERE uid = NONE`

## テスト

- CMS: テスト用 RSA 鍵で署名した ID トークンと差し替えた JWKS で、正常 / `aud` 違い / `iss` 違い / 期限切れ / 未知の `kid` / 同じ `uid` の 2 回目ログインで同じ user になる
- web: 型チェック・lint・既存テスト。実ログインは Firebase 設定後に手動確認

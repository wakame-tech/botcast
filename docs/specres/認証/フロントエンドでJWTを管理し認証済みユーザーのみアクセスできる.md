---
id: "01KQS7YE9H3RWJSHBNWPEJGYAV"
name: "フロントエンドでJWTを管理し認証済みユーザーのみアクセスできる"
status: "stable"
---

## 関連ファイル

- `web/src/contexts/AuthContext.tsx`
- `web/src/pages/SignInPage.tsx`
- `web/src/App.tsx`
- `web/src/api/client.ts`
- `web/src/components/Layout.tsx`

## 機能概要

`AuthContext` が JWT トークンを localStorage で管理し、未認証ユーザーをサインインページへリダイレクトする。認証済みユーザーの API リクエストには `Authorization: Bearer <token>` ヘッダーを自動付与する。

## 意図

React アプリ全体で認証状態を一元管理し、保護されたルートへの未認証アクセスを防ぐ。

## 主要なメンバー

- `AuthContext` — `token`・`signIn`・`signOut`・`isAuthenticated` を提供する Context
- `AuthProvider` — localStorage からトークンを初期化し Context を配布する
- `useAuth()` — AuthContext を取得するカスタムフック
- `getStoredToken()` — API クライアントがトークンを取得するユーティリティ
- `ProtectedLayout` — 未認証時に `/signin` へリダイレクトするラッパー
- `SignInPage` — サインイン・サインアップフォーム

## シナリオ

### 未認証ユーザーが保護されたページにアクセスする

1. ユーザーが `/` などの保護されたパスにアクセスする
2. `ProtectedLayout` が `isAuthenticated === false` を検出する
3. `/signin` にリダイレクトされる

### ユーザーがサインインする

1. ユーザーがメールとパスワードをフォームに入力して送信する
2. `SignInPage` が `/auth/signin` を呼び出す
3. 取得した JWT を `AuthContext.signIn()` で localStorage に保存する
4. `/` にリダイレクトされて保護されたページを表示できる

### 認証済みユーザーが API を呼び出す

1. API クライアントが `getStoredToken()` でトークンを取得する
2. `Authorization: Bearer <token>` ヘッダーを付与してリクエストを送信する
3. バックエンドが JWT を検証してレスポンスを返す

### ユーザーがサインアウトする

1. ユーザーがヘッダーの「Sign Out」ボタンをクリックする
2. `AuthContext.signOut()` が localStorage からトークンを削除する
3. `/signin` にリダイレクトされる

---
id: "01KNM2BBT5T43CMWAX32H3K14G"
name: "管理画面でレコードの閲覧・作成・編集・AI生成・削除ができる"
status: "stable"
---

## 関連ファイル

- `web/src/pages/CollectionDetailPage.tsx`
- `web/src/utils/generateDummyData.ts`
- `web/src/api/client.ts`
- `web/src/types.ts`

## 機能概要

CollectionDetailPage で、コレクション内レコードの一覧表示、JSON 手動入力による新規作成（ダミーデータ自動生成付き）、プロンプトによる AI 生成、インライン JSON 編集、確認ダイアログ付き削除を提供する。スキーマの閲覧もサポートする。

## 意図

人間の管理者がレコードの詳細操作を直感的に行えるようにし、AI 生成機能で手動入力の手間を削減する。

## 主要なメンバー

- `generateDummyData(schema)` — JSON Schema からサンプルデータを再帰生成するユーティリティ

## シナリオ

### レコード一覧を表示する

1. ユーザーが `/collections/{collectionId}` にアクセスする
2. コレクション情報とレコード一覧を取得する
3. カード形式で ID・タイムスタンプ・JSON データを表示する

### ダミーデータ付きでレコードを手動作成する

1. ユーザーが作成フォームを開く
2. `generateDummyData()` がスキーマからサンプル JSON を生成する
3. ユーザーがデータを編集して送信する

### AI でレコードを生成する

1. ユーザーがプロンプトを入力して「AI 生成」ボタンを押す
2. `recordsApi.generate()` でバックエンドに送信する
3. 生成されたレコードが一覧に追加される

### レコードをインライン編集する

1. ユーザーがレコードの「編集」ボタンをクリックする
2. JSON データがテキストエリアで編集可能になる
3. 保存するとバリデーション後に更新される

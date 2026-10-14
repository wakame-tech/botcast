# CMS のデータ設計の見直し（参照・所有者・スキーマ） — 設計

## 目的

botcast が botcast-cms に置くコレクション（podcasts / episodes / scripts / mails）の設計を見直す。

- 参照フィールド（`podcast_id` など）を `x-ref` で宣言し、CMS web でリンクにし、逆引きを宣言済みのコレクションに絞る
- 所有者を `data.user_id`（クライアントが自由に書ける値）から CMS の `owner_id` に一本化し、他人のレコードを書き換えられないようにする
- スキーマを厳密にし、変更を本番に反映できるようにする

## 現状の問題

| # | 問題 | 影響 |
|---|---|---|
| 1 | `x-ref` 未宣言 | CMS web でリンクにならない。逆引きが全コレクションを走査する |
| 2 | 所有者が `data.user_id`。コレクションの `access` 未設定 | 認証済みなら誰でも他人の番組を編集・削除できる。worker が作る episode の `owner_id` は `service` |
| 3 | スキーマが緩い | `sections` の要素・`schedule.arguments` が未宣言、`additionalProperties` 未指定 |
| 4 | seed (`ensure_collection`) は作成のみ、CMS にスキーマ更新 API が無い | スキーマを直しても本番に反映されない |
| 5 | `mails.podcast_id` が冗長 | `episode_id` から辿れる値と食い違いうる |

## 決定事項

| 項目 | 決定 |
|---|---|
| 所有者 | CMS の `owner_id` に一本化。`data.user_id` は廃止 |
| worker が作る episode の所有者 | 番組の `owner_id`。API キー + `X-On-Behalf-Of` で代理作成する |
| アクセス制御 | 全コレクション `access: {read: public, write: owner}` |
| スキーマ変更の反映 | CMS の `PUT /collections/{id}` が `schema` を受け付ける。seed が差分を見て更新する |
| 既存データ | cdfy-dev の SurrealDB に移行 SurrealQL を 1 回流す |

## botcast-cms の変更

### スキーマ更新

- `UpdateCollection` に `schema?: Record<string>` を追加する
- スキーマが変わる場合、既存の全レコードを新スキーマで検証する。違反が 1 件でもあれば 400（違反したレコード ID をメッセージに含める）で、何も変えない
- `x-unique` が変わる場合は 400（インデックスの作り直しは対象外）
- `x-reactive` の宣言はスキーマ作成時と同じ検証を通す

### 代理実行 (`X-On-Behalf-Of`)

- API キー認証が成功し、`X-On-Behalf-Of: <user subject>` があれば、JWT の `sub` をその値にして以降を処理する（作成レコードの `owner_id`・アクセス判定ともにそのユーザー）
- JWT 認証ではヘッダーを無視する。API キーは元々全権を持つので権限は広がらない
- 値は `user:` で始まる subject のみ受け付ける（`service` などは 400）

## botcast のスキーマ

全コレクション `additionalProperties: false`、`access: {read: "public", write: "owner"}`。

- **podcasts**: `title`*, `icon`*, `description`, `schedule = {cron*, script_id* (x-ref: scripts), enabled*, arguments: object}`
- **episodes**: `title`*, `podcast_id`* (x-ref: podcasts), `description`, `sections` (要素は `oneOf` で `{type: "Serif", speaker, text}` / `{type: "Audio", url, from?, to?}`), `audio_url`, `srt_url`, `duration_sec`
- **scripts**: `title`*, `template`*, `description`, `arguments: object`（引数の JSON Schema）
- **mails**: `episode_id`* (x-ref: episodes), `radio_name`*, `body`*

（* は required）

`x-ref` の値はコレクション名ではなくコレクション ID（環境ごとに異なるランダムなキー）。よって seed は参照先から順に `scripts → podcasts → episodes → mails` と ensure し、得た ID をスキーマに埋め込む。

`ensure_collection(name, schema, access) -> id`: 無ければ作成、あればスキーマ・access を比べて差分があれば `CollectionApi_update` で更新する。更新が 400 なら起動を失敗させる。

## worker

- `GenerateEpisode` は番組の `owner_id` を `X-On-Behalf-Of` に付けて episode を作成する。`data.user_id` は書かない
- 番組の `owner_id` が無い（`service` 所有）場合はヘッダーを付けない

## web

- 作成時に `user_id` を送らない（podcast / script / mail）
- `Podcast` などの型から `user_id` を除き、`owner_id` を record から取る
- 編集・削除の導線は `owner_id === useSession().userId` のときだけ出す
- `npm run generate` で CMS の型を再生成する

## 移行（cdfy-dev、1 回）

1. 新 CMS をリリースし Watchtower の反映を待つ
2. worker を止める（`docker compose stop worker`）
3. SurrealQL を流す（テーブル名はコレクションのキー。`SELECT id, name FROM collection` で引く）
   - podcasts / episodes / scripts / mails: `UPDATE <table> SET owner_id = data.user_id, data.user_id = NONE WHERE data.user_id != NONE`
   - mails: `UPDATE <table> SET data.podcast_id = NONE`
4. 新 worker / web をデプロイする。起動時の seed がスキーマ・access を更新する。移行漏れがあれば 400 で起動に失敗して気付ける

## テスト

- CMS: スキーマ更新（成功・既存レコード違反で 400・`x-unique` 変更で 400）、`X-On-Behalf-Of`（API キー時のみ有効・owner_id が付く・JWT では無視・不正値 400）
- worker: `ensure_collection` の差分更新、episode 作成時のヘッダー
- web: 所有者以外に編集導線が出ない、作成時に `user_id` を送らない

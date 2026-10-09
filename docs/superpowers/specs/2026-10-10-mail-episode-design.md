# お便りを読み上げるエピソードの定期生成 — 設計

## 目的

リスナーが前回エピソード宛てに投稿したお便りを、CMS のスクリプト（Node.js）で台本にし、
番組ごとのスケジュールで新しいエピソードを生成・音声化する。

## 決定事項

| 項目 | 決定 |
|---|---|
| お便りの投稿 | web のエピソード画面から（サインイン済みユーザー） |
| 読むお便り | 前回エピソード（その番組で最新のエピソード）宛てのお便り |
| 生成の起動 | 定期実行（番組ごとの cron）。動作確認用に MCP ツールでの手動実行も用意 |
| スクリプトの役割 | context を受け取り台本（sections）を JSON で標準出力する。LLM は使わない |
| スケジュールの正本 | CMS の `podcasts.data.schedule`。worker が kafru のスケジュールへ同期する |

## データ構造（botcast-cms）

### `mails` コレクション（新規・worker の初期化処理で作成）

```json
{
  "podcast_id": "番組のレコード ID",
  "episode_id": "宛先エピソードのレコード ID",
  "radio_name": "ラジオネーム",
  "body": "本文",
  "user_id": "投稿者 (JWT の sub)"
}
```

必須: `podcast_id` / `episode_id` / `radio_name` / `body` / `user_id`。
アクセス制御は既定（読み取り public・書き込み authenticated）のまま。

### `podcasts.data.schedule`（任意項目）

```json
{ "cron": "0 0 12 * * Fri *", "script_id": "スクリプトのレコード ID", "enabled": true }
```

- `cron` は kafru の `CronSchedule` が使う Quartz 形式 7 フィールド
  （`秒 分 時 日 月 曜日 年`）。**UTC** で評価される（上の例は毎週金曜 21:00 JST）
- `podcasts` コレクションのスキーマに `schedule` (object) を追加する。既存環境に反映するため、worker の `McpClient::ensure_collection` を「無ければ作成、スキーマが異なれば `CollectionApi_update` で更新」に変更する

### スクリプト（既存の `scripts` レコード）

- `template`: Node.js のコード。`arguments`: スクリプトに渡す任意の値（object）
- 実行時に `preload` で次の変数が定義される

  ```js
  const context = {
    podcast,          // 番組レコードの data + id
    previous_episode, // 番組の最新エピソード (data + id)。無ければ null
    mails,            // previous_episode 宛てのお便り (data + id) の配列。previous_episode が null なら []
    arguments,        // scripts.data.arguments
  };
  ```

- 標準出力に次の JSON を 1 つ出す

  ```json
  { "title": "任意", "sections": [{ "type": "Serif", "speaker": "3", "text": "..." }] }
  ```

  - `sections` は既存の `Section`（`Serif` / `Audio`）の配列で、1 件以上
  - `title` を省いた場合は `第N回`（N = 生成前のその番組のエピソード数 + 1）

## worker

### ジョブ `GenerateEpisode { podcast_id }`

`task_service::Args` に追加し、既存の `execute_task` ハンドラで実行する。

1. 番組を取得し、`schedule.script_id` のスクリプトを取得する（`script_id` が無ければ失敗）
2. 前回エピソードを取得する: `episodes` を `filter=podcast_id:eq:<id>`・`sort=created_at`・`order=desc`・`limit=1`
3. お便りを取得する: `mails` を `filter=episode_id:eq:<前回エピソード ID>`
4. エピソード数を取得する（`title` 省略時の番号用）
5. CMS `POST /scripts` を `language: "nodejs"`・`preload: "const context = <JSON>;"`・`code: template` で呼ぶ
6. 応答の `data.error` が空でなければ失敗。`data.stdout` を `{ title?, sections }` として解釈し、失敗すれば失敗
7. `episodes` にレコードを作成する（`podcast_id`・`title`・`description: ""`・`sections`・`user_id` = 番組の `user_id`）
8. 作成したエピソードに対して既存の `EpisodeService::generate_audio` を同じジョブ内で実行する

### 構成

| ユニット | 役割 |
|---|---|
| `CmsClient`（拡張） | レコード一覧（filter / sort / order / limit）・レコード作成・スクリプト実行を追加 |
| `EpisodeGenerationService`（新規） | 上記 1〜7。context の組み立てと出力の解釈は純粋関数に分ける |
| `EpisodeService`（既存） | 音声生成 |
| `ScheduleSync`（新規） | CMS → kafru のスケジュール同期 |
| MCP ツール `generate_episode(podcast_id)`（新規） | `GenerateEpisode` ジョブを投入する |

### スケジュール同期 `ScheduleSync`

- worker 起動時と、その後 60 秒ごとに実行する
- CMS の `podcasts` を全件取得し、`schedule.enabled == true` かつ `cron`・`script_id` がある番組を対象にする
- kafru のスケジュール（名前 `podcast:<id>`）と比較して差分を適用する
  - 対象にあって kafru に無い → 作成（`handler: execute_task`・`parameters.args = GenerateEpisode`・`status: Enabled`）
  - 両方にあって cron が異なる → 更新
  - kafru にあって対象に無い → 削除
- 比較は純粋関数 `diff_schedules(desired, current) -> Vec<ScheduleChange>` に分ける
- cron の文字列は 7 フィールドに分割して `CronSchedule` の setter で組み立てる。フィールド数が違う番組は警告ログを出して対象外にする

### エラー時の扱い

- スクリプト失敗・JSON 不正・CMS のエラー → ジョブ失敗。エピソードは作らない
- エピソード作成後の音声生成の失敗 → ジョブ失敗。エピソードは台本付きで残り、`generate_audio` で再実行できる
- スケジュール同期の失敗 → ログに残し、次の周期で再試行する

## web

- **エピソード画面**: 「この回にお便りを送る」フォーム（ラジオネーム・本文）と、この回宛てのお便り一覧
- **番組の編集画面**: 定期実行の設定（cron・スクリプトの選択・有効/無効）
- 使われていない古い部品を削除: `components/corner/CornerList.tsx`・`components/mail/MailList.tsx`・`components/mail/MailForm.tsx`・`components/task/EvaluateScriptForm.tsx`、`lib/api_client.ts` の `CornerInputSchema` / `MailInputSchema`

## テスト

- 単体テスト（worker）
  - `preload` の組み立て
  - スクリプト出力の解釈: 正常 / `title` 省略 / 不正な JSON / `sections` が空 / `error` が空でない
  - `diff_schedules`: 作成・更新・削除・変化なし
  - cron 文字列の分割（7 フィールド・不正なフィールド数）
- ローカルでの通し確認（botcast-cms + VOICEVOX + kafru）
  1. 前回エピソード宛てにお便りを 2 件投稿する（web）
  2. MCP `generate_episode` → お便りを読み上げる新しい回ができ、音声が付く
  3. 番組の cron を数分後に設定 → スケジュールでも新しい回が生成される

## 対象外

- LLM による台本生成との組み合わせ（既存の `generate_script` は変更しない）
- お便りの既読管理・モデレーション
- botcast-cms 側の変更（既存 API で足りる）

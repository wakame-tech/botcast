---
name: sdd-specre
description: "Spec-Driven Development (SDD) using specre. Use this skill when implementing new features, fixing bugs, or refactoring code. The agent reads specre cards as the source of truth for behavior, writes/updates cards as part of development, and maintains bidirectional traceability between specs and source code."
---

# Spec-Driven Development with specre

## このスキルはいつ使うか

- 新機能を実装するとき
- 既存機能を修正・リファクタリングするとき
- 振る舞いの仕様を調べるとき
- specre カードを作成・更新するとき

## プロジェクトの specre 構成

```
specre.toml          ← specre 設定（specre_dir, source_dirs, ext, language）
glossary.toml        ← 検索ヒント用のドメイン用語集
docs/specres/        ← specre カード格納ディレクトリ
  {ドメイン}/
    {振る舞い名}.md  ← 1 ファイル = 1 つの振る舞い
```

## 基本原則

1. **specre カードが信頼できる唯一の情報源** — コードを変更する前に、関連する specre カードを読む
2. **1 ファイル = 1 つの振る舞い** — 「また、…」と書きたくなったら別カードに分割する
3. **双方向トレーサビリティ** — specre カードの「関連ファイル」とソースコードの `@specre` マーカーを常に同期する
4. **仕様とコードを同時に更新する** — 実装を変えたら specre カードも更新する。乖離を放置しない

## specre カードのフォーマット

```markdown
---
id: "{ULID}"
name: "{主語は振る舞いを記述する文}"
status: "draft"
---

## 関連ファイル

- `src/path/to/file.rs`
- `src/path/to/test.rs` (Test)

## 機能概要

振る舞いの 1 段落要約。

## 意図

何をではなく「なぜ」を説明する。

## 主要なメンバー

- `stateName: Type` — 説明

## シナリオ

### 正常系シナリオ名

1. ユーザーが〜する
2. システムが〜する
3. 結果として〜になる

### 異常系シナリオ名

1. ユーザーが〜する
2. システムがエラーを表示する
```

### front matter フィールド

| フィールド      | 型     | 必須 | 説明                                                        |
| --------------- | ------ | ---- | ----------------------------------------------------------- |
| `id`            | ULID   | Yes  | 26 文字の一意識別子。ソースコードとの双方向リンクに使う     |
| `name`          | string | Yes  | 振る舞いを記述する明確な主語と述語を持つ文                  |
| `status`        | enum   | Yes  | `draft` / `in-development` / `stable` / `deprecated`        |
| `last_verified` | date   | No   | YYYY-MM-DD。stable なカードで実装と一致を最後に確認した日付 |

### ステータスライフサイクル

```
draft ──→ in-development ──→ stable ──→ deprecated
  ↑            │                │
  └────────────┘                │
  (要件変更)                     ↓
                           (置換または削除)
```

## ソースコードの @specre マーカー

ソースファイルにコメントで `@specre {ULID}` を記述し、specre カードへの逆参照を作る。

Rust ファイルの場合:
```rust
// @specre 01KNM2BBT5Y18PC3WDQQN18KCA
pub fn create_collection() {
    // ...
}
```

TypeScript ファイルの場合:
```typescript
// @specre 01KNM2BBT598J55K9BMM975FCZ
export function CollectionsPage() {
  // ...
}
```

- 配置場所: ファイル先頭、クラス/関数定義の上、または関係性が最も明確な場所
- 1 ファイルで複数の specre を参照可能
- CLI は `@specre [0-9A-Z]{26}` パターンでスキャンして検出する

## ワークフロー

### A. 新機能を実装する (SDD-New)

1. **specre カードを書く** — `docs/specres/{ドメイン}/{振る舞い名}.md` を作成。status は `draft`
   - ULID を生成して `id` に設定する
   - 関連ファイル（これから作るファイル）を記載する
   - 機能概要・シナリオを記述する
2. **実装する** — specre カードのシナリオに従ってコードを書く
3. **テストを書く** — specre カードのシナリオに対応するテストを作成する
   - テストファイルの先頭に `// @specre {ULID}` を追加する
   - specre カードの「関連ファイル」にテストファイルを `(Test)` マーク付きで追加する
4. **@specre マーカーを追加する** — 実装したソースファイルに `// @specre {ULID}` を記述する
5. **status を更新する** — `draft` → `in-development`（実装中）→ `stable`（完了・テスト通過）

### B. 既存機能を修正する (SDD-Fix)

1. **関連する specre カードを特定する**
   - 修正対象のソースファイルに `@specre` マーカーがあれば、そのカードを読む
   - マーカーがない場合は `docs/specres/` から関連カードを検索する
2. **specre カードのシナリオを確認する** — 現在の仕様を理解する
3. **修正する** — シナリオに照らして修正の影響範囲を判断する
4. **テストを更新する** — 振る舞いが変わった場合は既存テストを修正する
5. **specre カードを更新する** — 振る舞いが変わった場合はシナリオ・機能概要を更新する

### C. 振る舞いを調べる

1. **ソースファイルから specre カードを辿る** — `@specre` マーカーの ULID で `docs/specres/` 内を検索
2. **specre カードからソースを辿る** — 「関連ファイル」セクションのパスを開く
3. **ドメインから探す** — `docs/specres/{ドメイン}/` ディレクトリを一覧して該当カードを見つける

## ULID の生成方法

JavaScript で簡易生成できる:

```javascript
const chars = '0123456789ABCDEFGHJKMNPQRSTVWXYZ';
const now = Date.now();
function encodeTime(t) {
  let s = '';
  for (let i = 9; i >= 0; i--) { s = chars[t % 32] + s; t = Math.floor(t / 32); }
  return s;
}
function randomPart() {
  let s = '';
  for (let i = 0; i < 16; i++) { s += chars[Math.floor(Math.random() * 32)]; }
  return s;
}
console.log(encodeTime(now) + randomPart());
```

## このプロジェクトのドメイン構成

| ドメイン           | 概要                                                 | specre カード数 |
| ------------------ | ---------------------------------------------------- | --------------- |
| コレクション管理   | JSON Schema でのコレクション作成・一覧・更新・削除   | 2               |
| レコード管理       | スキーマバリデーション付き CRUD・AI 生成             | 3               |
| スクリプト実行     | Dify Sandbox での Python3/Node.js コード実行         | 1               |
| ジョブ管理         | Kafru ベースのバックグラウンドジョブキュー           | 1               |
| エラーハンドリング | AppError → HTTP ステータスコード自動変換             | 1               |
| MCP                | OpenAPI → MCP ツール公開                             | 1               |
| 管理画面           | React 管理ダッシュボード（コレクション・レコード等） | 4               |
| 基盤               | DI・サーバー初期化・TypeSpec→OpenAPI コード生成      | 2               |

## 自動生成ファイルの扱い

以下のファイルは OpenAPI 仕様から自動生成されるため、`@specre` マーカーを配置してはならない。再生成時にマーカーが消失する。

| 生成コマンド                              | 生成先                         |
| ----------------------------------------- | ------------------------------ |
| `just gen_server` (openapi-generator-cli) | `cms/crates/openapi-gen/` 全体 |
| `just gen_mcp` (openapi-mcp-generator)    | `mcp/src/index.ts`             |
| `tsp compile .` (TypeSpec)                | `spec/tsp-output/`             |

### 対処方法

- specre カードの「関連ファイル」に自動生成ファイルを記載する場合は `(Generated)` マークを付ける
- `@specre` マーカーは手動管理のソースファイル（`openapi_adapter.rs` 等）に配置する
- 自動生成ファイルの振る舞いを追跡する場合は、その生成元（`spec/main.tsp`）や実装先（`openapi_adapter.rs`）にマーカーを置く

## やってはいけないこと

- specre カードを読まずにコードを大幅変更する
- 実装を変えたのに specre カードを更新しない
- 1 枚のカードに複数の振る舞いを詰め込む
- ファイル名に連番（001_, 002_）を付ける
- シナリオにコードをそのまま貼り付ける（自然言語で記述する）
- `index.json` を手動で編集する（`specre index` で再生成するもの）

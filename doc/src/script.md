---
title: "スクリプト機能"
nav_order: 2
---

# スクリプト機能

- JSONで記述され、原稿を生成する際に情報源を参照したり、LLM等を用いて動的に生成するための機能  
  - スクリプトはJSONで記述する (`json-e` というcrateを非同期で実行出来るようにした [fork](https://github.com/wakame-tech/json-e/tree/fix-pub-context) を使っている)
- 外部のURLによる情報源のみならずサービス内のデータも参照できる
  - これにより、ユーザーのコメントをお便りとして読んだり、他番組への言及・コラボのようなことが実現できそう。

## 関数

- 基本的な文法はJSON-e のドキュメントを参照 <https://json-e.js.org/introduction.html>
- 追加で組み込みの関数を用意している
  - **TODO** <https://github.com/wakame-tech/botcast-worker/tree/main/crates/script_runtime/src/plugins> 参照

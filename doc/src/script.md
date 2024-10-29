---
title: "スクリプト機能"
nav_order: 2
---

# スクリプト機能

- JSONで記述され、原稿を生成する際に情報源を参照したり、LLM等を用いて動的に生成するための機能  
- 外部のURLによる情報源のみならずサービス内のデータも参照できる
  - これにより、ユーザーのコメントをお便りとして読んだり、他番組への言及・コラボのようなことが実現できそう。

## ランタイム

- [JSON-e](https://json-e.js.org/introduction.html) として実行される
  - 非同期で実行出来るようにした [github](https://github.com/wakame-tech/json-e/tree/fix-pub-context)
  - TODO: タイムアウトをつける
- 組み込みの関数が用意されていて利用することが出来る
  - `today(format)`: 現在時刻(日付)
  - `get(urn: Urn)`: サービス内のデータを URN(`urn:<resource>:<id>`) で取得出来る
    - 例えばコメント, 原稿等
  - `fetch(url), fetch_json(url)`: 外部のURLをHTML, JSONとして取得
  - `jq(json)`: JSONを加工
  - `llm(prompt)`: LLM呼び出し, 内部実装は [langchain-rs](https://github.com/Abraxas-365/langchain-rust)

## 原稿

スクリプトの評価結果が `Manuscript` の型になるものを原稿と呼ぶ。
ポッドキャストとエピソードは原稿になるスクリプトを結びつけることができる。
エピソードの原稿は基本的にポッドキャストの原稿を引き継ぐが、エピソードごとにスクリプトを設定することもできる。
音声生成時に `Section` を適切な長さに区切ったり、読みを生成(TODO)したりする

```typescript
type Section =
    {
        type: 'Serif',
        speaker: string,
        text: string,
    }

interface Manuscript {
    title: string;
    sections: Section[];
}
```

### 原稿となるスクリプトの例

```json
{
    "$let": {
        "speaker": "urn:voicevox:zunda_normal",
        "today": { "$eval": "today('%Y年%m月%d日')" },
        "novel": { "$eval": "text(fetch('https://www.aozora.gr.jp/cards/000081/files/45630_23908.html'))" }
    },
    "in": {
        "title": "ポッドキャスト ${today} 号",
        "sections": [
            {
                "type": "serif",
                "speaker": { "$eval": "${speaker}" },
                "text": {
                  "$eval": "こんにちは、今日は${today}です。突然ですが雨ニモマケズの概要を話します。"
                }
            },
            {
                "type": "serif",
                "speaker": { "$eval": "${speaker}" },
                "text": {
                    "$eval": "llm('次の文章を要約してください\n${novel}')"
                }
            }
        ]
    }
}
```

### 話数からタイトルをつける

- `self` でポッドキャスト自身のURNを取得できる

```json
{
    "$let": {
        "num": {
            "$eval": "str(len(get(self).episodes) + 1)"
        }
    },
    "in": {
        "title": { "$eval": "第${num}話" },
        "sections": []
    }
}
```

### 1つ前のエピソードを取得する

```json
{
    "$let": {
        "episodes_with_index": {
            "$map": {
                "$eval": "get('urn:podcast:${podcastId}').episodes"
            },
            "each(v,i)": {
                "index": { "$eval": "i" },
                "value": { "$eval": "v" }
            }
        }
    },
    "in": {
        "$let": {
            "i": {
                "$find": { "$eval": "episodes_with_index" },
                "each(e)": "e.value.id == episodeId"
            }
        },
        "in": {
            "$let": {
                "preEpisodeId": {
                    "$if": "i.index == 0",
                    "then": null,
                    "else": {
                        "$eval": "episodes_with_index[i.index - 1].value.id"
                    }
                }
            },
            "in": {
                "$eval": "get('urn:episode:${preEpisodeId}')"
            }
        }
    }
}
```

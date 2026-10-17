//! エンジン非依存の本文の処理（文の分割・絵文字の除去）。

const DELIMITERS: [char; 2] = ['\n', '。'];

/// 1 回の合成に渡す文の最大の文字数。
pub const MAX_SENTENCE_CHARS: usize = 100;

/// 文中の `http://` / `https://` で始まる URL を取り除く（URL は ASCII の空白以外の文字が続くところまで）。
pub fn strip_urls(text: &str) -> String {
    let mut res = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(start) = ["http://", "https://"]
        .iter()
        .filter_map(|scheme| rest.find(scheme))
        .min()
    {
        res.push_str(&rest[..start]);
        let url_len = rest[start..]
            .find(|c: char| !c.is_ascii_graphic())
            .unwrap_or(rest.len() - start);
        rest = &rest[start + url_len..];
    }
    res.push_str(rest);
    res
}

/// `max_chars` を超える 1 文を、読点（`、`）があればそこで、無ければ文字の境界で分ける。
fn split_long_sentence(sentence: &str, max_chars: usize) -> Vec<String> {
    let max_chars = max_chars.max(1);
    let mut res = vec![];
    let mut buf = String::new();
    let mut buf_chars = 0;
    for piece in sentence.split_inclusive('、') {
        let chars: Vec<char> = piece.chars().collect();
        if !buf.is_empty() && buf_chars + chars.len() > max_chars {
            res.push(std::mem::take(&mut buf));
            buf_chars = 0;
        }
        // 読点の間だけで上限を超えるときは文字の境界で切る
        for chunk in chars.chunks(max_chars) {
            if buf_chars + chunk.len() > max_chars {
                res.push(std::mem::take(&mut buf));
                buf_chars = 0;
            }
            buf.extend(chunk);
            buf_chars += chunk.len();
        }
    }
    if !buf.is_empty() {
        res.push(buf);
    }
    res
}

/// 本文から URL を取り除き、`。` と改行で区切り、`max_chars` 文字（バイト数ではない）を超えない範囲で文をまとめる。
///
/// 1 文だけで `max_chars` を超えるときは、読点か文字の境界で分ける。
pub fn split_text(text: &str, max_chars: usize) -> Vec<String> {
    let text = strip_urls(text);
    let mut buf = String::new();
    let mut buf_chars = 0;
    let mut res = vec![];
    let sentences = text.split_inclusive(DELIMITERS).flat_map(|sentence| {
        let sentence = sentence.trim();
        if sentence.chars().count() > max_chars {
            split_long_sentence(sentence, max_chars)
        } else {
            vec![sentence.to_string()]
        }
    });
    for sentence in sentences {
        // URL を取り除いて区切りだけ残った文も読まない
        if sentence.trim_matches(DELIMITERS).trim().is_empty() {
            continue;
        }
        let chars = sentence.chars().count();
        if !buf.is_empty() && buf_chars + chars > max_chars {
            res.push(std::mem::take(&mut buf));
            buf_chars = 0;
        }
        buf.push_str(&sentence);
        buf_chars += chars;
    }
    if !buf.is_empty() {
        res.push(buf);
    }
    res
}

fn is_emoji(c: char) -> bool {
    matches!(
        c as u32,
        0x1F000..=0x1FAFF // 絵文字・記号・国旗・肌の色
            | 0x2600..=0x27BF // その他の記号・装飾記号
            | 0x2B00..=0x2BFF // 矢印などの記号（⭐ など）
            | 0x2300..=0x23FF // 技術記号（⌚ ⏰ など）
            | 0xFE00..=0xFE0F // 異体字セレクタ
            | 0x200D // ゼロ幅接合子
            | 0x20E3 // 囲み記号（キーキャップ）
            | 0xE0020..=0xE007F // タグ文字
    )
}

/// 絵文字を取り除く（VOICEVOX 互換のエンジンに渡す前に使う）。
pub fn strip_emoji(text: &str) -> String {
    text.chars().filter(|c| !is_emoji(*c)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_counts_chars_not_bytes() {
        // 30 文字の文 × 3 は 90 文字なので 1 つにまとまる（バイト数では 270 バイト）
        let sentence = format!("{}。", "あ".repeat(29));
        let text = sentence.repeat(3);
        assert_eq!(split_text(&text, MAX_SENTENCE_CHARS), vec![text.clone()]);

        // 4 文目で 100 文字を超えるので分かれる
        let text = sentence.repeat(4);
        let res = split_text(&text, MAX_SENTENCE_CHARS);
        assert_eq!(res, vec![sentence.repeat(3), sentence.clone()]);
        assert!(res.iter().all(|s| s.chars().count() <= MAX_SENTENCE_CHARS));
    }

    #[test]
    fn split_by_delimiters_and_strip_urls() {
        let text = "こんにちは。\nhttps://example.com\n今日は晴れ。\n\n";
        assert_eq!(split_text(text, 6), vec!["こんにちは。", "今日は晴れ。"]);
        // 文中の URL は URL の部分だけを取り除く
        let text = "詳しくは https://example.com/a?b=1 を見て。http://x.jp。";
        assert_eq!(
            split_text(text, MAX_SENTENCE_CHARS),
            vec!["詳しくは  を見て。"]
        );
        // http で始まるだけの語は URL ではない
        assert_eq!(
            split_text("httpとは何か。", MAX_SENTENCE_CHARS),
            vec!["httpとは何か。"]
        );
    }

    #[test]
    fn strip_urls_stops_at_non_ascii() {
        assert_eq!(strip_urls("見て→https://example.com。"), "見て→。");
        assert_eq!(strip_urls("a http://x b https://y"), "a  b ");
        assert_eq!(strip_urls("URL なし"), "URL なし");
    }

    #[test]
    fn long_sentence_is_split_at_commas() {
        // 読点で分けられるときは読点で分ける
        let part = format!("{}、", "あ".repeat(59));
        let text = format!("{part}{part}{}。", "い".repeat(9));
        let res = split_text(&text, MAX_SENTENCE_CHARS);
        assert_eq!(
            res,
            vec![part.clone(), format!("{part}{}。", "い".repeat(9))]
        );
        assert!(res.iter().all(|s| s.chars().count() <= MAX_SENTENCE_CHARS));
        assert_eq!(res.concat(), text);
    }

    #[test]
    fn long_sentence_without_commas_is_split_by_chars() {
        let long = "あ".repeat(250);
        let res = split_text(&long, MAX_SENTENCE_CHARS);
        assert_eq!(
            res.iter().map(|s| s.chars().count()).collect::<Vec<_>>(),
            vec![100, 100, 50]
        );
        assert_eq!(res.concat(), long);

        // 長い文の後ろの短い文は、余りとまとめる
        let text = format!("{long}。短い。");
        let res = split_text(&text, MAX_SENTENCE_CHARS);
        assert_eq!(res.concat(), text);
        assert!(res
            .iter()
            .all(|s| !s.is_empty() && s.chars().count() <= MAX_SENTENCE_CHARS));
    }

    #[test]
    fn strip_emoji_keeps_japanese() {
        assert_eq!(
            strip_emoji("やった🎉！ありがとう😊👍🏻"),
            "やった！ありがとう"
        );
        assert_eq!(strip_emoji("家族👨‍👩‍👧と❤️"), "家族と");
        assert_eq!(
            strip_emoji("「こんにちは」、ー〜。"),
            "「こんにちは」、ー〜。"
        );
    }
}

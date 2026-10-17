//! エンジン非依存の本文の処理（文の分割・絵文字の除去）。

const DELIMITERS: [char; 2] = ['\n', '。'];

/// 1 回の合成に渡す文の最大の文字数。
pub const MAX_SENTENCE_CHARS: usize = 100;

/// 本文を `。` と改行で区切り、`max_chars` 文字（バイト数ではない）を超えない範囲で文をまとめる。
///
/// 1 文だけで `max_chars` を超えるときはそのまま 1 つにする。`http` で始まる文（URL）は読まない。
pub fn split_text(text: &str, max_chars: usize) -> Vec<String> {
    let mut buf = String::new();
    let mut buf_chars = 0;
    let mut res = vec![];
    for sentence in text.split_inclusive(DELIMITERS) {
        let sentence = sentence.trim();
        if sentence.is_empty() || sentence.starts_with("http") {
            continue;
        }
        let chars = sentence.chars().count();
        if !buf.is_empty() && buf_chars + chars > max_chars {
            res.push(std::mem::take(&mut buf));
            buf_chars = 0;
        }
        buf.push_str(sentence);
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
    fn split_by_delimiters_and_skip_urls() {
        let text = "こんにちは。\nhttps://example.com\n今日は晴れ。\n\n";
        assert_eq!(split_text(text, 6), vec!["こんにちは。", "今日は晴れ。"]);
    }

    #[test]
    fn long_sentence_is_kept_without_empty_chunk() {
        let long = "あ".repeat(120);
        assert_eq!(split_text(&long, MAX_SENTENCE_CHARS), vec![long.clone()]);
        let text = format!("{long}。短い。");
        assert_eq!(
            split_text(&text, MAX_SENTENCE_CHARS),
            vec![format!("{long}。"), "短い。".to_string()]
        );
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

-- パスワード認証の旧ユーザー (uid が無い user) が持つレコードを、Firebase でログインした新ユーザーに付け替えて旧ユーザーを消す。
-- 新ユーザーがちょうど 1 人いる状態で流す (先に botcast web で Google ログインを 1 回済ませておく)。違えば THROW で全体を取り消す。
-- uid = NONE は uid の UNIQUE インデックス経由だと NONE の行を拾えないので type::is::none で判定する。
BEGIN TRANSACTION;
LET $new = (SELECT VALUE <string> id FROM user WHERE !type::is::none(uid));
IF array::len($new) != 1 { THROW "Firebase でログインした user がちょうど 1 人である必要があります" };
LET $old = (SELECT VALUE <string> id FROM user WHERE type::is::none(uid));
FOR $c IN (SELECT id, name FROM collection WHERE name INSIDE ['podcasts', 'episodes', 'scripts', 'mails']) {
    UPDATE type::table(record::id($c.id)) SET owner_id = $new[0] WHERE owner_id INSIDE $old;
};
DELETE user WHERE type::is::none(uid);
COMMIT TRANSACTION;

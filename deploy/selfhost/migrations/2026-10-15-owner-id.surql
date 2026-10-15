-- 2026-10-15: 所有者を data.user_id から CMS の owner_id に移し、mails の data.podcast_id を消す。
-- 新しい worker を起動する前に、CMS の SurrealDB (namespace = テナント ID) に対して 1 回流す。
-- レコードはコレクションのキーを名前とするテーブルにある。
FOR $c IN (SELECT id, name FROM collection WHERE name INSIDE ['podcasts', 'episodes', 'scripts', 'mails']) {
    LET $t = type::table(record::id($c.id));
    UPDATE $t SET owner_id = data.user_id, data.user_id = NONE WHERE data.user_id != NONE;
    IF $c.name = 'mails' {
        UPDATE $t SET data.podcast_id = NONE;
    };
};

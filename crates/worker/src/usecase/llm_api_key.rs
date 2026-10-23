//! 台本生成などで LLM（Anthropic）を呼ぶときの API キーを決める（BYOK）。
//!
//! キーはジョブの持ち主（番組の `owner_id`）が CMS の `/me/secrets` に保存した `anthropic_api_key`。
//! worker はサービスの API キーで `X-On-Behalf-Of: <持ち主>` を付けて値を取り出し、そのジョブの間だけ使う。
//! 環境変数のキーへのフォールバックはしない。値は保存せず、ログ・span・エラーに出さない（[`ApiKey`]）

use super::cms_client::CmsClient;
use async_trait::async_trait;
use std::fmt;

/// CMS の秘密情報の名前（web のマイページの「LLM の API キー」と同じ）
pub(crate) const ANTHROPIC_API_KEY_SECRET: &str = "anthropic_api_key";

/// 持ち主がキーを設定していないときのジョブのエラー
pub(crate) const MISSING_API_KEY_MESSAGE: &str = "LLM の API キーが設定されていません。番組の持ち主のマイページの「設定」で「LLM の API キー」を登録してください";

/// LLM の API キー。`Debug` / `Display` は値を出さない
#[derive(Clone, PartialEq, Eq)]
pub(crate) struct ApiKey(String);

impl ApiKey {
    pub(crate) fn new(value: String) -> Self {
        Self(value)
    }

    /// リクエストのヘッダーに入れるときだけ使う
    pub(crate) fn expose(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for ApiKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ApiKey(<redacted>)")
    }
}

impl fmt::Display for ApiKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("<redacted>")
    }
}

#[async_trait]
pub(crate) trait SecretStore: Send + Sync {
    /// `owner` の秘密情報 `name` の値。設定が無ければ `None`
    async fn user_secret(&self, owner: &str, name: &str) -> anyhow::Result<Option<ApiKey>>;
}

#[async_trait]
impl SecretStore for CmsClient {
    async fn user_secret(&self, owner: &str, name: &str) -> anyhow::Result<Option<ApiKey>> {
        self.secret_value_as(owner, name).await
    }
}

/// ジョブの持ち主（番組の `owner_id`）の LLM の API キーを取り出す。
/// 人間のユーザー（`user:`）でない・キーが無いときは [`MISSING_API_KEY_MESSAGE`] のエラー
#[tracing::instrument(name = "llm.resolve_api_key", skip_all, err)]
pub(crate) async fn resolve_api_key(
    store: &dyn SecretStore,
    owner: Option<&str>,
) -> anyhow::Result<ApiKey> {
    let Some(owner) = owner.filter(|o| o.starts_with("user:")) else {
        anyhow::bail!("{MISSING_API_KEY_MESSAGE}（番組の持ち主がユーザーではありません）");
    };
    store
        .user_secret(owner, ANTHROPIC_API_KEY_SECRET)
        .await?
        .filter(|key| !key.expose().trim().is_empty())
        .ok_or_else(|| anyhow::anyhow!(MISSING_API_KEY_MESSAGE))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::sync::Mutex;

    const KEY: &str = "sk-ant-test-0123456789abcdef";

    struct FakeStore {
        secrets: HashMap<(String, String), String>,
        asked: Mutex<Vec<String>>,
    }

    impl FakeStore {
        fn new(entries: &[(&str, &str, &str)]) -> Self {
            Self {
                secrets: entries
                    .iter()
                    .map(|(o, n, v)| ((o.to_string(), n.to_string()), v.to_string()))
                    .collect(),
                asked: Default::default(),
            }
        }
    }

    #[async_trait]
    impl SecretStore for FakeStore {
        async fn user_secret(&self, owner: &str, name: &str) -> anyhow::Result<Option<ApiKey>> {
            self.asked.lock().unwrap().push(owner.to_string());
            Ok(self
                .secrets
                .get(&(owner.to_string(), name.to_string()))
                .cloned()
                .map(ApiKey::new))
        }
    }

    #[tokio::test]
    async fn uses_the_owner_key() {
        let store = FakeStore::new(&[
            ("user:a", ANTHROPIC_API_KEY_SECRET, KEY),
            ("user:b", ANTHROPIC_API_KEY_SECRET, "sk-other"),
        ]);
        let key = resolve_api_key(&store, Some("user:a")).await.unwrap();
        assert_eq!(key.expose(), KEY);
        assert_eq!(*store.asked.lock().unwrap(), vec!["user:a".to_string()]);
    }

    #[tokio::test]
    async fn fails_with_a_clear_message_without_a_key() {
        let store = FakeStore::new(&[
            ("user:b", ANTHROPIC_API_KEY_SECRET, KEY),
            ("user:blank", ANTHROPIC_API_KEY_SECRET, "  "),
        ]);
        for owner in [Some("user:a"), Some("user:blank"), Some("service"), None] {
            let err = resolve_api_key(&store, owner).await.unwrap_err();
            assert!(
                format!("{err:#}").contains("マイページ"),
                "{owner:?}: {err:#}"
            );
        }
        // サービス所有・持ち主不明の番組は CMS に聞かない
        assert_eq!(
            *store.asked.lock().unwrap(),
            vec!["user:a".to_string(), "user:blank".to_string()]
        );
    }

    #[test]
    fn key_is_redacted_in_debug_and_display() {
        let key = ApiKey::new(KEY.to_string());
        for s in [
            format!("{key:?}"),
            format!("{key}"),
            format!("{:?}", Some(key.clone())),
            format!("{:#}", anyhow::anyhow!("failed with {key}")),
        ] {
            assert!(!s.contains("sk-ant"), "{s}");
        }
    }
}

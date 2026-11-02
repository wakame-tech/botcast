//! worker が Web を読むための取得（[`agent_kit::safe_fetch`]）。取得してよいホストは CMS の `fetch_allowed_hosts`

pub(crate) use agent_kit::safe_fetch::*;

use super::cms_client::{CmsClient, ListQuery};
use anyhow::Context;
use async_trait::async_trait;
use serde_json::Value;

/// 取得してよいホストの一覧の CMS のコレクション
pub(crate) const FETCH_ALLOWED_HOSTS: &str = "fetch_allowed_hosts";

/// 許可リストとして読むレコードの数の上限
const ALLOWED_HOSTS_LIMIT: u32 = 200;

/// 取得のリクエストの `User-Agent`
pub(crate) const USER_AGENT: &str = "botcast-worker (+https://github.com/wakame-tech/botcast)";

#[async_trait]
impl HostAllowlist for CmsClient {
    async fn allowed_hosts(&self) -> anyhow::Result<Vec<String>> {
        let records = self
            .list_records(
                FETCH_ALLOWED_HOSTS,
                &ListQuery {
                    limit: Some(ALLOWED_HOSTS_LIMIT),
                    ..Default::default()
                },
            )
            .await
            .context("Failed to list allowed hosts")?;
        Ok(hosts_from_records(&records))
    }
}

/// `fetch_allowed_hosts` のレコードから `enabled` のホストを取り出す
pub(crate) fn hosts_from_records(records: &[Value]) -> Vec<String> {
    records
        .iter()
        .filter(|r| r["data"]["enabled"].as_bool() == Some(true))
        .filter_map(|r| r["data"]["host"].as_str())
        .map(normalize_host)
        .filter(|h| !h.is_empty())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn hosts_from_records_takes_enabled_hosts() {
        let records = vec![
            json!({ "id": "a", "data": { "host": "Zenn.dev", "enabled": true } }),
            json!({ "id": "b", "data": { "host": "off.example", "enabled": false } }),
            json!({ "id": "c", "data": { "host": "noflag.example" } }),
        ];
        assert_eq!(hosts_from_records(&records), vec!["zenn.dev".to_string()]);
    }
}

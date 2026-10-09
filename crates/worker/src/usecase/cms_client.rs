use anyhow::{bail, Context};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use std::collections::HashMap;
use tokio::sync::Mutex;

/// `GET /records/{collection}` のクエリ
#[derive(Debug, Default, Clone)]
pub(crate) struct ListQuery {
    /// `<field>:<op>:<value>` 形式 (1 件のみ)
    pub filter: Option<String>,
    pub sort: Option<String>,
    pub order_desc: bool,
    pub limit: Option<u32>,
}

impl ListQuery {
    fn to_params(&self) -> Vec<(&'static str, String)> {
        let mut params = vec![];
        if let Some(f) = &self.filter {
            params.push(("filter", f.clone()));
        }
        if let Some(s) = &self.sort {
            params.push(("sort", s.clone()));
        }
        if self.order_desc {
            params.push(("order", "desc".to_string()));
        }
        if let Some(l) = self.limit {
            params.push(("limit", l.to_string()));
        }
        params
    }
}

/// `POST /scripts` の結果
#[derive(Debug, Clone)]
pub(crate) struct ScriptOutput {
    pub stdout: String,
    pub error: String,
}

/// botcast-cms の REST API クライアント。
///
/// - 認証は `X-Api-Key`（環境変数 `API_KEY` = botcast-cms の API_KEY と同じ値）によるサービス間認証
/// - collection ID は自動採番のため、名前から ID を `/collections` で解決してキャッシュする
/// - 音声・字幕ファイルは `/records/{c}/{r}/images/{field}` に Base64 で保存する
pub(crate) struct CmsClient {
    http: reqwest::Client,
    base_url: String,
    api_key: Option<String>,
    collection_ids: Mutex<HashMap<String, String>>,
}

impl CmsClient {
    pub(crate) fn from_env() -> Self {
        let base_url =
            std::env::var("CMS_URL").unwrap_or_else(|_| "http://localhost:3002".to_string());
        let api_key = std::env::var("API_KEY").ok();
        Self::new(base_url, api_key)
    }

    pub(crate) fn new(base_url: String, api_key: Option<String>) -> Self {
        Self {
            http: reqwest::Client::new(),
            base_url: base_url.trim_end_matches('/').to_string(),
            api_key,
            collection_ids: Mutex::new(HashMap::new()),
        }
    }

    fn request(&self, method: reqwest::Method, path: &str) -> reqwest::RequestBuilder {
        let req = self.http.request(method, format!("{}{}", self.base_url, path));
        match &self.api_key {
            Some(key) => req.header("X-Api-Key", key),
            None => req,
        }
    }

    async fn send(req: reqwest::RequestBuilder) -> anyhow::Result<serde_json::Value> {
        let res = req.send().await.context("CMS request failed")?;
        let status = res.status();
        if !status.is_success() {
            let body = res.text().await.unwrap_or_default();
            bail!("CMS returned {status}: {body}");
        }
        res.json().await.context("Failed to parse CMS response")
    }

    /// collection 名を ID（`collection:` 接頭辞なし）に解決する
    pub(crate) async fn collection_id(&self, name: &str) -> anyhow::Result<String> {
        let mut ids = self.collection_ids.lock().await;
        if let Some(id) = ids.get(name) {
            return Ok(id.clone());
        }
        let collections = Self::send(self.request(reqwest::Method::GET, "/collections")).await?;
        *ids = parse_collection_ids(&collections);
        ids.get(name)
            .cloned()
            .with_context(|| format!("CMS collection '{name}' not found"))
    }

    /// レコード全体 (`id` / `data` / `owner_id` / ...) を取得する
    pub(crate) async fn get_record(
        &self,
        collection: &str,
        record_id: &str,
    ) -> anyhow::Result<serde_json::Value> {
        let cid = self.collection_id(collection).await?;
        Self::send(self.request(reqwest::Method::GET, &format!("/records/{cid}/{record_id}")))
            .await
    }

    /// レコードの `data` を取得する
    pub(crate) async fn get_record_data(
        &self,
        collection: &str,
        record_id: &str,
    ) -> anyhow::Result<serde_json::Value> {
        Ok(self.get_record(collection, record_id).await?["data"].clone())
    }

    /// レコード一覧 (レコード全体の配列) を取得する
    pub(crate) async fn list_records(
        &self,
        collection: &str,
        query: &ListQuery,
    ) -> anyhow::Result<Vec<serde_json::Value>> {
        let cid = self.collection_id(collection).await?;
        let res = Self::send(
            self.request(reqwest::Method::GET, &format!("/records/{cid}"))
                .query(&query.to_params()),
        )
        .await?;
        Ok(res.as_array().cloned().unwrap_or_default())
    }

    /// レコードを作成し、ID を返す
    pub(crate) async fn create_record(
        &self,
        collection: &str,
        data: serde_json::Value,
    ) -> anyhow::Result<String> {
        let cid = self.collection_id(collection).await?;
        let res = Self::send(
            self.request(reqwest::Method::POST, &format!("/records/{cid}"))
                .json(&serde_json::json!({ "data": data })),
        )
        .await?;
        res["id"]
            .as_str()
            .map(str::to_string)
            .context("id missing in create response")
    }

    /// Node.js のコードを CMS (Dify Sandbox) で実行する
    pub(crate) async fn execute_script(&self, code: &str) -> anyhow::Result<ScriptOutput> {
        let res = Self::send(
            self.request(reqwest::Method::POST, "/scripts")
                .json(&serde_json::json!({ "language": "nodejs", "code": code })),
        )
        .await?;
        Ok(ScriptOutput {
            stdout: res["data"]["stdout"].as_str().unwrap_or_default().to_string(),
            error: res["data"]["error"].as_str().unwrap_or_default().to_string(),
        })
    }

    /// レコードの `data` を置き換える
    pub(crate) async fn update_record_data(
        &self,
        collection: &str,
        record_id: &str,
        data: serde_json::Value,
    ) -> anyhow::Result<()> {
        let cid = self.collection_id(collection).await?;
        Self::send(
            self.request(reqwest::Method::PUT, &format!("/records/{cid}/{record_id}"))
                .json(&serde_json::json!({ "data": data })),
        )
        .await?;
        Ok(())
    }

    /// ファイルをレコードのフィールドに保存し、取得用の URL（CMS からの相対パス）を返す
    pub(crate) async fn upload_file(
        &self,
        collection: &str,
        record_id: &str,
        field: &str,
        data: &[u8],
        content_type: &str,
    ) -> anyhow::Result<String> {
        let cid = self.collection_id(collection).await?;
        let res = Self::send(
            self.request(
                reqwest::Method::POST,
                &format!("/records/{cid}/{record_id}/images/{field}"),
            )
            .json(&serde_json::json!({
                "data": BASE64.encode(data),
                "content_type": content_type,
            })),
        )
        .await?;
        res["url"]
            .as_str()
            .map(str::to_string)
            .context("url missing in upload response")
    }
}

fn parse_collection_ids(collections: &serde_json::Value) -> HashMap<String, String> {
    collections
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|c| {
            let name = c["name"].as_str()?;
            let id = c["id"].as_str()?;
            Some((
                name.to_string(),
                id.strip_prefix("collection:").unwrap_or(id).to_string(),
            ))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn list_query_to_params() {
        let q = ListQuery {
            filter: Some("podcast_id:eq:p1".to_string()),
            sort: Some("created_at".to_string()),
            order_desc: true,
            limit: Some(1),
        };
        assert_eq!(
            q.to_params(),
            vec![
                ("filter", "podcast_id:eq:p1".to_string()),
                ("sort", "created_at".to_string()),
                ("order", "desc".to_string()),
                ("limit", "1".to_string()),
            ]
        );
        assert!(ListQuery::default().to_params().is_empty());
    }

    #[test]
    fn parse_collection_ids_strips_prefix() {
        let json = serde_json::json!([
            { "id": "collection:abc", "name": "episodes" },
            { "id": "def", "name": "podcasts" },
            { "name": "broken" }
        ]);
        let ids = parse_collection_ids(&json);
        assert_eq!(ids.get("episodes").map(String::as_str), Some("abc"));
        assert_eq!(ids.get("podcasts").map(String::as_str), Some("def"));
        assert!(!ids.contains_key("broken"));
    }
}

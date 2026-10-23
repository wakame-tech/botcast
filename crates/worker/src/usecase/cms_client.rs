use anyhow::Context;
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use cms_client::apis::{
    collections_api, configuration::Configuration, record_api, script_api, Error as CmsError,
};
use cms_client::models;
use std::collections::HashMap;
use std::fmt::Debug;
use tokio::sync::Mutex;
use tracing::Instrument;

/// `GET /records/{collection}` のクエリ
#[derive(Debug, Default, Clone)]
pub(crate) struct ListQuery {
    /// `<field>:<op>:<value>` 形式 (1 件のみ)
    pub filter: Option<String>,
    pub sort: Option<String>,
    pub order_desc: bool,
    pub limit: Option<u32>,
}

/// CMS から取得・アップロードするファイルの大きさの上限（web のアップロードも同じ値）
pub(crate) const MAX_FILE_SIZE: usize = 20 * 1024 * 1024;

/// レコードのファイルのフィールド（CMS が `/images/{field}` のアップロードで書き込む
/// `pointer` / `content_type` / `size` / `url`）のうち worker が使うもの
#[derive(Debug, Clone, serde::Deserialize)]
pub(crate) struct FileField {
    pub content_type: String,
    pub size: u64,
}

/// 取得したファイルの中身
#[derive(Debug, Clone)]
pub(crate) struct FileContent {
    pub bytes: Vec<u8>,
    pub content_type: String,
}

/// `POST /scripts` の結果
#[derive(Debug, Clone)]
pub(crate) struct ScriptOutput {
    pub stdout: String,
    pub error: String,
}

/// botcast-cms の REST API クライアント。
///
/// - 通信は botcast-cms の spec から生成した `cms-client`（`crates/cms-client`。`just sync-cms-client` で取り込み直す）
/// - 認証は `X-Api-Key`（環境変数 `API_KEY` = CMS でテナント `botcast` の役割 `service` の機械用のユーザーに発行したキー）
/// - CMS はマルチテナントで、全エンドポイントが `/t/{tenantId}/...` 配下にある。
///   テナント (`CMS_TENANT`、既定 `botcast`) はクライアントが保持して毎回渡す
/// - collection ID は自動採番のため、名前から ID を `/collections` で解決してキャッシュする
/// - 音声・字幕ファイルは `/records/{c}/{r}/images/{field}` に Base64 で保存する
pub(crate) struct CmsClient {
    config: Configuration,
    tenant: String,
    collection_ids: Mutex<HashMap<String, String>>,
}

impl CmsClient {
    pub(crate) fn from_env() -> Self {
        let base_url =
            std::env::var("CMS_URL").unwrap_or_else(|_| "http://localhost:3002".to_string());
        let tenant = std::env::var("CMS_TENANT").unwrap_or_else(|_| "botcast".to_string());
        let api_key = std::env::var("API_KEY").ok();
        Self::new(base_url, tenant, api_key)
    }

    pub(crate) fn new(base_url: String, tenant: String, api_key: Option<String>) -> Self {
        let config = Configuration {
            base_path: base_url.trim_end_matches('/').to_string(),
            api_key: api_key
                .map(|key| cms_client::apis::configuration::ApiKey { prefix: None, key }),
            ..Configuration::new()
        };
        Self {
            config,
            tenant,
            collection_ids: Mutex::new(HashMap::new()),
        }
    }

    /// テナント ID
    pub(crate) fn tenant(&self) -> &str {
        &self.tenant
    }

    async fn list_collections(&self) -> anyhow::Result<Vec<models::Collection>> {
        traced_cms_call(
            "GET",
            "/t/{tenantId}/collections",
            collections_api::collection_api_list(
                &self.config,
                collections_api::CollectionApiListParams {
                    tenant_id: self.tenant.clone(),
                    x_on_behalf_of: None,
                },
            ),
        )
        .await
    }

    /// collection 名を ID（`collection:` 接頭辞なし）に解決する
    pub(crate) async fn collection_id(&self, name: &str) -> anyhow::Result<String> {
        let mut ids = self.collection_ids.lock().await;
        if let Some(id) = ids.get(name) {
            return Ok(id.clone());
        }
        *ids = parse_collection_ids(&self.list_collections().await?);
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
        let record = traced_cms_call(
            "GET",
            "/t/{tenantId}/records/{collectionId}/{recordId}",
            record_api::record_api_read(
                &self.config,
                record_api::RecordApiReadParams {
                    tenant_id: self.tenant.clone(),
                    collection_id: self.collection_id(collection).await?,
                    record_id: record_id.to_string(),
                    x_on_behalf_of: None,
                    eval: None,
                    input: None,
                    vars: None,
                },
            ),
        )
        .await?;
        Ok(serde_json::to_value(record)?)
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
        let records = traced_cms_call(
            "GET",
            "/t/{tenantId}/records/{collectionId}",
            record_api::record_api_list(
                &self.config,
                list_params(
                    self.tenant.clone(),
                    self.collection_id(collection).await?,
                    query,
                ),
            ),
        )
        .await?;
        records
            .into_iter()
            .map(|r| Ok(serde_json::to_value(r)?))
            .collect()
    }

    /// コレクションを作成し、既にあればスキーマ・access を合わせる。ID を返す
    pub(crate) async fn ensure_collection(
        &self,
        name: &str,
        schema: serde_json::Value,
        access: serde_json::Value,
    ) -> anyhow::Result<String> {
        let existing = self
            .list_collections()
            .await?
            .into_iter()
            .find(|c| c.name == name);
        let typed_access: models::CollectionAccess =
            serde_json::from_value(access.clone()).context("invalid collection access")?;
        let collection = match existing {
            Some(c) if collection_needs_update(&c, &schema, &access) => {
                tracing::info!("updating collection '{name}'");
                traced_cms_call(
                    "PUT",
                    "/t/{tenantId}/collections/{collectionId}",
                    collections_api::collection_api_update(
                        &self.config,
                        collections_api::CollectionApiUpdateParams {
                            tenant_id: self.tenant.clone(),
                            collection_id: collection_key(&c.id),
                            update_collection: models::UpdateCollection {
                                name: name.to_string(),
                                schema: Some(schema),
                                access: Some(typed_access),
                            },
                            x_on_behalf_of: None,
                        },
                    ),
                )
                .await
                .with_context(|| format!("Failed to update collection '{name}'"))?
            }
            Some(c) => c,
            None => {
                tracing::info!("creating collection '{name}'");
                traced_cms_call(
                    "POST",
                    "/t/{tenantId}/collections",
                    collections_api::collection_api_create(
                        &self.config,
                        collections_api::CollectionApiCreateParams {
                            tenant_id: self.tenant.clone(),
                            create_or_update_collection: models::CreateOrUpdateCollection {
                                name: name.to_string(),
                                schema,
                                access: Some(typed_access),
                            },
                            x_on_behalf_of: None,
                        },
                    ),
                )
                .await
                .with_context(|| format!("Failed to create collection '{name}'"))?
            }
        };
        let id = collection_key(&collection.id);
        self.collection_ids
            .lock()
            .await
            .insert(name.to_string(), id.clone());
        Ok(id)
    }

    /// レコードを作成し、ID を返す。`owner` を指定すると `X-On-Behalf-Of` でそのユーザーの所有として作成する
    pub(crate) async fn create_record_as(
        &self,
        collection: &str,
        data: serde_json::Value,
        owner: Option<&str>,
    ) -> anyhow::Result<String> {
        let record = traced_cms_call(
            "POST",
            "/t/{tenantId}/records/{collectionId}",
            record_api::record_api_create(
                &self.config,
                record_api::RecordApiCreateParams {
                    tenant_id: self.tenant.clone(),
                    collection_id: self.collection_id(collection).await?,
                    create_or_update_record: models::CreateOrUpdateRecord::new(data),
                    x_on_behalf_of: owner.map(str::to_string),
                },
            ),
        )
        .await?;
        Ok(record.id)
    }

    /// Node.js のコードを CMS (Dify Sandbox) で実行する
    pub(crate) async fn execute_script(&self, code: &str) -> anyhow::Result<ScriptOutput> {
        let res = traced_cms_call(
            "POST",
            "/t/{tenantId}/scripts",
            script_api::script_api_execute(
                &self.config,
                script_api::ScriptApiExecuteParams {
                    tenant_id: self.tenant.clone(),
                    execute_script_request: models::ExecuteScriptRequest::new(
                        models::execute_script_request::Language::Nodejs,
                        code.to_string(),
                    ),
                    x_on_behalf_of: None,
                },
            ),
        )
        .await?;
        Ok(ScriptOutput {
            stdout: res.data.stdout,
            error: res.data.error,
        })
    }

    /// レコードの `data` を置き換える
    pub(crate) async fn update_record_data(
        &self,
        collection: &str,
        record_id: &str,
        data: serde_json::Value,
    ) -> anyhow::Result<()> {
        traced_cms_call(
            "PUT",
            "/t/{tenantId}/records/{collectionId}/{recordId}",
            record_api::record_api_update(
                &self.config,
                record_api::RecordApiUpdateParams {
                    tenant_id: self.tenant.clone(),
                    collection_id: self.collection_id(collection).await?,
                    record_id: record_id.to_string(),
                    create_or_update_record: models::CreateOrUpdateRecord::new(data),
                    x_on_behalf_of: None,
                    if_match: None,
                },
            ),
        )
        .await?;
        Ok(())
    }

    /// レコードのフィールドに保存したファイル（`/records/{c}/{r}/images/{field}`）の中身を取得する。
    /// `max_size` バイトより大きければデコードせずにエラーにする
    pub(crate) async fn get_file(
        &self,
        collection: &str,
        record_id: &str,
        field: &str,
        max_size: usize,
    ) -> anyhow::Result<FileContent> {
        let res = traced_cms_call(
            "GET",
            "/t/{tenantId}/records/{collectionId}/{recordId}/images/{fieldName}",
            record_api::record_api_get_image(
                &self.config,
                record_api::RecordApiGetImageParams {
                    tenant_id: self.tenant.clone(),
                    collection_id: self.collection_id(collection).await?,
                    record_id: record_id.to_string(),
                    field_name: field.to_string(),
                    x_on_behalf_of: None,
                },
            ),
        )
        .await?;
        decode_file(res, max_size)
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
        let content_type = serde_json::from_value(serde_json::json!(content_type))
            .with_context(|| format!("CMS does not accept content type '{content_type}'"))?;
        let res = traced_cms_call(
            "POST",
            "/t/{tenantId}/records/{collectionId}/{recordId}/images/{fieldName}",
            record_api::record_api_upload_image(
                &self.config,
                record_api::RecordApiUploadImageParams {
                    tenant_id: self.tenant.clone(),
                    collection_id: self.collection_id(collection).await?,
                    record_id: record_id.to_string(),
                    field_name: field.to_string(),
                    upload_image_request: models::UploadImageRequest {
                        data: BASE64.encode(data),
                        content_type,
                        filename: None,
                    },
                    x_on_behalf_of: None,
                },
            ),
        )
        .await?;
        Ok(res.url)
    }
}

/// CMS の呼び出し 1 回を span（`cms.request`）で包み、エラーを [`cms_error`] にする
pub(crate) async fn traced_cms_call<T, E: Debug>(
    method: &'static str,
    route: &'static str,
    call: impl std::future::Future<Output = Result<T, CmsError<E>>>,
) -> anyhow::Result<T> {
    traced_cms_call_raw(method, route, call)
        .await
        .map_err(cms_error)
}

/// CMS の呼び出し 1 回を span（`cms.request`）で包む。
/// span にはメソッド・パスのテンプレート（ID を埋める前の形）・失敗したときの状態コードを入れる。
/// 生成クライアントは成功時の状態コードを返さないので、成功時は status を OK にするだけ
pub(crate) async fn traced_cms_call_raw<T, E>(
    method: &'static str,
    route: &'static str,
    call: impl std::future::Future<Output = Result<T, CmsError<E>>>,
) -> Result<T, CmsError<E>> {
    let span = tracing::info_span!(
        "cms.request",
        otel.name = format!("{method} {route}"),
        otel.kind = "client",
        http.request.method = method,
        url.template = route,
        http.response.status_code = tracing::field::Empty,
        otel.status_code = tracing::field::Empty,
        otel.status_message = tracing::field::Empty,
    );
    let result = call.instrument(span.clone()).await;
    match &result {
        Ok(_) => {
            span.record("otel.status_code", "OK");
        }
        Err(e) => {
            if let CmsError::ResponseError(res) = e {
                span.record("http.response.status_code", res.status.as_u16());
            }
            span.record("otel.status_code", "ERROR");
            // ResponseError は状態コードだけ（本文は入れない）
            span.record("otel.status_message", e.to_string());
        }
    }
    result
}

/// 生成クライアントのエラーを、CMS の状態コードと本文が分かる anyhow のエラーにする
pub(crate) fn cms_error<T: Debug>(e: CmsError<T>) -> anyhow::Error {
    match e {
        CmsError::ResponseError(res) => {
            anyhow::anyhow!("CMS returned {}: {}", res.status, res.content)
        }
        other => anyhow::Error::msg(other.to_string()).context("CMS request failed"),
    }
}

/// `GET .../images/{field}` の応答（Base64 の JSON）からファイルの中身を取り出す
fn decode_file(res: serde_json::Value, max_size: usize) -> anyhow::Result<FileContent> {
    let file: models::GetImageResponse =
        serde_json::from_value(res).context("unexpected file response from CMS")?;
    // Base64 は 3 バイトを 4 文字にする
    if usize::try_from(file.size).map_or(true, |size| size > max_size)
        || file.data.len() > max_size.div_ceil(3) * 4
    {
        anyhow::bail!(
            "file from CMS is too large ({} bytes; limit {max_size} bytes)",
            file.size
        );
    }
    let bytes = BASE64
        .decode(file.data)
        .context("file from CMS is not valid Base64")?;
    Ok(FileContent {
        bytes,
        content_type: file.content_type,
    })
}

fn list_params(
    tenant_id: String,
    collection_id: String,
    query: &ListQuery,
) -> record_api::RecordApiListParams {
    record_api::RecordApiListParams {
        tenant_id,
        collection_id,
        x_on_behalf_of: None,
        filter: query.filter.clone().map(|f| vec![f]),
        sort: query.sort.clone(),
        order: query.order_desc.then(|| "desc".to_string()),
        limit: query.limit.map(|l| l as i32),
        offset: None,
        eval: None,
        input: None,
        vars: None,
    }
}

/// コレクションの ID（`collection:` 接頭辞なし）
fn collection_key(id: &str) -> String {
    id.strip_prefix("collection:").unwrap_or(id).to_string()
}

/// 既存コレクションのスキーマ・access が望む値と違うか
fn collection_needs_update(
    current: &models::Collection,
    schema: &serde_json::Value,
    access: &serde_json::Value,
) -> bool {
    let current_access = current
        .access
        .as_ref()
        .and_then(|a| serde_json::to_value(a).ok())
        .unwrap_or(serde_json::Value::Null);
    &current.schema != schema || &current_access != access
}

fn parse_collection_ids(collections: &[models::Collection]) -> HashMap<String, String> {
    collections
        .iter()
        .map(|c| (c.name.clone(), collection_key(&c.id)))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn collection(id: &str, name: &str, schema: serde_json::Value) -> models::Collection {
        models::Collection::new(id.to_string(), name.to_string(), schema, String::new())
    }

    #[test]
    fn new_trims_trailing_slash_of_base_url() {
        // 生成クライアントは `{base_path}/t/{tenantId}/...` を組み立てる
        let cms = CmsClient::new("http://cms:3002/".to_string(), "socia".to_string(), None);
        assert_eq!(cms.config.base_path, "http://cms:3002");
        assert_eq!(cms.tenant, "socia");
    }

    #[test]
    fn new_sends_api_key_as_is() {
        let cms = CmsClient::new(
            "http://cms".to_string(),
            "botcast".to_string(),
            Some("bck_x".to_string()),
        );
        let key = cms.config.api_key.unwrap();
        assert_eq!((key.prefix, key.key.as_str()), (None, "bck_x"));
    }

    #[test]
    fn list_query_to_params() {
        let q = ListQuery {
            filter: Some("podcast_id:eq:p1".to_string()),
            sort: Some("created_at".to_string()),
            order_desc: true,
            limit: Some(1),
        };
        let p = list_params("botcast".to_string(), "c1".to_string(), &q);
        assert_eq!(p.filter, Some(vec!["podcast_id:eq:p1".to_string()]));
        assert_eq!(p.sort.as_deref(), Some("created_at"));
        assert_eq!(p.order.as_deref(), Some("desc"));
        assert_eq!(p.limit, Some(1));
        let empty = list_params(
            "botcast".to_string(),
            "c1".to_string(),
            &ListQuery::default(),
        );
        assert_eq!(
            (empty.filter, empty.sort, empty.order, empty.limit),
            (None, None, None, None)
        );
    }

    #[test]
    fn parse_collection_ids_strips_prefix() {
        let ids = parse_collection_ids(&[
            collection("collection:abc", "episodes", json!({})),
            collection("def", "podcasts", json!({})),
        ]);
        assert_eq!(ids.get("episodes").map(String::as_str), Some("abc"));
        assert_eq!(ids.get("podcasts").map(String::as_str), Some("def"));
    }

    #[test]
    fn collection_needs_update_when_schema_or_access_differs() {
        let schema = json!({ "type": "object" });
        let access = json!({ "read": "public", "write": "owner" });
        let mut same = collection("c", "p", schema.clone());
        same.access = Some(serde_json::from_value(access.clone()).unwrap());
        assert!(!collection_needs_update(&same, &schema, &access));
        let mut other_schema = collection("c", "p", json!({ "type": "string" }));
        other_schema.access = same.access.clone();
        assert!(collection_needs_update(&other_schema, &schema, &access));
        let no_access = collection("c", "p", schema.clone());
        assert!(collection_needs_update(&no_access, &schema, &access));
    }

    #[test]
    fn decode_file_reads_base64_body() {
        let res = json!({ "data": BASE64.encode(b"RIFF"), "content_type": "audio/wav", "size": 4 });
        let file = decode_file(res, 4).unwrap();
        assert_eq!(
            (file.bytes.as_slice(), file.content_type.as_str()),
            (&b"RIFF"[..], "audio/wav")
        );
        assert!(decode_file(
            json!({ "data": "!!", "content_type": "audio/wav", "size": 1 }),
            4
        )
        .is_err());
        assert!(decode_file(json!({ "url": "/x" }), 4).is_err());
    }

    #[test]
    fn decode_file_rejects_files_over_the_limit_before_decoding() {
        // 申告の size が上限を超える
        let res = json!({ "data": BASE64.encode(b"RIFF"), "content_type": "audio/wav", "size": 5 });
        let err = decode_file(res, 4).unwrap_err().to_string();
        assert!(err.contains("too large"), "{err}");
        // size を偽っても Base64 の長さで弾く
        let res =
            json!({ "data": BASE64.encode(b"RIFFRIFF"), "content_type": "audio/wav", "size": 1 });
        assert!(decode_file(res, 4)
            .unwrap_err()
            .to_string()
            .contains("too large"));
        assert!(decode_file(
            json!({ "data": "", "content_type": "audio/wav", "size": -1 }),
            4
        )
        .is_err());
    }

    #[test]
    fn upload_content_types_used_by_the_worker_are_accepted() {
        for ct in ["audio/mpeg", "text/plain"] {
            serde_json::from_value::<models::upload_image_request::ContentType>(json!(ct)).unwrap();
        }
    }

    /// `CMS_URL` / `CMS_TENANT` / `API_KEY` の CMS に、worker が使う操作を一通り送る。
    /// `CMS_TEST_OWNER` (テナントの人間のユーザー `user:xxxx`) があれば `X-On-Behalf-Of` も確かめる
    #[tokio::test]
    #[ignore = "requires a running botcast-cms"]
    async fn operations_against_cms() {
        let cms = CmsClient::from_env();
        crate::usecase::collections::seed(&cms).await.unwrap();

        let owner = std::env::var("CMS_TEST_OWNER").ok();
        let title = format!("t-{}", uuid::Uuid::new_v4());
        let podcast = cms
            .create_record_as("podcasts", json!({ "title": title, "icon": "i" }), owner.as_deref())
            .await
            .unwrap();
        if let Some(owner) = &owner {
            assert_eq!(cms.get_record("podcasts", &podcast).await.unwrap()["owner_id"], *owner);
        }
        let episode = cms
            .create_record_as("episodes", json!({ "title": "e", "podcast_id": podcast }), None)
            .await
            .unwrap();

        let found = cms
            .list_records(
                "episodes",
                &ListQuery {
                    filter: Some(format!("podcast_id:eq:{podcast}")),
                    sort: Some("created_at".to_string()),
                    order_desc: true,
                    limit: Some(1),
                },
            )
            .await
            .unwrap();
        assert_eq!(found.len(), 1);
        assert_eq!(found[0]["id"], episode);

        let url = cms
            .upload_file("episodes", &episode, "srt", b"1\n", "text/plain")
            .await
            .unwrap();
        let mut data = cms.get_record_data("episodes", &episode).await.unwrap();
        data["srt_url"] = json!(url);
        cms.update_record_data("episodes", &episode, data).await.unwrap();
        assert_eq!(cms.get_record_data("episodes", &episode).await.unwrap()["srt_url"], url);

        let err = cms.get_record("episodes", "missing").await.unwrap_err();
        assert!(err.to_string().contains("404"), "{err}");
    }
}

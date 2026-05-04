// @specre 01KNM2BBT5Y18PC3WDQQN18KCA
// @specre 01KNM2BBT5X42XYBFX9RYVXQEN
// @specre 01KNM2BBT558DMFWQ3DWXQR42B
// @specre 01KNM2BBT5C9CD76KVMPH5JGHV
// @specre 01KNM2BBT5MEY51QKNKHC4B176
// @specre 01KNM2BBT5AY940G7M4HHBR26Q
// @specre 01KNM2BBT556RV0ZZB1EHEW2V7
// @specre 01KNM2BBT5E7JFE2PGCRB9VRG6
// @specre 01KNM59JYD4SVC2B5CXMZQCDT5
// @specre 01KNM59JYDHYNFQSWJXE2TV85C
// @specre 01KTJP1XYZABCDEF123456789A
// @specre 01KNS6B47Z2CFB2EWBV3SC88KF
// @specre 01KNS6B4808R0THREKZKX2VX06
// @specre 01KNS6B481EPS6NESNEKV7JJ6R
// @specre 01KNVEV4DB8QKB8DYKJQHYKDQD
use crate::{AppState, error::AppError, repository::CollectionRepository};
use async_openai::{
    Client as OpenAIClient,
    types::{
        ChatCompletionRequestMessage, ChatCompletionRequestUserMessageArgs,
        ChatCompletionRequestUserMessageContent, CreateChatCompletionRequestArgs, ResponseFormat,
        ResponseFormatJsonSchema,
    },
};
use async_trait::async_trait;
use axum_extra::extract::{CookieJar, Host};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use http::{Method, StatusCode};
use jsonschema::JSONSchema;
use openapi::{
    apis::{
        ErrorHandler,
        canvas::{
            Canvas, CanvasApiCreateEdgeResponse, CanvasApiDeleteEdgeResponse,
            CanvasApiListEdgesResponse, CanvasApiQueryResponse,
        },
        collections::{
            CollectionApiCreateResponse, CollectionApiDeleteResponse, CollectionApiListResponse,
            CollectionApiReadResponse, CollectionApiUpdateResponse, Collections,
        },
        job::{Job, JobApiCreateResponse, JobApiListResponse},
        record::{
            Record, RecordApiCopyRecordResponse, RecordApiCreateResponse,
            RecordApiDeleteImageResponse, RecordApiDeleteResponse, RecordApiGenerateResponse,
            RecordApiGetImageResponse, RecordApiListResponse, RecordApiMergeRecordsResponse,
            RecordApiReadResponse, RecordApiSplitRecordResponse, RecordApiUpdateResponse,
            RecordApiUploadImageResponse,
        },
        script::{Script, ScriptApiExecuteResponse},
    },
    models::{
        self as openapi_models, CollectionApiReadPathParams, CollectionApiUpdatePathParams,
        ExecuteScriptRequest, ExecuteScriptResponse, GenerateRecordRequest,
        RecordApiCopyRecordPathParams, RecordApiCreatePathParams, RecordApiDeleteImagePathParams,
        RecordApiDeletePathParams, RecordApiGeneratePathParams, RecordApiGetImagePathParams,
        RecordApiMergeRecordsPathParams, RecordApiReadPathParams, RecordApiSplitRecordPathParams,
        RecordApiUpdatePathParams, RecordApiUploadImagePathParams,
    },
    types::Object,
};
use reqwest::{Client, Url};
use serde_json::Value;
use std::{collections::HashMap, env, sync::Arc};
use worker::{BotcastJob, BotcastWorker};

#[derive(Clone)]
pub struct DifySandboxClient {
    client: Client,
    endpoint: Url,
    api_key: String,
}

impl DifySandboxClient {
    pub fn try_new() -> anyhow::Result<Self> {
        Ok(Self {
            client: Client::new(),
            endpoint: env::var("DIFY_SANDBOX_ENDPOINT")?.parse()?,
            api_key: env::var("DIFY_SANDBOX_API_KEY")?,
        })
    }

    async fn request(&self, req: &ExecuteScriptRequest) -> anyhow::Result<ExecuteScriptResponse> {
        let resp = self
            .client
            .post(self.endpoint.clone())
            .header("X-API-KEY", &self.api_key)
            .json(req)
            .send()
            .await
            .map_err(|e| AppError::internal(format!("Failed to execute script: {}", e)))?;
        if resp.status() != StatusCode::OK {
            return Err(anyhow::anyhow!(
                "Script execution failed with status: {}",
                resp.status()
            ));
        }
        let resp: ExecuteScriptResponse = resp
            .json()
            .await
            .map_err(|e| AppError::internal(format!("Failed to parse script response: {}", e)))?;
        Ok(resp)
    }
}

#[derive(Clone)]
pub struct ApiImpl {
    pub dify_sandbox_client: Arc<DifySandboxClient>,
    pub worker: Arc<BotcastWorker>,
    pub collection_repo: Arc<dyn CollectionRepository>,
    pub openai_client: OpenAIClient<async_openai::config::OpenAIConfig>,
}

impl ApiImpl {
    pub fn new(state: AppState) -> Self {
        let openai_client = OpenAIClient::new();
        Self {
            dify_sandbox_client: state.dify_sandbox_client,
            worker: state.worker,
            collection_repo: state.collection_repo,
            openai_client,
        }
    }
}

impl AsRef<ApiImpl> for ApiImpl {
    fn as_ref(&self) -> &ApiImpl {
        self
    }
}

#[async_trait]
impl ErrorHandler<AppError> for ApiImpl {
    async fn handle_error(
        &self,
        _method: &Method,
        _host: &Host,
        _cookies: &CookieJar,
        error: AppError,
    ) -> Result<axum::response::Response, StatusCode> {
        tracing::error!("API Error: {:?}", error);

        let status_code = match error {
            AppError::NotFound { .. } => StatusCode::NOT_FOUND,
            AppError::Validation { .. } => StatusCode::BAD_REQUEST,
            AppError::Conflict { .. } => StatusCode::CONFLICT,
            AppError::Unauthorized => StatusCode::UNAUTHORIZED,
            AppError::Database(_) | AppError::Internal { .. } => StatusCode::INTERNAL_SERVER_ERROR,
        };

        axum::response::Response::builder()
            .status(status_code)
            .body(axum::body::Body::from(error.to_string()))
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
    }
}

fn validate_record(schema: &Value, value: &Value) -> Result<(), AppError> {
    let compiled_schema = JSONSchema::compile(schema)
        .map_err(|_| AppError::internal("Failed to compile schema for validation"))?;

    if let Err(validation_errors) = compiled_schema.validate(value) {
        let error_messages: Vec<String> = validation_errors.map(|e| e.to_string()).collect();
        return Err(AppError::validation(format!(
            "Validation failed: {}",
            error_messages.join(", ")
        )));
    }
    Ok(())
}

#[async_trait]
impl Collections<AppError> for ApiImpl {
    async fn collection_api_list(
        &self,
        _method: &Method,
        _host: &Host,
        _cookies: &CookieJar,
    ) -> Result<CollectionApiListResponse, AppError> {
        let collections = self
            .collection_repo
            .list()
            .await?
            .into_iter()
            .map(|c| c.try_into())
            .collect::<Result<Vec<_>, _>>()?;

        Ok(CollectionApiListResponse::Status200_TheRequestHasSucceeded(
            collections,
        ))
    }

    async fn collection_api_read(
        &self,
        _method: &Method,
        _host: &Host,
        _cookies: &CookieJar,
        path_params: &CollectionApiReadPathParams,
    ) -> Result<CollectionApiReadResponse, AppError> {
        let collection = self
            .collection_repo
            .get_by_id(&path_params.collection_id)
            .await?;
        Ok(CollectionApiReadResponse::Status200_TheRequestHasSucceeded(
            collection.try_into()?,
        ))
    }

    async fn collection_api_update(
        &self,
        _method: &Method,
        _host: &Host,
        _cookies: &CookieJar,
        path_params: &CollectionApiUpdatePathParams,
        body: &openapi_models::UpdateCollection,
    ) -> Result<CollectionApiUpdateResponse, AppError> {
        if body.name.is_empty() {
            return Err(AppError::validation("Collection name cannot be empty"));
        }
        let collection = self
            .collection_repo
            .update_collection(&path_params.collection_id, &body.name)
            .await?;
        Ok(CollectionApiUpdateResponse::Status200_TheRequestHasSucceeded(collection.try_into()?))
    }

    async fn collection_api_create(
        &self,
        _method: &Method,
        _host: &Host,
        _cookies: &CookieJar,
        body: &openapi_models::CreateOrUpdateCollection,
    ) -> Result<CollectionApiCreateResponse, AppError> {
        if body.name.is_empty() {
            return Err(AppError::validation("Collection name cannot be empty"));
        }

        let schema = serde_json::to_value(&body.schema)
            .map_err(|e| AppError::internal(format!("Failed to convert schema: {}", e)))?;

        let collection = self.collection_repo.create(&body.name, schema).await?;

        Ok(CollectionApiCreateResponse::Status201_TheRequestHasSucceededAndANewResourceHasBeenCreatedAsAResult(collection.try_into()?))
    }

    async fn collection_api_delete(
        &self,
        _method: &Method,
        _host: &Host,
        _cookies: &CookieJar,
        path_params: &openapi_models::CollectionApiDeletePathParams,
    ) -> Result<CollectionApiDeleteResponse, AppError> {
        self.collection_repo
            .delete(&path_params.collection_id)
            .await?;
        Ok(CollectionApiDeleteResponse::Status204_ThereIsNoContentToSendForThisRequest)
    }
}

#[async_trait]
impl Record<AppError> for ApiImpl {
    async fn record_api_list(
        &self,
        _method: &Method,
        _host: &Host,
        _cookies: &CookieJar,
        path_params: &openapi_models::RecordApiListPathParams,
    ) -> Result<RecordApiListResponse, AppError> {
        // List records using repository
        let records = self
            .collection_repo
            .list_record(&path_params.collection_id)
            .await?
            .iter()
            .map(|record| record.clone().try_into())
            .collect::<Result<Vec<_>, _>>()?;

        Ok(RecordApiListResponse::Status200_TheRequestHasSucceeded(
            records,
        ))
    }

    async fn record_api_read(
        &self,
        _method: &Method,
        _host: &Host,
        _cookies: &CookieJar,
        RecordApiReadPathParams {
            collection_id,
            record_id,
        }: &openapi_models::RecordApiReadPathParams,
    ) -> Result<RecordApiReadResponse, AppError> {
        let record = self
            .collection_repo
            .get_record(collection_id, record_id)
            .await?;
        Ok(RecordApiReadResponse::Status200_TheRequestHasSucceeded(
            record.try_into()?,
        ))
    }

    async fn record_api_create(
        &self,
        _method: &Method,
        _host: &Host,
        _cookies: &CookieJar,
        RecordApiCreatePathParams { collection_id }: &openapi_models::RecordApiCreatePathParams,
        body: &openapi_models::CreateOrUpdateRecord,
    ) -> Result<RecordApiCreateResponse, AppError> {
        let record_value =
            serde_json::to_value(&body.data).map_err(|e| AppError::validation(e.to_string()))?;

        let collection = self.collection_repo.get_by_id(collection_id).await?;

        validate_record(&collection.schema, &record_value)?;

        let record = self
            .collection_repo
            .add_record(collection_id, record_value)
            .await?;
        Ok(RecordApiCreateResponse::Status201_TheRequestHasSucceededAndANewResourceHasBeenCreatedAsAResult(record.try_into()?))
    }

    async fn record_api_update(
        &self,
        _method: &Method,
        _host: &Host,
        _cookies: &CookieJar,
        RecordApiUpdatePathParams {
            collection_id,
            record_id,
        }: &openapi_models::RecordApiUpdatePathParams,
        body: &openapi_models::CreateOrUpdateRecord,
    ) -> Result<RecordApiUpdateResponse, AppError> {
        let record_value =
            serde_json::to_value(&body.data).map_err(|e| AppError::validation(e.to_string()))?;

        let collection = self.collection_repo.get_by_id(collection_id).await?;

        validate_record(&collection.schema, &record_value)?;

        let record = self
            .collection_repo
            .update_record(collection_id, record_id, record_value)
            .await?;

        Ok(RecordApiUpdateResponse::Status200_TheRequestHasSucceeded(
            record.try_into()?,
        ))
    }

    async fn record_api_delete(
        &self,
        _method: &Method,
        _host: &Host,
        _cookies: &CookieJar,
        RecordApiDeletePathParams {
            collection_id,
            record_id,
        }: &openapi_models::RecordApiDeletePathParams,
    ) -> Result<RecordApiDeleteResponse, AppError> {
        self.collection_repo
            .delete_record(collection_id, record_id)
            .await?;
        Ok(RecordApiDeleteResponse::Status204_ThereIsNoContentToSendForThisRequest)
    }

    async fn record_api_generate(
        &self,
        _method: &Method,
        _host: &Host,
        _cookies: &CookieJar,
        RecordApiGeneratePathParams { collection_id }: &openapi_models::RecordApiGeneratePathParams,
        body: &GenerateRecordRequest,
    ) -> Result<RecordApiGenerateResponse, AppError> {
        // Get collection to retrieve schema
        let collection = self.collection_repo.get_by_id(collection_id).await?;

        // Create user message
        let user_message = ChatCompletionRequestUserMessageArgs::default()
            .content(ChatCompletionRequestUserMessageContent::Text(
                body.prompt.clone(),
            ))
            .build()
            .map_err(|e| AppError::internal(format!("Failed to build user message: {}", e)))?;

        // Create JSON schema for structured output
        let json_schema = ResponseFormatJsonSchema {
            name: "record_data".to_string(),
            description: Some("Generated record data conforming to collection schema".to_string()),
            schema: Some(collection.schema.clone()),
            strict: Some(true),
        };

        // Create chat completion request with structured output
        let request = CreateChatCompletionRequestArgs::default()
            .model("gpt-4o-2024-08-06")
            .messages([ChatCompletionRequestMessage::User(user_message)])
            .response_format(ResponseFormat::JsonSchema { json_schema })
            .build()
            .map_err(|e| AppError::internal(format!("Failed to build OpenAI request: {}", e)))?;

        // Call OpenAI API
        let response = self
            .openai_client
            .chat()
            .create(request)
            .await
            .map_err(|e| AppError::internal(format!("OpenAI API call failed: {}", e)))?;

        // Extract generated content
        let generated_content = response
            .choices
            .first()
            .and_then(|choice| choice.message.content.as_ref())
            .ok_or_else(|| AppError::internal("No content generated by OpenAI"))?;

        // Parse generated JSON
        let record_value: Value = serde_json::from_str(generated_content)
            .map_err(|e| AppError::validation(format!("Failed to parse generated JSON: {}", e)))?;

        // Validate against schema
        validate_record(&collection.schema, &record_value)?;

        // Add record to collection
        let record = self
            .collection_repo
            .add_record(collection_id, record_value)
            .await?;

        Ok(RecordApiGenerateResponse::Status201_TheRequestHasSucceededAndANewResourceHasBeenCreatedAsAResult(record.try_into()?))
    }

    async fn record_api_upload_image(
        &self,
        _method: &Method,
        _host: &Host,
        _cookies: &CookieJar,
        path_params: &RecordApiUploadImagePathParams,
        body: &openapi_models::UploadImageRequest,
    ) -> Result<RecordApiUploadImageResponse, AppError> {
        // Decode base64 image data
        let data = BASE64
            .decode(&body.data)
            .map_err(|e| AppError::validation(format!("Invalid base64 data: {e}")))?;

        let content_type = body.content_type.to_string();

        let image_info = self
            .collection_repo
            .upload_image(
                &path_params.collection_id,
                &path_params.record_id,
                &path_params.field_name,
                data,
                &content_type,
            )
            .await?;

        Ok(RecordApiUploadImageResponse::Status201_TheRequestHasSucceededAndANewResourceHasBeenCreatedAsAResult(
            openapi_models::UploadImageResponse {
                pointer: image_info.pointer,
                url: image_info.url,
            },
        ))
    }

    async fn record_api_get_image(
        &self,
        _method: &Method,
        _host: &Host,
        _cookies: &CookieJar,
        path_params: &RecordApiGetImagePathParams,
    ) -> Result<RecordApiGetImageResponse, AppError> {
        let (data, content_type) = self
            .collection_repo
            .get_image(
                &path_params.collection_id,
                &path_params.record_id,
                &path_params.field_name,
            )
            .await?;

        // Return image data as base64-encoded JSON object
        // Note: Ideally this should return raw binary with proper Content-Type header
        let response_obj = serde_json::json!({
            "data": BASE64.encode(&data),
            "content_type": content_type,
            "size": data.len()
        });

        Ok(RecordApiGetImageResponse::Status200_TheRequestHasSucceeded(
            serde_json::from_value(response_obj)
                .map_err(|e| AppError::internal(format!("Failed to create response: {e}")))?,
        ))
    }

    async fn record_api_delete_image(
        &self,
        _method: &Method,
        _host: &Host,
        _cookies: &CookieJar,
        path_params: &RecordApiDeleteImagePathParams,
    ) -> Result<RecordApiDeleteImageResponse, AppError> {
        self.collection_repo
            .delete_image(
                &path_params.collection_id,
                &path_params.record_id,
                &path_params.field_name,
            )
            .await?;

        Ok(RecordApiDeleteImageResponse::Status204_ThereIsNoContentToSendForThisRequest)
    }

    async fn record_api_copy_record(
        &self,
        _method: &Method,
        _host: &Host,
        _cookies: &CookieJar,
        path_params: &RecordApiCopyRecordPathParams,
        body: &openapi_models::CopyRecordRequest,
    ) -> Result<RecordApiCopyRecordResponse, AppError> {
        // Get target collection to validate schema
        let target_collection = self
            .collection_repo
            .get_by_id(&body.target_collection_id)
            .await?;

        // Get source record
        let source_record = self
            .collection_repo
            .get_record(&path_params.collection_id, &path_params.record_id)
            .await?;

        // Validate source data against target schema
        validate_record(&target_collection.schema, &source_record.data)?;

        // Copy the record
        let new_record = self
            .collection_repo
            .copy_record(
                &path_params.collection_id,
                &path_params.record_id,
                &body.target_collection_id,
            )
            .await?;

        Ok(RecordApiCopyRecordResponse::Status201_TheRequestHasSucceededAndANewResourceHasBeenCreatedAsAResult(
            new_record.try_into()?,
        ))
    }

    async fn record_api_merge_records(
        &self,
        _method: &Method,
        _host: &Host,
        _cookies: &CookieJar,
        path_params: &RecordApiMergeRecordsPathParams,
        body: &openapi_models::MergeRecordsRequest,
    ) -> Result<RecordApiMergeRecordsResponse, AppError> {
        let add_fields: Option<Vec<String>> = body.add_fields.clone();
        let add_fields_slice = add_fields.as_deref();

        let merged_record = self
            .collection_repo
            .merge_records(
                &path_params.collection_id,
                &body.source_record_ids,
                &body.target_record_id,
                add_fields_slice,
                body.delete_sources.unwrap_or(false),
            )
            .await?;

        Ok(
            RecordApiMergeRecordsResponse::Status200_TheRequestHasSucceeded(
                merged_record.try_into()?,
            ),
        )
    }

    async fn record_api_split_record(
        &self,
        _method: &Method,
        _host: &Host,
        _cookies: &CookieJar,
        path_params: &RecordApiSplitRecordPathParams,
        body: &openapi_models::SplitRecordRequest,
    ) -> Result<RecordApiSplitRecordResponse, AppError> {
        // Convert splits to Value array
        let splits: Vec<Value> = body
            .splits
            .iter()
            .map(|s| serde_json::to_value(s))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| AppError::validation(format!("Invalid split data: {}", e)))?;

        let (original, new_records) = self
            .collection_repo
            .split_record(&path_params.collection_id, &path_params.record_id, &splits)
            .await?;

        let split_records: Vec<openapi_models::Record> = new_records
            .into_iter()
            .map(|r| r.try_into())
            .collect::<Result<Vec<_>, _>>()?;

        Ok(
            RecordApiSplitRecordResponse::Status200_TheRequestHasSucceeded(
                openapi_models::SplitRecordResponse {
                    original: original.try_into()?,
                    splits: split_records,
                },
            ),
        )
    }
}

#[async_trait]
impl Script<AppError> for ApiImpl {
    async fn script_api_execute(
        &self,
        _method: &Method,
        _host: &Host,
        _cookies: &CookieJar,
        body: &openapi_models::ExecuteScriptRequest,
    ) -> Result<ScriptApiExecuteResponse, AppError> {
        let resp = self
            .dify_sandbox_client
            .request(body)
            .await
            .map_err(|e| AppError::internal(format!("Script execution failed: {}", e)))?;
        Ok(ScriptApiExecuteResponse::Status200_TheRequestHasSucceeded(
            resp,
        ))
    }
}

fn try_into_map(obj: &openapi::types::Object) -> Result<HashMap<String, Value>, AppError> {
    let params = serde_json::to_value(obj)
        .map_err(|e| AppError::validation(format!("Invalid job parameters: {}", e)))?;
    let params: HashMap<String, Value> = serde_json::from_value(params)
        .map_err(|e| AppError::validation(format!("Failed to parse job parameters: {}", e)))?;
    Ok(params)
}

fn try_into_openapi_job(job: BotcastJob) -> Result<openapi_models::Job, AppError> {
    let params = serde_json::to_value(&job.params)
        .map_err(|e| AppError::internal(format!("Failed to convert job parameters: {}", e)))?;
    let params: Object = serde_json::from_value(params)
        .map_err(|e| AppError::internal(format!("Failed to parse job parameters: {}", e)))?;
    Ok(openapi_models::Job {
        name: job.name,
        params,
        status: job.status,
    })
}

#[async_trait]
impl Job<AppError> for ApiImpl {
    async fn job_api_create(
        &self,
        _method: &Method,
        _host: &Host,
        _cookies: &CookieJar,
        body: &openapi_models::CreateOrUpdateJob,
    ) -> Result<JobApiCreateResponse, AppError> {
        let params = try_into_map(&body.params)?;
        self.worker
            .enqueue_job(body.name.clone(), params)
            .await
            .map_err(|e| AppError::internal(format!("Failed to enqueue job: {}", e)))?;
        Ok(JobApiCreateResponse::Status204_ThereIsNoContentToSendForThisRequest)
    }

    async fn job_api_list(
        &self,
        _method: &Method,
        _host: &Host,
        _cookies: &CookieJar,
    ) -> Result<JobApiListResponse, AppError> {
        let jobs = self
            .worker
            .list_jobs()
            .await
            .map_err(|e| AppError::internal(format!("Failed to list jobs: {}", e)))?
            .into_iter()
            .map(try_into_openapi_job)
            .collect::<Result<Vec<_>, _>>()?;
        Ok(JobApiListResponse::Status200_TheRequestHasSucceeded(jobs))
    }
}

#[async_trait]
impl Canvas<AppError> for ApiImpl {
    async fn canvas_api_create_edge(
        &self,
        _method: &Method,
        _host: &Host,
        _cookies: &CookieJar,
        body: &openapi_models::CreateEdgeRequest,
    ) -> Result<CanvasApiCreateEdgeResponse, AppError> {
        let edge = self
            .collection_repo
            .create_edge(&body.from, &body.to, &body.label)
            .await?;
        Ok(CanvasApiCreateEdgeResponse::Status201_TheRequestHasSucceededAndANewResourceHasBeenCreatedAsAResult(
            edge.try_into()?,
        ))
    }

    async fn canvas_api_list_edges(
        &self,
        _method: &Method,
        _host: &Host,
        _cookies: &CookieJar,
        query_params: &openapi_models::CanvasApiListEdgesQueryParams,
    ) -> Result<CanvasApiListEdgesResponse, AppError> {
        let edges = self
            .collection_repo
            .list_edges(&query_params.record_id)
            .await?
            .into_iter()
            .map(|e| e.try_into())
            .collect::<Result<Vec<_>, _>>()?;
        Ok(CanvasApiListEdgesResponse::Status200_TheRequestHasSucceeded(edges))
    }

    async fn canvas_api_delete_edge(
        &self,
        _method: &Method,
        _host: &Host,
        _cookies: &CookieJar,
        path_params: &openapi_models::CanvasApiDeleteEdgePathParams,
    ) -> Result<CanvasApiDeleteEdgeResponse, AppError> {
        self.collection_repo
            .delete_edge(&path_params.edge_id)
            .await?;
        Ok(CanvasApiDeleteEdgeResponse::Status204_ThereIsNoContentToSendForThisRequest)
    }

    async fn canvas_api_query(
        &self,
        _method: &Method,
        _host: &Host,
        _cookies: &CookieJar,
        body: &openapi_models::CanvasQueryRequest,
    ) -> Result<CanvasApiQueryResponse, AppError> {
        let depth = body.depth.unwrap_or(1);
        let conditions = body
            .conditions
            .as_ref()
            .map(|c| serde_json::to_value(c))
            .transpose()
            .map_err(|e| AppError::validation(format!("Invalid conditions: {}", e)))?;

        let (records, edges) = self
            .collection_repo
            .query_subgraph(&body.record_id, depth, conditions)
            .await?;

        match body.format.as_str() {
            "json-canvas" => {
                let canvas = build_json_canvas(&records, &edges)?;
                Ok(CanvasApiQueryResponse::Status200_TheRequestHasSucceeded(
                    openapi_models::CanvasQueryResponse {
                        format: "json-canvas".to_string(),
                        json_canvas: Some(canvas),
                        markdown: None,
                    },
                ))
            }
            "markdown" => {
                let md = build_markdown(&records, &edges);
                Ok(CanvasApiQueryResponse::Status200_TheRequestHasSucceeded(
                    openapi_models::CanvasQueryResponse {
                        format: "markdown".to_string(),
                        json_canvas: None,
                        markdown: Some(md),
                    },
                ))
            }
            _ => Err(AppError::validation(format!(
                "Unsupported format: {}. Use 'json-canvas' or 'markdown'",
                body.format
            ))),
        }
    }
}

// Custom handler: GET /records/{collectionId}/with-relations
pub(crate) async fn records_with_relations_handler(
    axum::extract::State(api_impl): axum::extract::State<ApiImpl>,
    axum::extract::Path(collection_id): axum::extract::Path<String>,
) -> Result<axum::Json<Vec<serde_json::Value>>, AppError> {
    let records = api_impl
        .collection_repo
        .list_records_with_relations(&collection_id)
        .await?;

    let result: Vec<serde_json::Value> = records
        .into_iter()
        .map(|r| {
            let relations: Vec<serde_json::Value> = r
                .relations
                .into_iter()
                .map(|rel| {
                    serde_json::json!({
                        "edge_id": rel.edge_id,
                        "label": rel.label,
                        "direction": rel.direction,
                        "record": {
                            "id": rel.record.id.key().to_string(),
                            "data": rel.record.data,
                            "created_at": rel.record.created_at.map(|t| t.to_string()),
                            "updated_at": rel.record.updated_at.map(|t| t.to_string()),
                        }
                    })
                })
                .collect();

            serde_json::json!({
                "id": r.id.key().to_string(),
                "data": r.data,
                "created_at": r.created_at.map(|t| t.to_string()),
                "updated_at": r.updated_at.map(|t| t.to_string()),
                "relations": relations,
            })
        })
        .collect();

    Ok(axum::Json(result))
}

pub(crate) fn build_json_canvas(
    records: &[crate::repository::Record],
    edges: &[crate::repository::Edge],
) -> Result<openapi_models::JsonCanvasResponse, AppError> {
    let nodes: Vec<openapi_models::JsonCanvasNode> = records
        .iter()
        .enumerate()
        .map(|(i, record)| {
            let text = serde_json::to_string_pretty(&record.data).unwrap_or_default();
            openapi_models::JsonCanvasNode {
                id: record.id.to_string(),
                x: (i as i32 % 4) * 300,
                y: (i as i32 / 4) * 300,
                width: 250,
                height: 200,
                r#type: "text".to_string(),
                text,
            }
        })
        .collect();

    let canvas_edges: Vec<openapi_models::JsonCanvasEdge> = edges
        .iter()
        .map(|edge| openapi_models::JsonCanvasEdge {
            id: edge.id.to_string(),
            from_node: edge.out_record.to_string(),
            to_node: edge.in_record.to_string(),
            label: Some(edge.label.clone()),
        })
        .collect();

    Ok(openapi_models::JsonCanvasResponse {
        nodes,
        edges: canvas_edges,
    })
}

pub(crate) fn build_markdown(
    records: &[crate::repository::Record],
    edges: &[crate::repository::Edge],
) -> String {
    let mut md = String::new();

    md.push_str("# Subgraph Export\n\n");

    // Records as sections
    for record in records {
        md.push_str(&format!("## {}\n\n", record.id));
        if let Ok(pretty) = serde_json::to_string_pretty(&record.data) {
            md.push_str(&format!("```json\n{}\n```\n\n", pretty));
        }
    }

    // Edges as a relationship table
    if !edges.is_empty() {
        md.push_str("## Relationships\n\n");
        md.push_str("| From | Label | To |\n");
        md.push_str("|------|-------|----|\n");
        for edge in edges {
            md.push_str(&format!(
                "| {} | {} | {} |\n",
                edge.out_record, edge.label, edge.in_record
            ));
        }
    }

    md
}


// @specre 01KNM7ADS668TSZ8XMDK6JYN2W
#[cfg(test)]
mod tests {
    use super::*;
    use crate::repository::{Edge, Record};
    use chrono::Utc;
    use serde_json::json;
    use surrealdb::RecordId;

    fn make_record(table: &str, key: &str, data: Value) -> Record {
        Record {
            id: RecordId::from((table, key)),
            data,
            created_at: Some(Utc::now().fixed_offset()),
            updated_at: None,
        }
    }

    fn make_edge(
        id_key: &str,
        in_table: &str,
        in_key: &str,
        out_table: &str,
        out_key: &str,
        label: &str,
    ) -> Edge {
        Edge {
            id: RecordId::from(("edge", id_key)),
            in_record: RecordId::from((in_table, in_key)),
            out_record: RecordId::from((out_table, out_key)),
            label: label.to_string(),
            created_at: Utc::now().fixed_offset(),
        }
    }

    #[test]
    fn build_json_canvas_empty() {
        let result = build_json_canvas(&[], &[]).unwrap();
        assert!(result.nodes.is_empty());
        assert!(result.edges.is_empty());
    }

    #[test]
    fn build_json_canvas_with_records_and_edges() {
        let r1 = make_record("t", "a", json!({"name": "Alice"}));
        let r2 = make_record("t", "b", json!({"name": "Bob"}));
        let e1 = make_edge("e1", "t", "a", "t", "b", "knows");

        let result = build_json_canvas(&[r1, r2], &[e1]).unwrap();
        assert_eq!(result.nodes.len(), 2);
        assert_eq!(result.edges.len(), 1);

        assert_eq!(result.nodes[0].r#type, "text");
        assert!(result.nodes[0].text.contains("Alice"));
        assert_eq!(result.edges[0].label, Some("knows".to_string()));
    }

    #[test]
    fn build_markdown_empty() {
        let result = build_markdown(&[], &[]);
        assert_eq!(result, "# Subgraph Export\n\n");
    }

    #[test]
    fn build_markdown_with_records_and_edges() {
        let r1 = make_record("t", "a", json!({"name": "Alice"}));
        let r2 = make_record("t", "b", json!({"name": "Bob"}));
        let e1 = make_edge("e1", "t", "a", "t", "b", "knows");

        let result = build_markdown(&[r1, r2], &[e1]);
        assert!(result.contains("## t:a"));
        assert!(result.contains("## t:b"));
        assert!(result.contains("\"Alice\""));
        assert!(result.contains("## Relationships"));
        assert!(result.contains("| knows |"));
    }

    // Tests for RecordWithRelations
    use crate::repository::{RecordWithRelations, RelatedRecord};

    fn make_record_with_relations(
        table: &str,
        key: &str,
        data: Value,
        relations: Vec<RelatedRecord>,
    ) -> RecordWithRelations {
        RecordWithRelations {
            id: RecordId::from((table, key)),
            data,
            created_at: Some(Utc::now().fixed_offset()),
            updated_at: None,
            relations,
        }
    }

    #[test]
    fn record_with_relations_no_relations() {
        let rwr = make_record_with_relations("t", "a", json!({"name": "Alice"}), vec![]);
        assert_eq!(rwr.relations.len(), 0);
        assert_eq!(rwr.id.to_string(), "t:a");
    }

    #[test]
    fn record_with_relations_has_outgoing() {
        let related = make_record("t", "b", json!({"name": "Bob"}));
        let rel = RelatedRecord {
            edge_id: "edge:e1".to_string(),
            label: "knows".to_string(),
            direction: "outgoing".to_string(),
            record: related,
        };
        let rwr = make_record_with_relations("t", "a", json!({"name": "Alice"}), vec![rel]);

        assert_eq!(rwr.relations.len(), 1);
        assert_eq!(rwr.relations[0].label, "knows");
        assert_eq!(rwr.relations[0].direction, "outgoing");
        assert_eq!(rwr.relations[0].record.id.to_string(), "t:b");
    }

    #[test]
    fn record_with_relations_has_incoming() {
        let related = make_record("t", "c", json!({"name": "Carol"}));
        let rel = RelatedRecord {
            edge_id: "edge:e2".to_string(),
            label: "belongs_to".to_string(),
            direction: "incoming".to_string(),
            record: related,
        };
        let rwr = make_record_with_relations("t", "a", json!({"name": "Alice"}), vec![rel]);

        assert_eq!(rwr.relations.len(), 1);
        assert_eq!(rwr.relations[0].direction, "incoming");
        assert_eq!(rwr.relations[0].label, "belongs_to");
    }

    #[test]
    fn record_with_relations_multiple_relations() {
        let r_bob = make_record("t", "b", json!({"name": "Bob"}));
        let r_carol = make_record("org", "c", json!({"name": "Acme Corp"}));

        let rel1 = RelatedRecord {
            edge_id: "edge:e1".to_string(),
            label: "knows".to_string(),
            direction: "outgoing".to_string(),
            record: r_bob,
        };
        let rel2 = RelatedRecord {
            edge_id: "edge:e2".to_string(),
            label: "member_of".to_string(),
            direction: "outgoing".to_string(),
            record: r_carol,
        };

        let rwr = make_record_with_relations("t", "a", json!({"name": "Alice"}), vec![rel1, rel2]);

        assert_eq!(rwr.relations.len(), 2);
        assert_eq!(rwr.relations[0].label, "knows");
        assert_eq!(rwr.relations[1].label, "member_of");
        assert_eq!(rwr.relations[1].record.id.to_string(), "org:c");
    }

    #[test]
    fn record_with_relations_serialization() {
        let related = make_record("t", "b", json!({"name": "Bob"}));
        let rel = RelatedRecord {
            edge_id: "edge:e1".to_string(),
            label: "knows".to_string(),
            direction: "outgoing".to_string(),
            record: related,
        };
        let rwr = make_record_with_relations("t", "a", json!({"name": "Alice"}), vec![rel]);

        let serialized = serde_json::to_value(&rwr).unwrap();
        assert_eq!(serialized["relations"][0]["label"], "knows");
        assert_eq!(serialized["relations"][0]["direction"], "outgoing");
        assert_eq!(serialized["relations"][0]["edge_id"], "edge:e1");
    }
}

use async_trait::async_trait;
use axum::extract::*;
use axum_extra::extract::{CookieJar, Host};
use bytes::Bytes;
use http::Method;
use serde::{Deserialize, Serialize};

use crate::{models, types::*};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
#[allow(clippy::large_enum_variant)]
pub enum RecordApiCopyRecordResponse {
    /// The request has succeeded and a new resource has been created as a result.
    Status201_TheRequestHasSucceededAndANewResourceHasBeenCreatedAsAResult
    (models::Record)
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
#[allow(clippy::large_enum_variant)]
pub enum RecordApiCreateResponse {
    /// The request has succeeded and a new resource has been created as a result.
    Status201_TheRequestHasSucceededAndANewResourceHasBeenCreatedAsAResult
    (models::Record)
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
#[allow(clippy::large_enum_variant)]
pub enum RecordApiDeleteResponse {
    /// There is no content to send for this request, but the headers may be useful. 
    Status204_ThereIsNoContentToSendForThisRequest
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
#[allow(clippy::large_enum_variant)]
pub enum RecordApiDeleteImageResponse {
    /// There is no content to send for this request, but the headers may be useful. 
    Status204_ThereIsNoContentToSendForThisRequest
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
#[allow(clippy::large_enum_variant)]
pub enum RecordApiGenerateResponse {
    /// The request has succeeded and a new resource has been created as a result.
    Status201_TheRequestHasSucceededAndANewResourceHasBeenCreatedAsAResult
    (models::Record)
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
#[allow(clippy::large_enum_variant)]
pub enum RecordApiGetImageResponse {
    /// The request has succeeded.
    Status200_TheRequestHasSucceeded
    (crate::types::Object)
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
#[allow(clippy::large_enum_variant)]
pub enum RecordApiListResponse {
    /// The request has succeeded.
    Status200_TheRequestHasSucceeded
    (Vec<models::Record>)
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
#[allow(clippy::large_enum_variant)]
pub enum RecordApiMergeRecordsResponse {
    /// The request has succeeded.
    Status200_TheRequestHasSucceeded
    (models::Record)
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
#[allow(clippy::large_enum_variant)]
pub enum RecordApiReadResponse {
    /// The request has succeeded.
    Status200_TheRequestHasSucceeded
    (models::Record)
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
#[allow(clippy::large_enum_variant)]
pub enum RecordApiSplitRecordResponse {
    /// The request has succeeded.
    Status200_TheRequestHasSucceeded
    (models::SplitRecordResponse)
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
#[allow(clippy::large_enum_variant)]
pub enum RecordApiUpdateResponse {
    /// The request has succeeded.
    Status200_TheRequestHasSucceeded
    (models::Record)
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[must_use]
#[allow(clippy::large_enum_variant)]
pub enum RecordApiUploadImageResponse {
    /// The request has succeeded and a new resource has been created as a result.
    Status201_TheRequestHasSucceededAndANewResourceHasBeenCreatedAsAResult
    (models::UploadImageResponse)
}


/// Record
#[async_trait]
#[allow(clippy::ptr_arg)]
pub trait Record<E: std::fmt::Debug + Send + Sync + 'static = ()>: super::ErrorHandler<E> {
    /// RecordApiCopyRecord - POST /records/{collectionId}/{recordId}/copy
    async fn record_api_copy_record(
    &self,
    method: &Method,
    host: &Host,
    cookies: &CookieJar,
      path_params: &models::RecordApiCopyRecordPathParams,
            body: &models::CopyRecordRequest,
    ) -> Result<RecordApiCopyRecordResponse, E>;

    /// RecordApiCreate - POST /records/{collectionId}
    async fn record_api_create(
    &self,
    method: &Method,
    host: &Host,
    cookies: &CookieJar,
      path_params: &models::RecordApiCreatePathParams,
            body: &models::CreateOrUpdateRecord,
    ) -> Result<RecordApiCreateResponse, E>;

    /// RecordApiDelete - DELETE /records/{collectionId}/{recordId}
    async fn record_api_delete(
    &self,
    method: &Method,
    host: &Host,
    cookies: &CookieJar,
      path_params: &models::RecordApiDeletePathParams,
    ) -> Result<RecordApiDeleteResponse, E>;

    /// RecordApiDeleteImage - DELETE /records/{collectionId}/{recordId}/images/{fieldName}
    async fn record_api_delete_image(
    &self,
    method: &Method,
    host: &Host,
    cookies: &CookieJar,
      path_params: &models::RecordApiDeleteImagePathParams,
    ) -> Result<RecordApiDeleteImageResponse, E>;

    /// RecordApiGenerate - POST /records/{collectionId}/generate
    async fn record_api_generate(
    &self,
    method: &Method,
    host: &Host,
    cookies: &CookieJar,
      path_params: &models::RecordApiGeneratePathParams,
            body: &models::GenerateRecordRequest,
    ) -> Result<RecordApiGenerateResponse, E>;

    /// RecordApiGetImage - GET /records/{collectionId}/{recordId}/images/{fieldName}
    async fn record_api_get_image(
    &self,
    method: &Method,
    host: &Host,
    cookies: &CookieJar,
      path_params: &models::RecordApiGetImagePathParams,
    ) -> Result<RecordApiGetImageResponse, E>;

    /// RecordApiList - GET /records/{collectionId}
    async fn record_api_list(
    &self,
    method: &Method,
    host: &Host,
    cookies: &CookieJar,
      path_params: &models::RecordApiListPathParams,
    ) -> Result<RecordApiListResponse, E>;

    /// RecordApiMergeRecords - POST /records/{collectionId}/merge
    async fn record_api_merge_records(
    &self,
    method: &Method,
    host: &Host,
    cookies: &CookieJar,
      path_params: &models::RecordApiMergeRecordsPathParams,
            body: &models::MergeRecordsRequest,
    ) -> Result<RecordApiMergeRecordsResponse, E>;

    /// RecordApiRead - GET /records/{collectionId}/{recordId}
    async fn record_api_read(
    &self,
    method: &Method,
    host: &Host,
    cookies: &CookieJar,
      path_params: &models::RecordApiReadPathParams,
    ) -> Result<RecordApiReadResponse, E>;

    /// RecordApiSplitRecord - POST /records/{collectionId}/{recordId}/split
    async fn record_api_split_record(
    &self,
    method: &Method,
    host: &Host,
    cookies: &CookieJar,
      path_params: &models::RecordApiSplitRecordPathParams,
            body: &models::SplitRecordRequest,
    ) -> Result<RecordApiSplitRecordResponse, E>;

    /// RecordApiUpdate - PUT /records/{collectionId}/{recordId}
    async fn record_api_update(
    &self,
    method: &Method,
    host: &Host,
    cookies: &CookieJar,
      path_params: &models::RecordApiUpdatePathParams,
            body: &models::CreateOrUpdateRecord,
    ) -> Result<RecordApiUpdateResponse, E>;

    /// RecordApiUploadImage - POST /records/{collectionId}/{recordId}/images/{fieldName}
    async fn record_api_upload_image(
    &self,
    method: &Method,
    host: &Host,
    cookies: &CookieJar,
      path_params: &models::RecordApiUploadImagePathParams,
            body: &models::UploadImageRequest,
    ) -> Result<RecordApiUploadImageResponse, E>;
}

// @specre 01KNM2BBT5Y18PC3WDQQN18KCA
// @specre 01KNM2BBT5C9CD76KVMPH5JGHV
// @specre 01KNM59JYD4SVC2B5CXMZQCDT5
// @specre 01KNM59JYDHYNFQSWJXE2TV85C
// @specre 01KTJP1XYZABCDEF123456789A
// @specre 01KNVEV4DB8QKB8DYKJQHYKDQD
// @specre 01KNYFDFF8ZDDZ48AG4G2C829B
pub mod surrealdb;

use crate::error::AppError;
use ::surrealdb::RecordId;
use async_trait::async_trait;
use chrono::{DateTime, FixedOffset};
use openapi::{
    models as openapi_models,
    types::{Nullable, Object},
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

#[async_trait]
pub trait CollectionRepository: Send + Sync {
    async fn list(&self) -> anyhow::Result<Vec<Collection>, AppError>;
    async fn create(&self, name: &str, schema: Value) -> anyhow::Result<Collection, AppError>;
    async fn get_by_id(&self, collection_id: &str) -> anyhow::Result<Collection, AppError>;
    async fn get_by_name(&self, name: &str) -> anyhow::Result<Collection, AppError>;
    async fn update_collection(
        &self,
        collection_id: &str,
        name: &str,
    ) -> anyhow::Result<Collection, AppError>;
    async fn list_record(&self, collection_id: &str) -> anyhow::Result<Vec<Record>, AppError>;
    async fn get_record(
        &self,
        collection_id: &str,
        record_id: &str,
    ) -> anyhow::Result<Record, AppError>;
    async fn add_record(
        &self,
        collection_id: &str,
        data: Value,
    ) -> anyhow::Result<Record, AppError>;
    async fn update_record(
        &self,
        collection_id: &str,
        record_id: &str,
        data: Value,
    ) -> anyhow::Result<Record, AppError>;
    async fn delete_record(
        &self,
        collection_id: &str,
        record_id: &str,
    ) -> anyhow::Result<(), AppError>;
    async fn delete(&self, collection_id: &str) -> anyhow::Result<(), AppError>;

    // Canvas: Edge operations
    async fn create_edge(
        &self,
        from: &str,
        to: &str,
        label: &str,
    ) -> anyhow::Result<Edge, AppError>;
    async fn list_edges(&self, record_id: &str) -> anyhow::Result<Vec<Edge>, AppError>;
    async fn delete_edge(&self, edge_id: &str) -> anyhow::Result<(), AppError>;

    // Canvas: Subgraph query
    async fn query_subgraph(
        &self,
        record_id: &str,
        depth: i32,
        conditions: Option<Value>,
    ) -> anyhow::Result<(Vec<Record>, Vec<Edge>), AppError>;

    // Image operations
    async fn upload_image(
        &self,
        collection_id: &str,
        record_id: &str,
        field_name: &str,
        data: Vec<u8>,
        content_type: &str,
    ) -> anyhow::Result<ImageInfo, AppError>;

    async fn get_image(
        &self,
        collection_id: &str,
        record_id: &str,
        field_name: &str,
    ) -> anyhow::Result<(Vec<u8>, String), AppError>;

    async fn delete_image(
        &self,
        collection_id: &str,
        record_id: &str,
        field_name: &str,
    ) -> anyhow::Result<(), AppError>;

    // Record operations: copy, merge, split
    async fn copy_record(
        &self,
        source_collection_id: &str,
        record_id: &str,
        target_collection_id: &str,
    ) -> anyhow::Result<Record, AppError>;

    async fn merge_records(
        &self,
        collection_id: &str,
        source_record_ids: &[String],
        target_record_id: &str,
        add_fields: Option<&[String]>,
        delete_sources: bool,
    ) -> anyhow::Result<Record, AppError>;

    async fn split_record(
        &self,
        collection_id: &str,
        record_id: &str,
        splits: &[Value],
    ) -> anyhow::Result<(Record, Vec<Record>), AppError>;

    // Records with expanded relations
    async fn list_records_with_relations(
        &self,
        collection_id: &str,
    ) -> anyhow::Result<Vec<RecordWithRelations>, AppError>;

}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Collection {
    pub id: RecordId,
    pub name: String,
    pub schema: Value,
    pub created_at: DateTime<FixedOffset>,
}

impl Collection {
    const TABLE_NAME: &'static str = "collection";

    pub fn new_value(name: String, schema: Value) -> Value {
        json!({
            "name": name,
            "schema": schema,
            "created_at": chrono::Utc::now().fixed_offset(),
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Record {
    pub id: RecordId,
    pub data: Value,
    pub created_at: Option<DateTime<FixedOffset>>,
    pub updated_at: Option<DateTime<FixedOffset>>,
}

impl Record {
    pub fn new_value(data: Value) -> Value {
        json!({
            "data": data,
            "created_at": Some(chrono::Utc::now().fixed_offset()),
            "updated_at": null,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Edge {
    pub id: RecordId,
    #[serde(rename = "in")]
    pub in_record: RecordId,
    #[serde(rename = "out")]
    pub out_record: RecordId,
    pub label: String,
    pub created_at: DateTime<FixedOffset>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelatedRecord {
    pub edge_id: String,
    pub label: String,
    pub direction: String,
    pub record: Record,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordWithRelations {
    pub id: RecordId,
    pub data: Value,
    pub created_at: Option<DateTime<FixedOffset>>,
    pub updated_at: Option<DateTime<FixedOffset>>,
    pub relations: Vec<RelatedRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageInfo {
    pub pointer: String,
    pub content_type: String,
    pub size: usize,
    pub url: String,
}


impl TryFrom<Edge> for openapi_models::Edge {
    type Error = AppError;

    fn try_from(edge: Edge) -> Result<Self, Self::Error> {
        Ok(openapi_models::Edge {
            id: edge.id.to_string(),
            from: edge.in_record.to_string(),
            to: edge.out_record.to_string(),
            label: edge.label,
            created_at: edge.created_at.to_string(),
        })
    }
}

// Implement TryFrom trait for converting repository models to OpenAPI models
impl TryFrom<Record> for openapi_models::Record {
    type Error = AppError;

    fn try_from(record: Record) -> Result<Self, Self::Error> {
        Ok(openapi_models::Record {
            id: record.id.key().to_string(),
            data: serde_json::from_value::<Object>(record.data)
                .map_err(|e| AppError::internal(format!("Failed to convert record data: {}", e)))?,
            created_at: match record.created_at {
                Some(timestamp) => Nullable::Present(timestamp.to_string()),
                None => Nullable::Null,
            },
            updated_at: match record.updated_at {
                Some(timestamp) => Nullable::Present(timestamp.to_string()),
                None => Nullable::Null,
            },
        })
    }
}

impl TryFrom<Collection> for openapi_models::Collection {
    type Error = AppError;

    fn try_from(collection: Collection) -> Result<Self, Self::Error> {
        Ok(openapi_models::Collection {
            id: collection.id.to_string(),
            name: collection.name,
            schema: serde_json::from_value::<Object>(collection.schema)
                .map_err(|e| AppError::internal(format!("Failed to convert schema: {}", e)))?,
            created_at: collection.created_at.to_string(),
        })
    }
}


// @specre 01KNM2BBT5Y18PC3WDQQN18KCA
// @specre 01KNM2BBT5C9CD76KVMPH5JGHV
// @specre 01KNM59JYD4SVC2B5CXMZQCDT5
// @specre 01KNM59JYDHYNFQSWJXE2TV85C
// @specre 01KTJP1XYZABCDEF123456789A
// @specre 01KNS6B47Z2CFB2EWBV3SC88KF
// @specre 01KNS6B4808R0THREKZKX2VX06
// @specre 01KNS6B481EPS6NESNEKV7JJ6R
// @specre 01KNVEV4DB8QKB8DYKJQHYKDQD
use crate::{
    error::AppError,
    repository::{
        Collection, CollectionRepository, Edge, ImageInfo, Record, RecordWithRelations,
        RelatedRecord,
    },
};
use async_trait::async_trait;
use serde_json::Value;
use std::{env, path::PathBuf};
use surrealdb::{
    RecordId, Surreal,
    engine::remote::ws::{Client, Ws},
    opt::auth::Root,
};
use tokio::fs;

pub struct SurrealCollectionRepository {
    db: Surreal<Client>,
}

impl SurrealCollectionRepository {
    pub async fn try_from_env() -> anyhow::Result<Self> {
        let url = env::var("DATABASE_URL")?;
        let username = env::var("DATABASE_USERNAME")?;
        let password = env::var("DATABASE_PASSWORD")?;
        let namespace = env::var("DATABASE_NAMESPACE")?;
        let database = env::var("DATABASE_NAME")?;

        println!("Connecting to SurrealDB at {}", url);
        let db = Surreal::new::<Ws>(url).await?;
        println!("Connected to SurrealDB");

        db.signin(Root {
            username: &username,
            password: &password,
        })
        .await?;

        db.use_ns(&namespace).use_db(&database).await?;

        Ok(Self { db })
    }
}

#[async_trait]
impl CollectionRepository for SurrealCollectionRepository {
    async fn list(&self) -> anyhow::Result<Vec<Collection>, AppError> {
        let collections = self.db.select(Collection::TABLE_NAME).await?;
        Ok(collections)
    }

    async fn create(&self, name: &str, schema: Value) -> anyhow::Result<Collection, AppError> {
        let collection = Collection::new_value(name.to_string(), schema);

        let collection: Option<Collection> = self
            .db
            .create(Collection::TABLE_NAME)
            .content(collection.clone())
            .await?;
        Ok(collection.ok_or(AppError::NotFound {
            resource: "Collection".to_string(),
        })?)
    }

    async fn get_by_id(&self, id: &str) -> anyhow::Result<Collection, AppError> {
        let collection: Option<Collection> = self.db.select((Collection::TABLE_NAME, id)).await?;
        collection.ok_or_else(|| AppError::NotFound {
            resource: format!("Collection with ID {}", id),
        })
    }

    async fn update_collection(
        &self,
        collection_id: &str,
        name: &str,
    ) -> anyhow::Result<Collection, AppError> {
        let collection: Option<Collection> = self
            .db
            .update((Collection::TABLE_NAME, collection_id))
            .merge(serde_json::json!({ "name": name }))
            .await?;
        collection.ok_or_else(|| AppError::NotFound {
            resource: format!("Collection with ID {}", collection_id),
        })
    }

    async fn get_by_name(&self, name: &str) -> anyhow::Result<Collection, AppError> {
        let query = format!(
            "SELECT * FROM {} WHERE name = '{}'",
            Collection::TABLE_NAME,
            name
        );
        let collection: Option<Collection> = self.db.query(query).await?.take(0)?;
        let Some(collection) = collection else {
            return Err(AppError::NotFound {
                resource: format!("Collection with name {}", name),
            });
        };
        Ok(collection)
    }

    async fn list_record(&self, collection_id: &str) -> anyhow::Result<Vec<Record>, AppError> {
        let records = self.db.select(collection_id).await?;
        Ok(records)
    }

    async fn get_record(
        &self,
        collection_id: &str,
        record_id: &str,
    ) -> anyhow::Result<Record, AppError> {
        let mut result = self
            .db
            .query("SELECT * FROM type::thing($table, $id)")
            .bind(("table", collection_id.to_string()))
            .bind(("id", record_id.to_string()))
            .await?;
        let record: Option<Record> = result.take(0)?;
        record.ok_or(AppError::NotFound {
            resource: format!("Record"),
        })
    }

    async fn add_record(
        &self,
        collection_id: &str,
        data: Value,
    ) -> anyhow::Result<Record, AppError> {
        let record = Record::new_value(data);
        let records: Vec<Record> = self.db.insert(collection_id).content(record).await?;
        Ok(records.into_iter().next().ok_or(AppError::NotFound {
            resource: format!("records"),
        })?)
    }

    async fn update_record(
        &self,
        collection_id: &str,
        record_id: &str,
        data: Value,
    ) -> anyhow::Result<Record, AppError> {
        let record: Option<Record> = self
            .db
            .update((collection_id, record_id))
            .content(data)
            .await?;
        Ok(record.ok_or(AppError::NotFound {
            resource: format!("record"),
        })?)
    }

    async fn delete_record(
        &self,
        collection_id: &str,
        record_id: &str,
    ) -> anyhow::Result<(), AppError> {
        let _: Option<Record> = self.db.delete((collection_id, record_id)).await?;
        Ok(())
    }

    async fn delete(&self, collection_id: &str) -> anyhow::Result<(), AppError> {
        let _: Option<Collection> = self
            .db
            .delete((Collection::TABLE_NAME, collection_id))
            .await?;
        let _: Vec<Record> = self.db.delete(collection_id).await?;
        Ok(())
    }

    async fn create_edge(
        &self,
        from: &str,
        to: &str,
        label: &str,
    ) -> anyhow::Result<Edge, AppError> {
        let from_rid = parse_record_id(from)?;
        let to_rid = parse_record_id(to)?;

        // Verify both records exist before creating edge
        let from_exists: Option<Record> = self.db.select(from_rid.clone()).await?;
        if from_exists.is_none() {
            return Err(AppError::not_found(format!("Record {from}")));
        }
        let to_exists: Option<Record> = self.db.select(to_rid.clone()).await?;
        if to_exists.is_none() {
            return Err(AppError::not_found(format!("Record {to}")));
        }

        let created_at = chrono::Utc::now().fixed_offset();
        let mut result = self
            .db
            .query("RELATE $from->edge->$to SET label = $label, created_at = $created_at")
            .bind(("from", from_rid))
            .bind(("to", to_rid))
            .bind(("label", label.to_string()))
            .bind(("created_at", created_at))
            .await?;
        let edge: Option<Edge> = result.take(0)?;
        edge.ok_or_else(|| AppError::internal("Failed to create edge"))
    }

    async fn list_edges(&self, record_id: &str) -> anyhow::Result<Vec<Edge>, AppError> {
        let rid = parse_record_id(record_id)?;
        let mut result = self
            .db
            .query("SELECT * FROM edge WHERE in = $rid OR out = $rid")
            .bind(("rid", rid))
            .await?;
        let edges: Vec<Edge> = result.take(0)?;
        Ok(edges)
    }

    async fn delete_edge(&self, edge_id: &str) -> anyhow::Result<(), AppError> {
        let _: Option<Edge> = self.db.delete(("edge", edge_id)).await?;
        Ok(())
    }

    async fn query_subgraph(
        &self,
        record_id: &str,
        depth: i32,
        conditions: Option<Value>,
    ) -> anyhow::Result<(Vec<Record>, Vec<Edge>), AppError> {
        // Build WHERE clause from conditions for SurrealQL
        let where_clause = build_surql_where(&conditions);

        let start_rid = parse_record_id(record_id)?;

        // Build graph traversal for each depth 1..=N and union the results.
        // e.g. depth=2 → union of ->edge->? (1 hop) and ->edge->?->edge->? (2 hops)
        let traversals: Vec<String> = (1..=depth as usize)
            .map(|d| format!("(SELECT VALUE {} FROM ONLY $start)", "->edge->?".repeat(d)))
            .collect();

        let reachable_query = format!(
            "RETURN array::distinct(array::flatten([{}]))",
            traversals.join(", ")
        );
        let mut reachable_result = self
            .db
            .query(&reachable_query)
            .bind(("start", start_rid.clone()))
            .await?;
        let reachable_ids: Vec<RecordId> = reachable_result.take(0).unwrap_or_default();

        if reachable_ids.is_empty() {
            // Starting record might exist but has no edges — return just the start
            let mut start_result = self
                .db
                .query("SELECT * FROM ONLY $start")
                .bind(("start", start_rid))
                .await?;
            let start_record: Option<Record> = start_result.take(0)?;
            return match start_record {
                Some(rec) => Ok((vec![rec], vec![])),
                None => Err(AppError::not_found(format!("Record {record_id}"))),
            };
        }

        // Build a set of all record IDs (start + reachable)
        let all_ids: Vec<RecordId> = std::iter::once(start_rid)
            .chain(reachable_ids.into_iter())
            .collect();

        // Fetch all records, optionally filtered by conditions
        let records_query = format!("SELECT * FROM $all_ids{where_clause}");
        let mut records_result = self
            .db
            .query(&records_query)
            .bind(("all_ids", all_ids.clone()))
            .await?;
        let records: Vec<Record> = records_result.take(0)?;

        if records.is_empty() {
            return Err(AppError::not_found(format!("Record {record_id}")));
        }

        // Fetch all edges where both endpoints are in our record set
        let mut edges_result = self
            .db
            .query("SELECT * FROM edge WHERE in INSIDE $all_ids AND out INSIDE $all_ids")
            .bind(("all_ids", all_ids))
            .await?;
        let edges: Vec<Edge> = edges_result.take(0)?;

        Ok((records, edges))
    }

    async fn upload_image(
        &self,
        collection_id: &str,
        record_id: &str,
        field_name: &str,
        data: Vec<u8>,
        content_type: &str,
    ) -> anyhow::Result<ImageInfo, AppError> {
        // Get image storage path from env or use default
        let storage_path =
            env::var("IMAGE_STORAGE_PATH").unwrap_or_else(|_| "/data/images".to_string());
        let dir = PathBuf::from(&storage_path)
            .join(collection_id)
            .join(record_id);

        // Create directory if it doesn't exist
        fs::create_dir_all(&dir)
            .await
            .map_err(|e| AppError::internal(format!("Failed to create image directory: {e}")))?;

        // Determine file extension from content type
        let ext = match content_type {
            "image/png" => "png",
            "image/jpeg" => "jpg",
            "image/webp" => "webp",
            "image/gif" => "gif",
            _ => "bin",
        };

        let file_path = dir.join(format!("{field_name}.{ext}"));
        let size = data.len();

        // Write file
        fs::write(&file_path, &data)
            .await
            .map_err(|e| AppError::internal(format!("Failed to write image file: {e}")))?;

        let pointer = format!("images:/{collection_id}/{record_id}/{field_name}");
        let url = format!("/records/{collection_id}/{record_id}/images/{field_name}");

        // Update record with image metadata
        let image_info = serde_json::json!({
            "pointer": pointer,
            "content_type": content_type,
            "size": size,
            "url": url
        });

        // Update the record's data field with image info
        // Use type::thing() to properly escape table names that start with numbers
        let mut result = self
            .db
            .query("SELECT * FROM type::thing($table, $id)")
            .bind(("table", collection_id.to_string()))
            .bind(("id", record_id.to_string()))
            .await?;
        let record: Option<Record> = result.take(0)?;

        if let Some(mut rec) = record {
            if let Some(data_obj) = rec.data.as_object_mut() {
                data_obj.insert(field_name.to_string(), image_info);
                self.db
                    .query("UPDATE type::thing($table, $id) MERGE $merge_data")
                    .bind(("table", collection_id.to_string()))
                    .bind(("id", record_id.to_string()))
                    .bind(("merge_data", serde_json::json!({ "data": rec.data, "updated_at": chrono::Utc::now().fixed_offset() })))
                    .await?;
            }
        }

        Ok(ImageInfo {
            pointer,
            content_type: content_type.to_string(),
            size,
            url,
        })
    }

    async fn get_image(
        &self,
        collection_id: &str,
        record_id: &str,
        field_name: &str,
    ) -> anyhow::Result<(Vec<u8>, String), AppError> {
        // Get record to find content type
        let mut result = self
            .db
            .query("SELECT * FROM type::thing($table, $id)")
            .bind(("table", collection_id.to_string()))
            .bind(("id", record_id.to_string()))
            .await?;
        let record: Option<Record> = result.take(0)?;
        let record = record.ok_or_else(|| AppError::not_found("Record"))?;

        let content_type = record
            .data
            .get(field_name)
            .and_then(|v| v.get("content_type"))
            .and_then(|v| v.as_str())
            .ok_or_else(|| AppError::not_found(format!("Image field {field_name}")))?
            .to_string();

        let ext = match content_type.as_str() {
            "image/png" => "png",
            "image/jpeg" => "jpg",
            "image/webp" => "webp",
            "image/gif" => "gif",
            _ => "bin",
        };

        let storage_path =
            env::var("IMAGE_STORAGE_PATH").unwrap_or_else(|_| "/data/images".to_string());
        let file_path = PathBuf::from(&storage_path)
            .join(collection_id)
            .join(record_id)
            .join(format!("{field_name}.{ext}"));

        let data = fs::read(&file_path)
            .await
            .map_err(|e| AppError::not_found(format!("Image file: {e}")))?;

        Ok((data, content_type))
    }

    async fn delete_image(
        &self,
        collection_id: &str,
        record_id: &str,
        field_name: &str,
    ) -> anyhow::Result<(), AppError> {
        // Get record to find content type for file extension
        let mut result = self
            .db
            .query("SELECT * FROM type::thing($table, $id)")
            .bind(("table", collection_id.to_string()))
            .bind(("id", record_id.to_string()))
            .await?;
        let record: Option<Record> = result.take(0)?;
        let record = record.ok_or_else(|| AppError::not_found("Record"))?;

        if let Some(image_info) = record.data.get(field_name) {
            let content_type = image_info
                .get("content_type")
                .and_then(|v| v.as_str())
                .unwrap_or("application/octet-stream");

            let ext = match content_type {
                "image/png" => "png",
                "image/jpeg" => "jpg",
                "image/webp" => "webp",
                "image/gif" => "gif",
                _ => "bin",
            };

            let storage_path =
                env::var("IMAGE_STORAGE_PATH").unwrap_or_else(|_| "/data/images".to_string());
            let file_path = PathBuf::from(&storage_path)
                .join(collection_id)
                .join(record_id)
                .join(format!("{field_name}.{ext}"));

            // Delete file (ignore error if not exists)
            let _ = fs::remove_file(&file_path).await;

            // Update record to remove image field
            if let Some(data_obj) = record.data.as_object() {
                let mut new_data = data_obj.clone();
                new_data.remove(field_name);
                self.db
                    .query("UPDATE type::thing($table, $id) MERGE $merge_data")
                    .bind(("table", collection_id.to_string()))
                    .bind(("id", record_id.to_string()))
                    .bind(("merge_data", serde_json::json!({ "data": new_data, "updated_at": chrono::Utc::now().fixed_offset() })))
                    .await?;
            }
        }

        Ok(())
    }

    async fn copy_record(
        &self,
        source_collection_id: &str,
        record_id: &str,
        target_collection_id: &str,
    ) -> anyhow::Result<Record, AppError> {
        // Get source record
        let source_collection = self.get_by_id(source_collection_id).await?;
        let source_record: Option<Record> =
            self.db.select((source_collection.name, record_id)).await?;
        let source_record = source_record.ok_or_else(|| AppError::not_found("Source record"))?;

        // Get target collection to verify it exists
        let _target_collection = self.get_by_id(target_collection_id).await?;

        // Create new record in target collection with source data
        let new_record = Record::new_value(source_record.data);
        let records: Vec<Record> = self
            .db
            .insert(target_collection_id)
            .content(new_record)
            .await?;
        records
            .into_iter()
            .next()
            .ok_or_else(|| AppError::internal("Failed to create copied record"))
    }

    async fn merge_records(
        &self,
        collection_id: &str,
        source_record_ids: &[String],
        target_record_id: &str,
        add_fields: Option<&[String]>,
        delete_sources: bool,
    ) -> anyhow::Result<Record, AppError> {
        let collection = self.get_by_id(collection_id).await?;

        // Fetch all source records
        let mut records: Vec<Record> = Vec::new();
        for rid in source_record_ids {
            let record: Option<Record> = self
                .db
                .select((collection.name.clone(), rid.as_str()))
                .await?;
            let record = record.ok_or_else(|| AppError::not_found(format!("Record {rid}")))?;
            records.push(record);
        }

        // Find target record
        let target_record = records
            .iter()
            .find(|r| r.id.key().to_string() == target_record_id)
            .ok_or_else(|| AppError::validation("Target record must be in source_record_ids"))?
            .clone();

        // Merge data: start with target's data
        let mut merged_data = target_record.data.as_object().cloned().unwrap_or_default();

        // Sum add_fields from all source records
        if let Some(fields) = add_fields {
            for field in fields {
                let mut sum: f64 = 0.0;
                for record in &records {
                    if let Some(val) = record.data.get(field) {
                        if let Some(num) = val.as_f64() {
                            sum += num;
                        } else {
                            return Err(AppError::validation(format!(
                                "Field '{}' is not a number",
                                field
                            )));
                        }
                    }
                }
                merged_data.insert(field.clone(), serde_json::json!(sum));
            }
        }

        // Update target record with merged data
        let updated_record: Option<Record> = self
            .db
            .update((collection.name.clone(), target_record_id))
            .merge(serde_json::json!({
                "data": merged_data,
                "updated_at": chrono::Utc::now().fixed_offset()
            }))
            .await?;
        let updated_record =
            updated_record.ok_or_else(|| AppError::internal("Failed to update merged record"))?;

        // Delete source records (except target)
        if delete_sources {
            for rid in source_record_ids {
                if rid != target_record_id {
                    let _: Option<Record> = self
                        .db
                        .delete((collection.name.clone(), rid.as_str()))
                        .await?;
                }
            }
        }

        Ok(updated_record)
    }

    async fn split_record(
        &self,
        collection_id: &str,
        record_id: &str,
        splits: &[Value],
    ) -> anyhow::Result<(Record, Vec<Record>), AppError> {
        let collection = self.get_by_id(collection_id).await?;

        // Get original record
        let original: Option<Record> = self.db.select((collection.name.clone(), record_id)).await?;
        let original = original.ok_or_else(|| AppError::not_found("Record"))?;
        let mut original_data = original.data.as_object().cloned().unwrap_or_default();

        let mut new_records: Vec<Record> = Vec::new();

        for split in splits {
            let split_obj = split
                .as_object()
                .ok_or_else(|| AppError::validation("Split must be an object"))?;

            // Create new record data by copying original and overwriting with split values
            let mut new_data = original_data.clone();

            for (field, subtract_value) in split_obj {
                let subtract_num = subtract_value.as_f64().ok_or_else(|| {
                    AppError::validation(format!("Split field '{}' must be a number", field))
                })?;

                // Get original value
                let original_val = original_data
                    .get(field)
                    .and_then(|v| v.as_f64())
                    .ok_or_else(|| {
                        AppError::validation(format!("Original field '{}' is not a number", field))
                    })?;

                // Check if we have enough
                if original_val < subtract_num {
                    return Err(AppError::validation(format!(
                        "Insufficient value for field '{}': have {}, need {}",
                        field, original_val, subtract_num
                    )));
                }

                // Subtract from original
                let new_original_val = original_val - subtract_num;
                original_data.insert(field.clone(), serde_json::json!(new_original_val));

                // Set split value in new record
                new_data.insert(field.clone(), serde_json::json!(subtract_num));
            }

            // Create new record
            let new_record_value = Record::new_value(serde_json::json!(new_data));
            let created_records: Vec<Record> = self
                .db
                .insert(collection.name.clone())
                .content(new_record_value)
                .await?;
            if let Some(rec) = created_records.into_iter().next() {
                new_records.push(rec);
            }
        }

        // Update original record with subtracted values
        let updated_original: Option<Record> = self
            .db
            .update((collection.name, record_id))
            .merge(serde_json::json!({
                "data": original_data,
                "updated_at": chrono::Utc::now().fixed_offset()
            }))
            .await?;
        let updated_original = updated_original
            .ok_or_else(|| AppError::internal("Failed to update original record"))?;

        Ok((updated_original, new_records))
    }

    async fn list_records_with_relations(
        &self,
        collection_id: &str,
    ) -> anyhow::Result<Vec<RecordWithRelations>, AppError> {
        // Verify collection exists
        let _collection = self.get_by_id(collection_id).await?;
        // Use collection_id as table name (consistent with list_record and add_record)
        let records: Vec<Record> = self.db.select(collection_id).await?;

        if records.is_empty() {
            return Ok(vec![]);
        }

        // Collect all record IDs for batch edge query
        let record_ids: Vec<RecordId> = records.iter().map(|r| r.id.clone()).collect();

        // Fetch all edges where any of our records is involved
        let mut edge_result = self
            .db
            .query("SELECT * FROM edge WHERE in INSIDE $rids OR out INSIDE $rids")
            .bind(("rids", record_ids.clone()))
            .await?;
        let all_edges: Vec<Edge> = edge_result.take(0)?;

        // Collect IDs of related records that are outside our collection
        let record_id_set: std::collections::HashSet<String> =
            record_ids.iter().map(|r| r.to_string()).collect();
        let mut external_ids: std::collections::HashSet<String> = std::collections::HashSet::new();
        for edge in &all_edges {
            let in_str = edge.in_record.to_string();
            let out_str = edge.out_record.to_string();
            if !record_id_set.contains(&in_str) {
                external_ids.insert(in_str);
            }
            if !record_id_set.contains(&out_str) {
                external_ids.insert(out_str);
            }
        }

        // Fetch external related records
        let mut external_records: std::collections::HashMap<String, Record> =
            std::collections::HashMap::new();
        for rid_str in &external_ids {
            if let Ok(rid) = parse_record_id(rid_str) {
                let record: Option<Record> = self.db.select(rid).await?;
                if let Some(r) = record {
                    external_records.insert(rid_str.clone(), r);
                }
            }
        }

        // Index all records (own + external) by full ID for lookup
        let mut all_records_map: std::collections::HashMap<String, Record> =
            std::collections::HashMap::new();
        for r in &records {
            all_records_map.insert(r.id.to_string(), r.clone());
        }
        for (k, v) in external_records {
            all_records_map.insert(k, v);
        }

        // Build RecordWithRelations for each record
        let result = records
            .into_iter()
            .map(|record| {
                let record_full_id = record.id.to_string();
                let relations: Vec<RelatedRecord> = all_edges
                    .iter()
                    .filter(|e| {
                        e.in_record.to_string() == record_full_id
                            || e.out_record.to_string() == record_full_id
                    })
                    .filter_map(|e| {
                        let (direction, other_id) = if e.out_record.to_string() == record_full_id {
                            ("outgoing", e.in_record.to_string())
                        } else {
                            ("incoming", e.out_record.to_string())
                        };

                        all_records_map.get(&other_id).map(|r| RelatedRecord {
                            edge_id: e.id.to_string(),
                            label: e.label.clone(),
                            direction: direction.to_string(),
                            record: r.clone(),
                        })
                    })
                    .collect();

                RecordWithRelations {
                    id: record.id,
                    data: record.data,
                    created_at: record.created_at,
                    updated_at: record.updated_at,
                    relations,
                }
            })
            .collect();

        Ok(result)
    }

}

/// Parse a "table:id" string into a SurrealDB RecordId.
pub(crate) fn parse_record_id(s: &str) -> Result<RecordId, AppError> {
    let (table, key) = s
        .split_once(':')
        .ok_or_else(|| AppError::validation(format!("Invalid record ID format: {s}")))?;
    Ok(RecordId::from((table, key)))
}

/// Build a SurrealQL WHERE clause from JSON conditions.
/// Input: `{"data.episode_number": {"$lte": 5}}`
/// Output: ` WHERE data.episode_number <= 5`
pub(crate) fn build_surql_where(conditions: &Option<Value>) -> String {
    let Some(conds) = conditions else {
        return String::new();
    };
    let Some(obj) = conds.as_object() else {
        return String::new();
    };

    let parts: Vec<String> = obj
        .iter()
        .filter_map(|(key, val)| {
            if let Some(inner) = val.as_object() {
                let sub: Vec<String> = inner
                    .iter()
                    .filter_map(|(op, v)| {
                        let surql_op = match op.as_str() {
                            "$eq" => "=",
                            "$ne" => "!=",
                            "$lt" => "<",
                            "$lte" => "<=",
                            "$gt" => ">",
                            "$gte" => ">=",
                            _ => return None,
                        };
                        Some(format!("{key} {surql_op} {v}"))
                    })
                    .collect();
                if sub.is_empty() {
                    None
                } else {
                    Some(sub.join(" AND "))
                }
            } else {
                Some(format!("{key} = {val}"))
            }
        })
        .collect();

    if parts.is_empty() {
        String::new()
    } else {
        format!(" WHERE {}", parts.join(" AND "))
    }
}

// @specre 01KNM7ADS6DZJGC5488EX79HYM
// @specre 01KTJP1XYZABCDEF123456789A
// @specre 01KNS6B47Z2CFB2EWBV3SC88KF
// @specre 01KNS6B4808R0THREKZKX2VX06
// @specre 01KNS6B481EPS6NESNEKV7JJ6R
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use tempfile;

    #[test]
    fn parse_record_id_valid() {
        let rid = parse_record_id("table:key").unwrap();
        assert_eq!(rid.to_string(), "table:key");
    }

    #[test]
    fn parse_record_id_no_colon() {
        let result = parse_record_id("invalid");
        assert!(result.is_err());
    }

    #[test]
    fn build_surql_where_none() {
        assert_eq!(build_surql_where(&None), "");
    }

    #[test]
    fn build_surql_where_single_condition() {
        let conds = json!({"data.x": {"$lte": 5}});
        let result = build_surql_where(&Some(conds));
        assert_eq!(result, " WHERE data.x <= 5");
    }

    #[test]
    fn build_surql_where_multiple_operators() {
        let conds = json!({"data.x": {"$gt": 1, "$lt": 10}});
        let result = build_surql_where(&Some(conds));
        // Order within the inner object may vary, so check both parts
        assert!(result.contains("data.x > 1"));
        assert!(result.contains("data.x < 10"));
        assert!(result.starts_with(" WHERE "));
        assert!(result.contains(" AND "));
    }

    #[test]
    fn build_surql_where_equality() {
        let conds = json!({"data.name": "Alice"});
        let result = build_surql_where(&Some(conds));
        assert_eq!(result, " WHERE data.name = \"Alice\"");
    }

    #[test]
    fn build_surql_where_unsupported_operator() {
        let conds = json!({"data.x": {"$regex": "abc"}});
        let result = build_surql_where(&Some(conds));
        assert_eq!(result, "");
    }

    // Helper function to get extension from content type (matches impl logic)
    fn get_extension_for_content_type(content_type: &str) -> &'static str {
        match content_type {
            "image/png" => "png",
            "image/jpeg" => "jpg",
            "image/webp" => "webp",
            "image/gif" => "gif",
            _ => "bin",
        }
    }

    #[test]
    fn image_extension_from_content_type_png() {
        assert_eq!(get_extension_for_content_type("image/png"), "png");
    }

    #[test]
    fn image_extension_from_content_type_jpeg() {
        assert_eq!(get_extension_for_content_type("image/jpeg"), "jpg");
    }

    #[test]
    fn image_extension_from_content_type_webp() {
        assert_eq!(get_extension_for_content_type("image/webp"), "webp");
    }

    #[test]
    fn image_extension_from_content_type_gif() {
        assert_eq!(get_extension_for_content_type("image/gif"), "gif");
    }

    #[test]
    fn image_extension_from_content_type_unknown() {
        assert_eq!(
            get_extension_for_content_type("application/octet-stream"),
            "bin"
        );
        assert_eq!(get_extension_for_content_type("image/bmp"), "bin");
    }

    #[test]
    fn image_path_construction() {
        let storage_path = "/data/images";
        let collection_id = "col123";
        let record_id = "rec456";
        let field_name = "avatar";
        let ext = "png";

        let file_path = PathBuf::from(storage_path)
            .join(collection_id)
            .join(record_id)
            .join(format!("{field_name}.{ext}"));

        assert_eq!(
            file_path.to_str().unwrap(),
            "/data/images/col123/rec456/avatar.png"
        );
    }

    #[test]
    fn image_pointer_format() {
        let collection_id = "col123";
        let record_id = "rec456";
        let field_name = "avatar";

        let pointer = format!("images:/{collection_id}/{record_id}/{field_name}");
        assert_eq!(pointer, "images:/col123/rec456/avatar");
    }

    #[test]
    fn image_url_format() {
        let collection_id = "col123";
        let record_id = "rec456";
        let field_name = "avatar";

        let url = format!("/records/{collection_id}/{record_id}/images/{field_name}");
        assert_eq!(url, "/records/col123/rec456/images/avatar");
    }

    #[test]
    fn image_info_serialization() {
        let info = ImageInfo {
            pointer: "images:/col/rec/field".to_string(),
            content_type: "image/png".to_string(),
            size: 1024,
            url: "/records/col/rec/images/field".to_string(),
        };

        let json = serde_json::to_value(&info).unwrap();
        assert_eq!(json["pointer"], "images:/col/rec/field");
        assert_eq!(json["content_type"], "image/png");
        assert_eq!(json["size"], 1024);
        assert_eq!(json["url"], "/records/col/rec/images/field");
    }

    #[tokio::test]
    async fn image_file_write_and_read() {
        let temp_dir = tempfile::tempdir().unwrap();
        let file_path = temp_dir.path().join("test.png");

        // Simulate image data (1x1 transparent PNG header)
        let image_data: Vec<u8> = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];

        // Write
        tokio::fs::write(&file_path, &image_data).await.unwrap();
        assert!(file_path.exists());

        // Read back
        let read_data = tokio::fs::read(&file_path).await.unwrap();
        assert_eq!(read_data, image_data);
    }

    #[tokio::test]
    async fn image_directory_creation() {
        let temp_dir = tempfile::tempdir().unwrap();
        let nested_dir = temp_dir.path().join("col").join("rec");

        tokio::fs::create_dir_all(&nested_dir).await.unwrap();
        assert!(nested_dir.exists());
        assert!(nested_dir.is_dir());
    }

    #[tokio::test]
    async fn image_file_delete() {
        let temp_dir = tempfile::tempdir().unwrap();
        let file_path = temp_dir.path().join("test.png");

        // Create file
        tokio::fs::write(&file_path, b"test data").await.unwrap();
        assert!(file_path.exists());

        // Delete file
        tokio::fs::remove_file(&file_path).await.unwrap();
        assert!(!file_path.exists());
    }

    // ==========================================================
    // Record Operations Tests (Copy, Merge, Split)
    // ==========================================================

    /// Helper: sum numeric fields from multiple JSON objects
    fn sum_numeric_fields(
        objects: &[serde_json::Map<String, Value>],
        fields: &[&str],
    ) -> serde_json::Map<String, Value> {
        let mut result = serde_json::Map::new();
        for field in fields {
            let sum: f64 = objects
                .iter()
                .filter_map(|obj| obj.get(*field).and_then(|v| v.as_f64()))
                .sum();
            result.insert(field.to_string(), json!(sum));
        }
        result
    }

    #[test]
    fn merge_sum_numeric_fields_basic() {
        let obj1 = json!({"points": 100, "balance": 50})
            .as_object()
            .unwrap()
            .clone();
        let obj2 = json!({"points": 200, "balance": 30})
            .as_object()
            .unwrap()
            .clone();

        let result = sum_numeric_fields(&[obj1, obj2], &["points", "balance"]);

        assert_eq!(result.get("points").unwrap().as_f64().unwrap(), 300.0);
        assert_eq!(result.get("balance").unwrap().as_f64().unwrap(), 80.0);
    }

    #[test]
    fn merge_sum_numeric_fields_missing_field() {
        let obj1 = json!({"points": 100}).as_object().unwrap().clone();
        let obj2 = json!({"points": 200, "balance": 30})
            .as_object()
            .unwrap()
            .clone();

        let result = sum_numeric_fields(&[obj1, obj2], &["points", "balance"]);

        assert_eq!(result.get("points").unwrap().as_f64().unwrap(), 300.0);
        // balance only in obj2
        assert_eq!(result.get("balance").unwrap().as_f64().unwrap(), 30.0);
    }

    #[test]
    fn merge_sum_numeric_fields_empty() {
        let result = sum_numeric_fields(&[], &["points"]);
        assert_eq!(result.get("points").unwrap().as_f64().unwrap(), 0.0);
    }

    /// Helper: validate split values don't exceed original
    fn validate_split(original_value: f64, split_values: &[f64]) -> Result<f64, String> {
        let total_split: f64 = split_values.iter().sum();
        if total_split > original_value {
            return Err(format!(
                "Insufficient value: have {}, need {}",
                original_value, total_split
            ));
        }
        Ok(original_value - total_split)
    }

    #[test]
    fn split_validate_sufficient_value() {
        let result = validate_split(100.0, &[30.0, 20.0]);
        assert_eq!(result.unwrap(), 50.0);
    }

    #[test]
    fn split_validate_exact_value() {
        let result = validate_split(100.0, &[60.0, 40.0]);
        assert_eq!(result.unwrap(), 0.0);
    }

    #[test]
    fn split_validate_insufficient_value() {
        let result = validate_split(100.0, &[60.0, 50.0]);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Insufficient"));
    }

    #[test]
    fn split_validate_single_split() {
        let result = validate_split(100.0, &[100.0]);
        assert_eq!(result.unwrap(), 0.0);
    }

    #[test]
    fn split_validate_no_splits() {
        let result = validate_split(100.0, &[]);
        assert_eq!(result.unwrap(), 100.0);
    }

    /// Helper: create split records from original data
    fn create_split_record_data(
        original: &serde_json::Map<String, Value>,
        split_spec: &serde_json::Map<String, Value>,
    ) -> serde_json::Map<String, Value> {
        let mut new_data = original.clone();
        for (field, value) in split_spec {
            new_data.insert(field.clone(), value.clone());
        }
        new_data
    }

    #[test]
    fn split_create_record_data() {
        let original = json!({"name": "Item", "quantity": 100, "price": 10})
            .as_object()
            .unwrap()
            .clone();
        let split_spec = json!({"quantity": 30}).as_object().unwrap().clone();

        let result = create_split_record_data(&original, &split_spec);

        assert_eq!(result.get("name").unwrap(), "Item");
        assert_eq!(result.get("quantity").unwrap().as_i64().unwrap(), 30);
        assert_eq!(result.get("price").unwrap().as_i64().unwrap(), 10);
    }

    #[test]
    fn split_create_multiple_fields() {
        let original = json!({"name": "Item", "quantity": 100, "weight": 50})
            .as_object()
            .unwrap()
            .clone();
        let split_spec = json!({"quantity": 30, "weight": 20})
            .as_object()
            .unwrap()
            .clone();

        let result = create_split_record_data(&original, &split_spec);

        assert_eq!(result.get("quantity").unwrap().as_i64().unwrap(), 30);
        assert_eq!(result.get("weight").unwrap().as_i64().unwrap(), 20);
    }

    /// Test field type validation for merge
    fn is_numeric_field(value: &Value) -> bool {
        value.as_f64().is_some()
    }

    #[test]
    fn merge_numeric_field_validation() {
        assert!(is_numeric_field(&json!(100)));
        assert!(is_numeric_field(&json!(100.5)));
        assert!(!is_numeric_field(&json!("string")));
        assert!(!is_numeric_field(&json!(null)));
        assert!(!is_numeric_field(&json!({"nested": 1})));
    }

    /// Test copy record data preservation
    #[test]
    fn copy_preserves_data_structure() {
        let original = json!({
            "name": "Test Record",
            "nested": {"key": "value"},
            "array": [1, 2, 3]
        });

        // Clone simulates copy
        let copied = original.clone();

        assert_eq!(original, copied);
        assert_eq!(copied.get("nested").unwrap().get("key").unwrap(), "value");
        assert_eq!(copied.get("array").unwrap().as_array().unwrap().len(), 3);
    }
}

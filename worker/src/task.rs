use surrealdb::opt::RecordId;
use uuid::Uuid;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub(crate) struct Task {
    pub(crate) id: RecordId,
}

impl Task {
    pub(crate) fn new() -> Self {
        Self {
            id: RecordId::from(("task".to_string(), Uuid::new_v4().to_string())),
        }
    }
}

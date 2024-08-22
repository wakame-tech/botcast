use surrealdb::opt::RecordId;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct Task {
    id: RecordId,
}

impl Task {
    fn new(id: usize) -> Self {
        Self {
            id: RecordId::from(("task".to_string(), format!("{}", id))),
        }
    }
}

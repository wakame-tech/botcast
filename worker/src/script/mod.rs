use surrealdb::opt::RecordId;
use uuid::Uuid;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub(crate) struct Serif {
    pub(crate) text: String,
    speaker_id: String,
}

impl Serif {
    pub(crate) fn new(text: String, speaker_id: String) -> Self {
        Self { text, speaker_id }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub(crate) struct Episode {
    pub(crate) id: RecordId,
    title: String,
    pub(crate) serifs: Vec<Serif>,
}
impl Episode {
    pub(crate) fn new(title: String, serifs: Vec<Serif>) -> Self {
        Self {
            id: RecordId::from(("episode".to_string(), Uuid::new_v4().to_string())),
            title,
            serifs,
        }
    }
}

use crate::voicevox::Speaker;
use surrealdb::opt::RecordId;
use uuid::Uuid;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub(crate) struct Serif {
    text: String,
    speaker: Speaker,
}

impl Serif {
    pub(crate) fn new(text: String, speaker: Speaker) -> Self {
        Self { text, speaker }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub(crate) struct Episode {
    id: RecordId,
    title: String,
    serifs: Vec<Serif>,
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

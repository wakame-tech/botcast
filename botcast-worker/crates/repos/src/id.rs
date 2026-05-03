use uuid::Uuid;

// NOTE: #[derive(sqlx::Type)] + #[sqlx(transparent)] cannot useable in query_as! macro
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(transparent)]
pub struct PodcastId(pub Uuid);

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(transparent)]
pub struct EpisodeId(pub Uuid);

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(transparent)]
pub struct TaskId(pub Uuid);

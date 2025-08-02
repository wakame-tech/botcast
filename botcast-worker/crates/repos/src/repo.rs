use crate::{
    entities::{
        corners::Model as Corner, episodes::Model as Episode, mails::Model as Mail,
        podcasts::Model as Podcast, scripts::Model as Script, tasks::Model as Task,
        users::Model as User,
    },
    error::Error,
    id::{CornerId, EpisodeId, MailId, PodcastId, ScriptId, TaskId},
};
use async_trait::async_trait;
use chrono::{DateTime, FixedOffset, Utc};
use uuid::Uuid;

#[derive(Debug, serde::Serialize, sqlx::FromRow)]
pub struct Secret {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub secret: String,
    pub decrypted_secret: String,
    pub key_id: Option<Uuid>,
    pub created_at: DateTime<FixedOffset>,
    pub updated_at: DateTime<FixedOffset>,
}

#[async_trait]
pub trait UserRepo: Send + Sync {
    async fn find_by_id(&self, id: &Uuid) -> anyhow::Result<User, Error>;
    async fn find_by_auth_id(&self, auth_id: &Uuid) -> anyhow::Result<User, Error>;
}

#[async_trait]
pub trait PodcastRepo: Send + Sync {
    async fn list(&self, user_id: &Uuid) -> anyhow::Result<Vec<(Podcast, Option<User>)>, Error>;
    async fn latests(&self, limit: u64) -> anyhow::Result<Vec<(Podcast, Option<User>)>, Error>;
    async fn find_by_id(&self, id: &PodcastId) -> Result<Podcast, Error>;
    async fn create(&self, podcast: Podcast) -> anyhow::Result<(), Error>;
    async fn update(&self, podcast: Podcast) -> anyhow::Result<(), Error>;
    async fn delete(&self, id: &PodcastId) -> anyhow::Result<(), Error>;
}

#[async_trait]
pub trait EpisodeRepo: Send + Sync {
    async fn find_by_id(&self, id: &EpisodeId) -> anyhow::Result<Episode, Error>;
    async fn find_all_by_podcast_id(
        &self,
        podcast_id: &PodcastId,
    ) -> anyhow::Result<Vec<Episode>, Error>;
    async fn create(&self, episode: Episode) -> anyhow::Result<(), Error>;
    async fn update(&self, episode: Episode) -> anyhow::Result<(), Error>;
    async fn delete(&self, id: &EpisodeId) -> anyhow::Result<(), Error>;
}

#[async_trait]
pub trait ScriptRepo: Send + Sync {
    async fn list(&self, user_id: &Uuid) -> anyhow::Result<Vec<Script>, Error>;
    async fn find_by_id(&self, id: &ScriptId) -> anyhow::Result<Script, Error>;
    async fn create(&self, script: Script) -> anyhow::Result<(), Error>;
    async fn update(&self, script: Script) -> anyhow::Result<(), Error>;
    async fn delete(&self, id: &ScriptId) -> anyhow::Result<(), Error>;
}

#[async_trait]
pub trait CornerRepo: Send + Sync {
    async fn find_by_id(&self, id: &CornerId) -> anyhow::Result<(Corner, Option<User>), Error>;
    async fn find_all_by_podcast_id(
        &self,
        podcast_id: &PodcastId,
    ) -> anyhow::Result<Vec<(Corner, Option<User>)>, Error>;
    async fn create(&self, corner: Corner) -> anyhow::Result<(), Error>;
    async fn update(&self, corner: Corner) -> anyhow::Result<(), Error>;
    async fn delete(&self, id: &CornerId) -> anyhow::Result<(), Error>;
}

#[async_trait]
pub trait MailRepo: Send + Sync {
    async fn list(&self, corner_id: &CornerId) -> anyhow::Result<Vec<(Mail, Option<User>)>, Error>;
    async fn find_by_id(&self, id: &MailId) -> anyhow::Result<(Mail, Option<User>), Error>;
    async fn create(&self, mail: Mail) -> anyhow::Result<(), Error>;
    async fn update(&self, mail: Mail) -> anyhow::Result<(), Error>;
    async fn delete(&self, id: &MailId) -> anyhow::Result<(), Error>;
}

#[async_trait]
pub trait TaskRepo: Send + Sync {
    async fn pop(&self, now: DateTime<Utc>) -> anyhow::Result<Option<Task>, Error>;
    async fn list(&self, user_id: &Uuid) -> anyhow::Result<Vec<Task>, Error>;
    async fn find_by_id(&self, id: &TaskId) -> anyhow::Result<Task, Error>;
    async fn create(&self, task: Task) -> anyhow::Result<(), Error>;
    async fn update(&self, task: Task) -> anyhow::Result<(), Error>;
    #[allow(dead_code)]
    async fn delete(&self, id: &TaskId) -> anyhow::Result<(), Error>;
}

#[async_trait]
pub trait SecretRepo: Send + Sync {
    async fn list(&self, user_id: &Uuid) -> anyhow::Result<Vec<Secret>, Error>;
    async fn find_by_name(&self, user_id: &Uuid, name: &str) -> anyhow::Result<Secret, Error>;
    async fn create(
        &self,
        user_id: &Uuid,
        secret: &String,
        name: &String,
    ) -> anyhow::Result<(), Error>;
    async fn update(
        &self,
        user_id: &Uuid,
        id: &Uuid,
        secret: &String,
        name: &String,
    ) -> anyhow::Result<(), Error>;
    async fn delete(&self, user_id: &Uuid, id: &Uuid) -> anyhow::Result<(), Error>;
}

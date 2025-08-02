use crate::entities::corners::{self, Entity as CornerEntity, Model as Corner};
use crate::entities::episodes::{Entity as EpisodeEntity, Model as Episode};
use crate::entities::mails::{self, Entity as MailEntity, Model as Mail};
use crate::entities::podcasts::{self, Entity as PodcastEntity, Model as Podcast};
use crate::entities::scripts::{self, Entity as ScriptEntity, Model as Script};
use crate::entities::sea_orm_active_enums::TaskStatus;
use crate::entities::tasks::{self, Entity as TaskEntity, Model as Task};
use crate::entities::users::{self, Entity as UserEntity, Model as User};
use crate::repo::{Secret, UserRepo};
use crate::{
    error::Error,
    id::{CornerId, EpisodeId, MailId, PodcastId, ScriptId, TaskId},
    repo::{CornerRepo, EpisodeRepo, MailRepo, PodcastRepo, ScriptRepo, SecretRepo, TaskRepo},
};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, DatabaseBackend, DatabaseConnection, DbErr,
    EntityTrait, IntoActiveModel, ModelTrait, QueryFilter, QueryOrder, QuerySelect, RuntimeErr,
    Statement,
};
use sqlx::FromRow;
use uuid::Uuid;

pub struct PostgresUserRepo {
    db: DatabaseConnection,
}

impl PostgresUserRepo {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait]
impl UserRepo for PostgresUserRepo {
    async fn find_by_id(&self, id: &Uuid) -> anyhow::Result<User, Error> {
        let Some(user) = UserEntity::find_by_id(*id)
            .one(&self.db)
            .await
            .map_err(Error::Other)?
        else {
            return Err(Error::NotFound("user".to_string(), id.to_string()));
        };
        Ok(user)
    }

    async fn find_by_auth_id(&self, auth_id: &Uuid) -> anyhow::Result<User, Error> {
        let Some(podcast) = UserEntity::find()
            .filter(users::Column::AuthId.eq(auth_id.to_string()))
            .one(&self.db)
            .await
            .map_err(Error::Other)?
        else {
            return Err(Error::NotFound("user".to_string(), auth_id.to_string()));
        };
        Ok(podcast)
    }
}

pub struct PostgresPodcastRepo {
    db: DatabaseConnection,
}

impl PostgresPodcastRepo {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait]
impl PodcastRepo for PostgresPodcastRepo {
    async fn list(&self, user_id: &Uuid) -> anyhow::Result<Vec<(Podcast, Option<User>)>, Error> {
        let podcasts = PodcastEntity::find()
            .filter(podcasts::Column::UserId.eq(*user_id))
            .find_also_related(UserEntity)
            .all(&self.db)
            .await
            .map_err(Error::Other)?;
        Ok(podcasts)
    }

    async fn latests(&self, limit: u64) -> anyhow::Result<Vec<(Podcast, Option<User>)>, Error> {
        let podcasts = PodcastEntity::find()
            .order_by_desc(podcasts::Column::CreatedAt)
            .limit(limit)
            .find_also_related(UserEntity)
            .all(&self.db)
            .await
            .map_err(Error::Other)?;
        Ok(podcasts)
    }

    async fn find_by_id(&self, id: &PodcastId) -> anyhow::Result<Podcast, Error> {
        let Some(podcast) = PodcastEntity::find_by_id(id.0)
            .one(&self.db)
            .await
            .map_err(Error::Other)?
        else {
            return Err(Error::NotFound("podcast".to_string(), id.0.to_string()));
        };
        Ok(podcast)
    }

    async fn create(&self, podcast: Podcast) -> anyhow::Result<(), Error> {
        PodcastEntity::insert(podcast.into_active_model())
            .exec(&self.db)
            .await
            .map_err(Error::Other)?;
        Ok(())
    }

    async fn update(&self, podcast: Podcast) -> anyhow::Result<(), Error> {
        podcast
            .into_active_model()
            .save(&self.db)
            .await
            .map_err(Error::Other)?;
        Ok(())
    }

    async fn delete(&self, id: &PodcastId) -> anyhow::Result<(), Error> {
        let podcast = PodcastEntity::find_by_id(id.0)
            .one(&self.db)
            .await
            .map_err(Error::Other)?
            .ok_or_else(|| Error::NotFound("podcast".to_string(), id.0.to_string()))?;
        podcast.delete(&self.db).await.map_err(Error::Other)?;
        Ok(())
    }
}

pub struct PostgresEpisodeRepo {
    db: DatabaseConnection,
}

impl PostgresEpisodeRepo {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait]
impl EpisodeRepo for PostgresEpisodeRepo {
    async fn find_by_id(&self, id: &EpisodeId) -> anyhow::Result<Episode, Error> {
        EpisodeEntity::find_by_id(id.0)
            .one(&self.db)
            .await
            .map_err(Error::Other)?
            .ok_or_else(|| Error::NotFound("episode".to_string(), id.0.to_string()))
    }

    async fn find_all_by_podcast_id(
        &self,
        podcast_id: &PodcastId,
    ) -> anyhow::Result<Vec<Episode>, Error> {
        let podcast = PodcastEntity::find_by_id(podcast_id.0)
            .one(&self.db)
            .await
            .map_err(Error::Other)?
            .ok_or_else(|| Error::NotFound("podcast".to_string(), podcast_id.0.to_string()))?;
        podcast
            .find_related(EpisodeEntity)
            .all(&self.db)
            .await
            .map_err(Error::Other)
    }

    async fn create(&self, episode: Episode) -> anyhow::Result<(), Error> {
        EpisodeEntity::insert(episode.into_active_model())
            .exec(&self.db)
            .await
            .map_err(Error::Other)?;
        Ok(())
    }

    async fn update(&self, episode: Episode) -> anyhow::Result<(), Error> {
        episode
            .into_active_model()
            .save(&self.db)
            .await
            .map_err(Error::Other)?;
        Ok(())
    }

    async fn delete(&self, id: &EpisodeId) -> anyhow::Result<(), Error> {
        let episode = EpisodeEntity::find_by_id(id.0)
            .one(&self.db)
            .await
            .map_err(Error::Other)?
            .ok_or_else(|| Error::NotFound("episode".to_string(), id.0.to_string()))?;
        episode.delete(&self.db).await.map_err(Error::Other)?;
        Ok(())
    }
}

pub struct PostgresScriptRepo {
    db: DatabaseConnection,
}

impl PostgresScriptRepo {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait]
impl ScriptRepo for PostgresScriptRepo {
    async fn list(&self, user_id: &Uuid) -> anyhow::Result<Vec<Script>, Error> {
        let scripts = ScriptEntity::find()
            .filter(scripts::Column::UserId.eq(*user_id))
            .all(&self.db)
            .await
            .map_err(Error::Other)?;
        Ok(scripts)
    }

    async fn find_by_id(&self, id: &ScriptId) -> anyhow::Result<Script, Error> {
        ScriptEntity::find_by_id(id.0)
            .one(&self.db)
            .await
            .map_err(Error::Other)?
            .ok_or_else(|| Error::NotFound("script".to_string(), id.0.to_string()))
    }

    async fn create(&self, script: Script) -> anyhow::Result<(), Error> {
        ScriptEntity::insert(script.into_active_model())
            .exec(&self.db)
            .await
            .map_err(Error::Other)?;
        Ok(())
    }

    async fn update(&self, script: Script) -> anyhow::Result<(), Error> {
        script
            .into_active_model()
            .save(&self.db)
            .await
            .map_err(Error::Other)?;
        Ok(())
    }

    async fn delete(&self, id: &ScriptId) -> anyhow::Result<(), Error> {
        let script = ScriptEntity::find_by_id(id.0)
            .one(&self.db)
            .await
            .map_err(Error::Other)?
            .ok_or_else(|| Error::NotFound("script".to_string(), id.0.to_string()))?;
        script.delete(&self.db).await.map_err(Error::Other)?;
        Ok(())
    }
}

pub struct PostgresCornerRepo {
    db: DatabaseConnection,
}

impl PostgresCornerRepo {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait]
impl CornerRepo for PostgresCornerRepo {
    async fn find_all_by_podcast_id(
        &self,
        podcast_id: &PodcastId,
    ) -> anyhow::Result<Vec<(Corner, Option<User>)>, Error> {
        CornerEntity::find()
            .filter(corners::Column::PodcastId.eq(podcast_id.0))
            .find_also_related(UserEntity)
            .all(&self.db)
            .await
            .map_err(Error::Other)
    }

    async fn find_by_id(&self, id: &CornerId) -> anyhow::Result<(Corner, Option<User>), Error> {
        CornerEntity::find_by_id(id.0)
            .find_also_related(UserEntity)
            .one(&self.db)
            .await
            .map_err(Error::Other)?
            .ok_or_else(|| Error::NotFound("corner".to_string(), id.0.to_string()))
    }

    async fn create(&self, corner: Corner) -> anyhow::Result<(), Error> {
        CornerEntity::insert(corner.into_active_model())
            .exec(&self.db)
            .await
            .map_err(Error::Other)?;
        Ok(())
    }

    async fn update(&self, corner: Corner) -> anyhow::Result<(), Error> {
        corner
            .into_active_model()
            .save(&self.db)
            .await
            .map_err(Error::Other)?;
        Ok(())
    }

    async fn delete(&self, id: &CornerId) -> anyhow::Result<(), Error> {
        let corner = CornerEntity::find_by_id(id.0)
            .one(&self.db)
            .await
            .map_err(Error::Other)?
            .ok_or_else(|| Error::NotFound("corner".to_string(), id.0.to_string()))?;
        corner.delete(&self.db).await.map_err(Error::Other)?;
        Ok(())
    }
}

pub struct PostgresMailRepo {
    db: DatabaseConnection,
}

impl PostgresMailRepo {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait]
impl MailRepo for PostgresMailRepo {
    async fn list(&self, corner_id: &CornerId) -> anyhow::Result<Vec<(Mail, Option<User>)>, Error> {
        MailEntity::find()
            .filter(mails::Column::CornerId.eq(corner_id.0))
            .find_also_related(UserEntity)
            .all(&self.db)
            .await
            .map_err(Error::Other)
    }

    async fn find_by_id(&self, id: &MailId) -> anyhow::Result<(Mail, Option<User>), Error> {
        let mail = MailEntity::find_by_id(id.0)
            .find_also_related(UserEntity)
            .one(&self.db)
            .await
            .map_err(Error::Other)?
            .ok_or_else(|| Error::NotFound("mail".to_string(), id.0.to_string()))?;
        Ok(mail)
    }

    async fn create(&self, mail: Mail) -> anyhow::Result<(), Error> {
        MailEntity::insert(mail.into_active_model())
            .exec(&self.db)
            .await
            .map_err(Error::Other)?;
        Ok(())
    }

    async fn update(&self, mail: Mail) -> anyhow::Result<(), Error> {
        mail.into_active_model()
            .save(&self.db)
            .await
            .map_err(Error::Other)?;
        Ok(())
    }

    async fn delete(&self, id: &MailId) -> anyhow::Result<(), Error> {
        let mail = MailEntity::find_by_id(id.0)
            .one(&self.db)
            .await
            .map_err(Error::Other)?
            .ok_or_else(|| Error::NotFound("mail".to_string(), id.0.to_string()))?;
        mail.delete(&self.db).await.map_err(Error::Other)?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct PostgresTaskRepo {
    db: DatabaseConnection,
}

impl PostgresTaskRepo {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait]
impl TaskRepo for PostgresTaskRepo {
    async fn list(&self, id: &Uuid) -> anyhow::Result<Vec<Task>, Error> {
        let tasks = TaskEntity::find()
            .filter(tasks::Column::UserId.eq(*id))
            .all(&self.db)
            .await
            .map_err(Error::Other)?;
        Ok(tasks)
    }

    async fn find_by_id(&self, id: &TaskId) -> anyhow::Result<Task, Error> {
        let task = TaskEntity::find_by_id(id.0)
            .one(&self.db)
            .await
            .map_err(Error::Other)?
            .ok_or_else(|| Error::NotFound("task".to_string(), id.0.to_string()))?;
        Ok(task)
    }

    async fn pop(&self, now: DateTime<Utc>) -> anyhow::Result<Option<Task>, Error> {
        let task = TaskEntity::find()
            .filter(tasks::Column::Status.eq(TaskStatus::Pending))
            .filter(tasks::Column::ExecuteAfter.lt(now))
            .order_by_asc(tasks::Column::ExecuteAfter)
            .limit(1)
            .one(&self.db)
            .await
            .map_err(Error::Other)?;
        Ok(task)
    }

    async fn create(&self, task: Task) -> anyhow::Result<(), Error> {
        TaskEntity::insert(task.into_active_model())
            .exec(&self.db)
            .await
            .map_err(Error::Other)?;
        Ok(())
    }

    async fn update(&self, task: Task) -> anyhow::Result<(), Error> {
        task.into_active_model()
            .save(&self.db)
            .await
            .map_err(Error::Other)?;
        Ok(())
    }

    async fn delete(&self, id: &TaskId) -> anyhow::Result<(), Error> {
        let task = TaskEntity::find_by_id(id.0)
            .one(&self.db)
            .await
            .map_err(Error::Other)?
            .ok_or_else(|| Error::NotFound("task".to_string(), id.0.to_string()))?;
        task.delete(&self.db).await.map_err(Error::Other)?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct PostgresSecretRepo {
    db: DatabaseConnection,
}

impl PostgresSecretRepo {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait]
impl SecretRepo for PostgresSecretRepo {
    async fn list(&self, user_id: &Uuid) -> anyhow::Result<Vec<Secret>, Error> {
        let secrets = self
            .db
            .query_all(Statement::from_sql_and_values(
                DatabaseBackend::Postgres,
                "select * from vault.decrypted_secrets where name like $1",
                vec![format!("{}:%", user_id).into()],
            ))
            .await
            .map_err(Error::Other)?;
        let secrets = secrets
            .iter()
            .map(|secret| Secret::from_row(secret.try_as_pg_row().unwrap()))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| Error::Other(DbErr::Query(RuntimeErr::SqlxError(e))))?;
        Ok(secrets)
    }

    async fn find_by_name(&self, user_id: &Uuid, name: &str) -> anyhow::Result<Secret, Error> {
        // NOTE: `name` is globally unique for users, so prefixed `user_uuid`
        let secret = self
            .db
            .query_one(Statement::from_sql_and_values(
                DatabaseBackend::Postgres,
                "select * from vault.decrypted_secrets where name = $1",
                vec![format!("{}:{}", user_id, name).into()],
            ))
            .await
            .map_err(Error::Other)?
            .ok_or_else(|| Error::NotFound("secret".to_string(), name.to_string()))?;
        let row = secret.try_as_pg_row().unwrap();
        let secret = Secret::from_row(row)
            .map_err(|e| Error::Other(DbErr::Query(RuntimeErr::SqlxError(e))))?;
        Ok(secret)
    }

    async fn create(
        &self,
        user_id: &Uuid,
        secret: &String,
        name: &String,
    ) -> anyhow::Result<(), Error> {
        self.db
            .execute(Statement::from_sql_and_values(
                DatabaseBackend::Postgres,
                "select vault.create_secret($1, $2)",
                vec![secret.into(), format!("{}:{}", user_id, name).into()],
            ))
            .await
            .map_err(Error::Other)?;
        Ok(())
    }

    async fn update(
        &self,
        user_id: &Uuid,
        id: &Uuid,
        secret: &String,
        name: &String,
    ) -> anyhow::Result<(), Error> {
        self.db
            .execute(Statement::from_sql_and_values(
                DatabaseBackend::Postgres,
                "select vault.update_secret($1, $2, $3)",
                vec![
                    id.hyphenated().to_string().into(),
                    secret.into(),
                    format!("{}:{}", user_id, name).into(),
                ],
            ))
            .await
            .map_err(Error::Other)?;
        Ok(())
    }

    async fn delete(&self, user_id: &Uuid, id: &Uuid) -> anyhow::Result<(), Error> {
        self.db
            .execute(Statement::from_sql_and_values(
                DatabaseBackend::Postgres,
                "delete from vault.secrets where name like $1 and id = $2::uuid",
                vec![format!("{}:%", user_id).into(), id.to_string().into()],
            ))
            .await
            .map_err(Error::Other)?;
        Ok(())
    }
}

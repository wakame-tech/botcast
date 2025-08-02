use super::ApiImpl;
use anyhow::Result;
use async_trait::async_trait;
use axum::http::Method;
use axum_extra::extract::{CookieJar, Host};
use openapi::{
    apis::episodes::*,
    models::{self, *},
};
use repos::{
    entities::{episodes::Model as Episode, users::Model as User},
    id::EpisodeId,
    repo::EpisodeRepo,
    storage::Storage,
};
use std::time::Duration;
use uuid::Uuid;

pub(crate) fn into_episode_model(
    episode: Episode,
    audio_presign_url: Option<String>,
    srt_presign_url: Option<String>,
) -> Result<models::Episode> {
    Ok(models::Episode {
        id: episode.id,
        title: episode.title,
        sections: serde_json::from_value(episode.sections)?,
        description: episode.description.unwrap(),
        audio_url: audio_presign_url,
        srt_url: srt_presign_url,
        created_at: episode.created_at.into(),
        duration_sec: episode.duration_sec.map(|d| d as f64),
    })
}

#[async_trait]
impl Episodes<anyhow::Error> for ApiImpl {
    type Claims = User;

    async fn episodes_post(
        &self,
        _method: &Method,
        _host: &Host,
        _cookie: &CookieJar,
        user: &Self::Claims,
        body: &EpisodesPostRequest,
    ) -> Result<EpisodesPostResponse> {
        self.episode_repo
            .create(Episode {
                id: Uuid::new_v4(),
                title: body.title.clone(),
                audio_url: None,
                user_id: Some(user.id),
                podcast_id: body.podcast_id.parse().unwrap(),
                srt_url: None,
                created_at: chrono::Utc::now().into(),
                sections: serde_json::to_value(body.sections.clone().unwrap_or_default())?,
                description: body.description.clone(),
                duration_sec: None,
            })
            .await?;
        Ok(EpisodesPostResponse::Status200_OK)
    }

    async fn episodes_episode_id_delete(
        &self,
        _method: &Method,
        _host: &Host,
        _cookie: &CookieJar,
        _claims: &Self::Claims,
        path_params: &EpisodesEpisodeIdDeletePathParams,
    ) -> Result<EpisodesEpisodeIdDeleteResponse> {
        self.episode_repo
            .delete(&EpisodeId(path_params.episode_id.parse().unwrap()))
            .await?;
        Ok(EpisodesEpisodeIdDeleteResponse::Status200_OK)
    }

    async fn episodes_episode_id_get(
        &self,
        _method: &Method,
        _host: &Host,
        _cookie: &CookieJar,
        _claims: &Self::Claims,
        _path_params: &EpisodesEpisodeIdGetPathParams,
    ) -> Result<EpisodesEpisodeIdGetResponse> {
        let episode = self
            .episode_repo
            .find_by_id(&EpisodeId(_path_params.episode_id.parse().unwrap()))
            .await?;
        let audio_presign_url = if let Some(audio_url) = &episode.audio_url {
            let audio_presign_url = self
                .storage
                .get_presign_url(&audio_url, Duration::from_secs(3600))
                .await?;
            Some(audio_presign_url)
        } else {
            None
        };
        let srt_presign_url = if let Some(srt_url) = &episode.srt_url {
            let srt_presign_url = self
                .storage
                .get_presign_url(&srt_url, Duration::from_secs(3600))
                .await?;
            Some(srt_presign_url)
        } else {
            None
        };

        Ok(EpisodesEpisodeIdGetResponse::Status200_OK(
            into_episode_model(episode, audio_presign_url, srt_presign_url)?,
        ))
    }

    async fn episodes_episode_id_put(
        &self,
        _method: &Method,
        _host: &Host,
        _cookies: &CookieJar,
        _claims: &Self::Claims,
        path_params: &EpisodesEpisodeIdPutPathParams,
        body: &EpisodesEpisodeIdPutRequest,
    ) -> Result<EpisodesEpisodeIdPutResponse> {
        let episode = self
            .episode_repo
            .find_by_id(&EpisodeId(path_params.episode_id.parse().unwrap()))
            .await?;
        self.episode_repo
            .create(Episode {
                id: Uuid::new_v4(),
                title: body.title.clone(),
                audio_url: None,
                user_id: episode.user_id,
                podcast_id: episode.podcast_id,
                srt_url: episode.srt_url,
                created_at: chrono::Utc::now().into(),
                sections: serde_json::to_value(body.sections.clone().unwrap_or_default())?,
                description: body.description.clone(),
                duration_sec: None,
            })
            .await?;
        Ok(EpisodesEpisodeIdPutResponse::Status200_OK)
    }
}

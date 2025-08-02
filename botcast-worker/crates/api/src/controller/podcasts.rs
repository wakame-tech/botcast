use super::{ApiImpl, corners::into_corner_model, episodes::into_episode_model};
use anyhow::Result;
use async_trait::async_trait;
use axum::http::Method;
use axum_extra::extract::{CookieJar, Host};
use openapi::{
    apis::podcasts::*,
    models::{self, *},
};
use repos::{
    entities::{podcasts::Model as Podcast, users::Model as User},
    id::PodcastId,
    repo::{CornerRepo, EpisodeRepo, PodcastRepo, UserRepo},
};
use uuid::Uuid;

fn into_podcast_model(podcast: (Podcast, Option<User>)) -> models::Podcast {
    let (podcast, user) = podcast;
    models::Podcast {
        id: podcast.id,
        title: podcast.title,
        icon: podcast.icon,
        created_at: podcast.created_at.into(),
        description: podcast.description.unwrap(),
        user: user
            .map(|user| models::User {
                id: user.id,
                auth_id: user.auth_id.parse().unwrap(),
                email: user.email,
                name: user.name.unwrap(),
            })
            .unwrap(),
    }
}

#[async_trait]
impl Podcasts<anyhow::Error> for ApiImpl {
    type Claims = User;

    async fn podcast_podcast_id_delete(
        &self,
        _method: &Method,
        _host: &Host,
        _cookie: &CookieJar,
        _claims: &Self::Claims,
        path_params: &PodcastPodcastIdDeletePathParams,
    ) -> Result<PodcastPodcastIdDeleteResponse> {
        self.podcast_repo
            .delete(&PodcastId(path_params.podcast_id.parse()?))
            .await?;
        Ok(PodcastPodcastIdDeleteResponse::Status200_OK)
    }

    async fn podcast_podcast_id_get(
        &self,
        _method: &Method,
        _host: &Host,
        _cookie: &CookieJar,
        _claims: &Self::Claims,
        _path_params: &PodcastPodcastIdGetPathParams,
    ) -> Result<PodcastPodcastIdGetResponse> {
        let podcast_id = PodcastId(_path_params.podcast_id.parse()?);
        let podcast = self.podcast_repo.find_by_id(&podcast_id).await?;
        let episodes = self
            .episode_repo
            .find_all_by_podcast_id(&podcast_id)
            .await?
            .into_iter()
            .map(|episode| into_episode_model(episode, None, None))
            .collect::<Result<Vec<_>>>()?;
        let corners = self
            .corner_repo
            .find_all_by_podcast_id(&podcast_id)
            .await?
            .into_iter()
            .map(into_corner_model)
            .collect::<Result<Vec<_>>>()?;
        let user = self.user_repo.find_by_id(&podcast.user_id.unwrap()).await?;
        Ok(PodcastPodcastIdGetResponse::Status200_OK(
            PodcastPodcastIdGet200Response {
                podcast: into_podcast_model((podcast, Some(user))),
                episodes,
                corners,
            },
        ))
    }

    async fn podcast_podcast_id_put(
        &self,
        _method: &Method,
        _host: &Host,
        _cookies: &CookieJar,
        user: &Self::Claims,
        _path_params: &PodcastPodcastIdPutPathParams,
        body: &PodcastsPostRequest,
    ) -> Result<PodcastPodcastIdPutResponse> {
        self.podcast_repo
            .create(Podcast {
                id: Uuid::new_v4(),
                title: body.title.clone(),
                description: Some(body.description.clone()),
                icon: body.icon.clone(),
                user_id: Some(user.id),
                created_at: chrono::Utc::now().into(),
            })
            .await?;
        Ok(PodcastPodcastIdPutResponse::Status200_OK)
    }

    async fn podcasts_get(
        &self,
        _method: &Method,
        _host: &Host,
        _cookies: &CookieJar,
        user: &Self::Claims,
    ) -> Result<PodcastsGetResponse> {
        let podcasts = self.podcast_repo.list(&user.id).await?;
        Ok(PodcastsGetResponse::Status200_OK(
            podcasts.into_iter().map(into_podcast_model).collect(),
        ))
    }

    async fn podcasts_post(
        &self,
        _method: &Method,
        _host: &Host,
        _cookies: &CookieJar,
        user: &Self::Claims,
        body: &PodcastsPostRequest,
    ) -> Result<PodcastsPostResponse> {
        self.podcast_repo
            .create(Podcast {
                id: Uuid::new_v4(),
                title: body.title.clone(),
                description: Some(body.description.clone()),
                icon: body.icon.clone(),
                user_id: Some(user.id),
                created_at: chrono::Utc::now().into(),
            })
            .await?;
        Ok(PodcastsPostResponse::Status200_OK)
    }

    async fn top_podcasts_get(
        &self,
        _method: &Method,
        _host: &Host,
        _cookies: &CookieJar,
        _claims: &Self::Claims,
    ) -> Result<TopPodcastsGetResponse> {
        let podcasts = self.podcast_repo.latests(10).await?;
        Ok(TopPodcastsGetResponse::Status200_OK(
            podcasts.into_iter().map(into_podcast_model).collect(),
        ))
    }
}

use super::{as_string, evaluate_args, Plugin};
use anyhow::Result;
use json_e::{
    value::{AsyncCallable, Function, Value},
    Context,
};
use openapi_client::{
    apis::{
        auth_api::me_get,
        configuration::Configuration,
        episodes_api::{episodes_episode_id_get, episodes_post},
        podcasts_api::podcast_podcast_id_get,
        scripts_api::scripts_script_id_get,
    },
    models::EpisodesPostRequest,
};
use openapi_client::{
    apis::{episodes_api::episodes_episode_id_put, mails_api::corners_corner_id_mails_get},
    models::{EpisodesEpisodeIdPutRequest, Section},
};
use tracing::instrument;

#[derive(Clone)]
struct Me(Configuration);

#[async_trait::async_trait]
impl AsyncCallable for Me {
    #[instrument(skip(self))]
    async fn call(&self, _: &Context<'_>, _: &[Value]) -> Result<Value> {
        let me = me_get(&self.0).await?;
        let me = serde_json::to_value(me)?;
        Ok(me.into())
    }
}

#[derive(Clone)]
struct GetPodcast(Configuration);

#[async_trait::async_trait]
impl AsyncCallable for GetPodcast {
    #[instrument(skip(self, ctx))]
    async fn call(&self, ctx: &Context<'_>, args: &[Value]) -> Result<Value> {
        let evaluated = evaluate_args(ctx, args).await?;
        let id = as_string(&evaluated[0])?;
        let podcast = podcast_podcast_id_get(&self.0, &id).await?;
        let podcast = serde_json::to_value(podcast)?;
        Ok(podcast.into())
    }
}

#[derive(Clone)]
struct GetEpisode(Configuration);

#[async_trait::async_trait]
impl AsyncCallable for GetEpisode {
    #[instrument(skip(self, ctx))]
    async fn call(&self, ctx: &Context<'_>, args: &[Value]) -> Result<Value> {
        let evaluated = evaluate_args(ctx, args).await?;
        let id = as_string(&evaluated[0])?;
        let episode = episodes_episode_id_get(&self.0, &id).await?;
        let episode = serde_json::to_value(episode)?;
        Ok(episode.into())
    }
}

#[derive(Clone)]
struct GetScript(Configuration);

#[async_trait::async_trait]
impl AsyncCallable for GetScript {
    #[instrument(skip(self, ctx))]
    async fn call(&self, ctx: &Context<'_>, args: &[Value]) -> Result<Value> {
        let evaluated = evaluate_args(ctx, args).await?;
        let id = as_string(&evaluated[0])?;
        let res = scripts_script_id_get(&self.0, &id).await?;
        let res = serde_json::to_value(res)?;
        Ok(res.into())
    }
}

#[derive(Clone)]
struct NewEpisode(Configuration);

#[async_trait::async_trait]
impl AsyncCallable for NewEpisode {
    #[instrument(skip(self, ctx))]
    async fn call(&self, ctx: &Context<'_>, args: &[Value]) -> Result<Value> {
        let evaluated = evaluate_args(ctx, args).await?;
        let podcast_id = as_string(&evaluated[0])?;
        let title = as_string(&evaluated[1])?;
        let sections: Vec<Section> = serde_json::from_value(evaluated[2].clone())?;
        let description = evaluated.get(3).map(as_string).transpose()?;
        episodes_post(
            &self.0,
            EpisodesPostRequest {
                podcast_id: podcast_id.clone(),
                title: title.clone(),
                description,
                sections: Some(sections),
            },
        )
        .await?;
        Ok(Value::Null)
    }
}

#[derive(Clone)]
struct UpdateEpisode(Configuration);

#[async_trait::async_trait]
impl AsyncCallable for UpdateEpisode {
    #[instrument(skip(self, ctx))]
    async fn call(&self, ctx: &Context<'_>, args: &[Value]) -> Result<Value> {
        let args = evaluate_args(ctx, args).await?;
        let episode_id = as_string(&args[0])?;
        let title = as_string(&args[1])?;
        let sections: Option<Vec<Section>> =
            (!args[2].is_null()).then_some(serde_json::from_value(args[2].clone())?);
        let description = (!args[3].is_null()).then_some(as_string(&args[3])?);

        episodes_episode_id_put(
            &self.0,
            &episode_id,
            EpisodesEpisodeIdPutRequest {
                title: title.clone(),
                description,
                sections,
            },
        )
        .await?;
        Ok(Value::Null)
    }
}

#[derive(Clone)]
struct GetPodcastMails(Configuration);

#[async_trait::async_trait]
impl AsyncCallable for GetPodcastMails {
    #[instrument(skip(self, ctx))]
    async fn call(&self, ctx: &Context<'_>, args: &[Value]) -> Result<Value> {
        let args = evaluate_args(ctx, args).await?;
        let corner_id = as_string(&args[0])?;
        let mails = corners_corner_id_mails_get(&self.0, &corner_id).await?;
        Ok(serde_json::to_value(mails)?.into())
    }
}

pub struct BotCastApiPlugin {
    configuration: Configuration,
}

impl BotCastApiPlugin {
    pub fn new(configuration: Configuration) -> Self {
        Self { configuration }
    }
}

impl Plugin for BotCastApiPlugin {
    fn register_functions(&self, context: &mut Context<'_>) {
        let functions = vec![
            (
                "me",
                Box::new(Me(self.configuration.clone())) as Box<dyn AsyncCallable>,
            ),
            (
                "get_podcast",
                Box::new(GetPodcast(self.configuration.clone())),
            ),
            (
                "get_episode",
                Box::new(GetEpisode(self.configuration.clone())),
            ),
            (
                "get_script",
                Box::new(GetScript(self.configuration.clone())),
            ),
            (
                "new_episode",
                Box::new(NewEpisode(self.configuration.clone())),
            ),
            (
                "update_episode",
                Box::new(UpdateEpisode(self.configuration.clone())),
            ),
            (
                "get_podcast_mails",
                Box::new(GetPodcastMails(self.configuration.clone())),
            ),
        ];
        for (name, func) in functions {
            context.insert(name, Value::Function(Function::new(name, func)));
        }
    }
}

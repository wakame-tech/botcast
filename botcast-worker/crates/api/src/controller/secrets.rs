use super::ApiImpl;
use anyhow::Result;
use async_trait::async_trait;
use axum::http::Method;
use axum_extra::extract::{CookieJar, Host};
use openapi::{apis::secrets::*, models::*};
use repos::entities::users::Model as User;
use repos::repo::SecretRepo;

#[async_trait]
impl Secrets<anyhow::Error> for ApiImpl {
    type Claims = User;

    async fn secrets_get(
        &self,
        _method: &Method,
        _host: &Host,
        _cookies: &CookieJar,
        user: &Self::Claims,
    ) -> Result<SecretsGetResponse> {
        let secrets = self
            .secret_repo
            .list(&user.id)
            .await?
            .iter()
            .map(|s| Secret {
                id: s.id,
                name: s.name.split_terminator(':').nth(1).unwrap().to_string(),
            })
            .collect::<Vec<_>>();
        Ok(SecretsGetResponse::Status200_OK(secrets))
    }

    async fn secrets_post(
        &self,
        _method: &Method,
        _host: &Host,
        _cookies: &CookieJar,
        user: &Self::Claims,
        body: &SecretsPostRequest,
    ) -> Result<SecretsPostResponse> {
        // TODO: use transaction
        if let Some(news) = &body.news {
            for secret in news {
                self.secret_repo
                    .create(&user.id, &secret.value, &secret.name)
                    .await?;
            }
        }
        if let Some(deletion_ids) = &body.deletion_ids {
            for id in deletion_ids {
                self.secret_repo
                    .delete(&user.id, &id.parse().unwrap())
                    .await?;
            }
        }

        Ok(SecretsPostResponse::Status200_OK)
    }
}

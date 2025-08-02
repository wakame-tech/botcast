use super::auth::into_user_model;
use super::corners::into_corner_model;
use super::{ApiImpl, into_openapi_object};
use anyhow::Result;
use async_trait::async_trait;
use axum::http::Method;
use axum_extra::extract::{CookieJar, Host};
use openapi::models;
use openapi::{apis::mails::*, models::*};
use repos::entities::mails::Model as Mail;
use repos::entities::users::Model as User;
use repos::id::CornerId;
use repos::repo::{CornerRepo, MailRepo};
use uuid::Uuid;

pub(crate) fn into_mail_model(
    mail: Mail,
    user: models::User,
    corner: models::Corner,
) -> Result<models::Mail> {
    Ok(models::Mail {
        id: mail.id,
        body: into_openapi_object(mail.body)?,
        user,
        corner,
        created_at: mail.created_at.into(),
    })
}

#[async_trait]
impl Mails<anyhow::Error> for ApiImpl {
    type Claims = User;

    async fn corners_corner_id_mails_get(
        &self,
        _method: &Method,
        _host: &Host,
        _cookies: &CookieJar,
        _claims: &Self::Claims,
        path_params: &CornersCornerIdMailsGetPathParams,
    ) -> Result<CornersCornerIdMailsGetResponse> {
        let (corner, _) = self
            .corner_repo
            .find_by_id(&CornerId(path_params.corner_id.parse().unwrap()))
            .await?;
        let mails = self
            .mail_repo
            .list(&CornerId(path_params.corner_id.parse().unwrap()))
            .await?;
        let mails = mails
            .into_iter()
            .map(move |(mail, user)| {
                let user = user.unwrap();
                into_mail_model(
                    mail,
                    into_user_model(user.clone())?,
                    into_corner_model((corner.clone(), Some(user)))?,
                )
            })
            .collect::<Result<Vec<_>>>()?;
        Ok(CornersCornerIdMailsGetResponse::Status200_OK(mails))
    }

    async fn corners_corner_id_mails_post(
        &self,
        _method: &Method,
        _host: &Host,
        _cookies: &CookieJar,
        user: &Self::Claims,
        path_params: &models::CornersCornerIdMailsPostPathParams,
        body: &models::CornersCornerIdMailsPostRequest,
    ) -> Result<CornersCornerIdMailsPostResponse> {
        self.mail_repo
            .create(Mail {
                id: Uuid::new_v4(),
                body: serde_json::to_value(body.body.clone())
                    .map_err(|_| anyhow::anyhow!("Failed to serialize body"))?,
                user_id: user.id,
                corner_id: path_params.corner_id.parse().unwrap(),
                created_at: chrono::Utc::now().into(),
            })
            .await?;
        Ok(CornersCornerIdMailsPostResponse::Status200_OK)
    }

    async fn corners_corner_id_mails_mail_id_delete(
        &self,
        _method: &Method,
        _host: &Host,
        _cookie: &CookieJar,
        _claims: &Self::Claims,
        _path_params: &CornersCornerIdMailsMailIdDeletePathParams,
    ) -> Result<CornersCornerIdMailsMailIdDeleteResponse> {
        self.mail_repo
            .delete(&repos::id::MailId(_path_params.mail_id.parse().unwrap()))
            .await?;
        Ok(CornersCornerIdMailsMailIdDeleteResponse::Status200_OK)
    }
}

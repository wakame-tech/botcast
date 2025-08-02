use super::{ApiImpl, auth::into_user_model, into_openapi_object};
use anyhow::Result;
use async_trait::async_trait;
use axum::http::Method;
use axum_extra::extract::{CookieJar, Host};
use openapi::{
    apis::corners::*,
    models::{self, *},
};
use repos::{
    entities::{corners::Model as Corner, users::Model as User},
    id::CornerId,
    repo::CornerRepo,
};

pub(crate) fn into_corner_model((corner, user): (Corner, Option<User>)) -> Result<models::Corner> {
    let user = user.unwrap();
    Ok(models::Corner {
        id: corner.id,
        title: corner.title,
        description: corner.description,
        requesting_mail: Some(corner.requesting_mail),
        mail_schema: Some(into_openapi_object(corner.mail_schema)?),
        user: into_user_model(user)?,
    })
}

#[async_trait]
impl Corners<anyhow::Error> for ApiImpl {
    type Claims = User;

    async fn corners_corner_id_delete(
        &self,
        _method: &Method,
        _host: &Host,
        _cookie: &CookieJar,
        _claims: &Self::Claims,
        _path_params: &CornersCornerIdDeletePathParams,
    ) -> Result<CornersCornerIdDeleteResponse> {
        self.corner_repo
            .delete(&CornerId(_path_params.corner_id.parse().unwrap()))
            .await?;
        Ok(CornersCornerIdDeleteResponse::Status200_OK)
    }

    async fn corners_corner_id_get(
        &self,
        _method: &Method,
        _host: &Host,
        _cookie: &CookieJar,
        _claims: &Self::Claims,
        _path_params: &CornersCornerIdGetPathParams,
    ) -> Result<CornersCornerIdGetResponse> {
        let (corner, user) = self
            .corner_repo
            .find_by_id(&CornerId(_path_params.corner_id.parse().unwrap()))
            .await?;
        Ok(CornersCornerIdGetResponse::Status200_OK(into_corner_model(
            (corner, user),
        )?))
    }

    async fn corners_corner_id_put(
        &self,
        _method: &Method,
        _host: &Host,
        _cookies: &CookieJar,
        _claims: &Self::Claims,
        _path_params: &CornersCornerIdPutPathParams,
        body: &CornersCornerIdPutRequest,
    ) -> Result<CornersCornerIdPutResponse> {
        let (corner, _) = self
            .corner_repo
            .find_by_id(&CornerId(_path_params.corner_id.parse().unwrap()))
            .await?;
        self.corner_repo
            .update(Corner {
                title: body.title.clone(),
                description: body.description.clone(),
                mail_schema: body
                    .mail_schema
                    .clone()
                    .map(|s| {
                        serde_json::to_value(s)
                            .map_err(|_| anyhow::anyhow!("Failed to serialize mail schema"))
                    })
                    .unwrap_or(Ok(corner.mail_schema))?,
                ..corner
            })
            .await?;

        Ok(CornersCornerIdPutResponse::Status200_OK)
    }
}

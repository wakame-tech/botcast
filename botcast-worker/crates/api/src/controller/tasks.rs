use super::{ApiImpl, into_openapi_object};
use anyhow::Result;
use async_trait::async_trait;
use axum::http::Method;
use axum_extra::extract::{CookieJar, Host};
use chrono::Utc;
use openapi::{
    apis::tasks::*,
    models::{self, *},
};
use repos::{
    entities::{sea_orm_active_enums::TaskStatus, tasks::Model as Task, users::Model as User},
    id::TaskId,
    repo::TaskRepo,
};
use uuid::Uuid;

fn into_task_status_model(task_status: TaskStatus) -> models::TaskStatus {
    match task_status {
        TaskStatus::Pending => models::TaskStatus::Pending,
        TaskStatus::Running => models::TaskStatus::Running,
        TaskStatus::Completed => models::TaskStatus::Completed,
        TaskStatus::Failed => models::TaskStatus::Failed,
    }
}

pub(crate) fn into_task_model(task: Task) -> Result<models::Task> {
    Ok(models::Task {
        id: task.id,
        status: into_task_status_model(task.status),
        args: into_openapi_object(task.args)?,
        user_id: task.user_id,
        execute_after: task.execute_after.into(),
        executed_at: task.executed_at.map(Into::into),
        executed_finished_at: task.executed_finished_at.map(Into::into),
        result: task.result.map(into_openapi_object).transpose()?,
        cron: task.cron,
    })
}

#[async_trait]
impl Tasks<anyhow::Error> for ApiImpl {
    type Claims = User;

    async fn tasks_get(
        &self,
        _method: &Method,
        _host: &Host,
        _cookies: &CookieJar,
        user: &Self::Claims,
    ) -> Result<TasksGetResponse> {
        let tasks = self.task_repo.list(&user.id).await?;
        let tasks = tasks
            .into_iter()
            .map(into_task_model)
            .collect::<Result<Vec<_>>>()?;
        Ok(TasksGetResponse::Status200_OK(tasks))
    }

    async fn tasks_post(
        &self,
        _method: &Method,
        _host: &Host,
        _cookies: &CookieJar,
        user: &Self::Claims,
        body: &models::TasksPostRequest,
    ) -> Result<TasksPostResponse> {
        let task = Task {
            id: Uuid::new_v4(),
            status: TaskStatus::Pending,
            args: serde_json::to_value(body.args.clone())?,
            user_id: Some(user.id),
            execute_after: Utc::now().into(),
            executed_at: None,
            executed_finished_at: None,
            result: None,
            cron: body.cron.clone(),
        };
        self.task_repo.create(task).await?;
        Ok(TasksPostResponse::Status200_OK)
    }

    async fn tasks_task_id_put(
        &self,
        _method: &Method,
        _host: &Host,
        _cookies: &CookieJar,
        _claims: &Self::Claims,
        path_params: &TasksTaskIdPutPathParams,
        body: &models::TasksTaskIdPutRequest,
    ) -> Result<TasksTaskIdPutResponse> {
        let task = self
            .task_repo
            .find_by_id(&TaskId(path_params.task_id))
            .await?;
        let task = Task {
            id: path_params.task_id,
            status: TaskStatus::Pending,
            args: serde_json::to_value(body.args.clone())?,
            user_id: None,
            execute_after: Utc::now().into(),
            executed_at: None,
            executed_finished_at: None,
            result: None,
            cron: task.cron.clone(),
        };
        self.task_repo.update(task).await?;
        Ok(TasksTaskIdPutResponse::Status200_OK)
    }

    async fn tasks_task_id_delete(
        &self,
        _method: &Method,
        _host: &Host,
        _cookies: &CookieJar,
        _claims: &Self::Claims,
        path_params: &TasksTaskIdDeletePathParams,
    ) -> Result<TasksTaskIdDeleteResponse> {
        self.task_repo.delete(&TaskId(path_params.task_id)).await?;
        Ok(TasksTaskIdDeleteResponse::Status200_OK)
    }
}

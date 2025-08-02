use sea_orm_migration::{prelude::*, schema::*};
use sea_orm_migration::prelude::extension::postgres::Type;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Create custom enum types
        manager
            .create_type(
                Type::create()
                    .as_enum(TaskStatus::Table)
                    .values([
                        TaskStatus::Pending,
                        TaskStatus::Running,
                        TaskStatus::Completed,
                        TaskStatus::Failed,
                    ])
                    .to_owned(),
            )
            .await?;

        // Create users table (no dependencies)
        manager
            .create_table(
                Table::create()
                    .table(Users::Table)
                    .if_not_exists()
                    .col(pk_uuid(Users::Id))
                    .col(text(Users::AuthId).unique_key())
                    .col(text(Users::Email).unique_key())
                    .col(text_null(Users::Name))
                    .to_owned(),
            )
            .await?;

        // Create podcasts table (depends on users)
        manager
            .create_table(
                Table::create()
                    .table(Podcasts::Table)
                    .if_not_exists()
                    .col(pk_uuid(Podcasts::Id))
                    .col(text(Podcasts::Title))
                    .col(uuid_null(Podcasts::UserId))
                    .col(string(Podcasts::Icon))
                    .col(timestamp_with_time_zone(Podcasts::CreatedAt))
                    .col(text_null(Podcasts::Description))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_podcasts_user_id")
                            .from(Podcasts::Table, Podcasts::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Create episodes table (depends on podcasts and users)
        manager
            .create_table(
                Table::create()
                    .table(Episodes::Table)
                    .if_not_exists()
                    .col(pk_uuid(Episodes::Id))
                    .col(text(Episodes::Title))
                    .col(text_null(Episodes::AudioUrl))
                    .col(uuid_null(Episodes::UserId))
                    .col(uuid(Episodes::PodcastId))
                    .col(text_null(Episodes::SrtUrl))
                    .col(timestamp_with_time_zone(Episodes::CreatedAt))
                    .col(json_binary(Episodes::Sections))
                    .col(text_null(Episodes::Description))
                    .col(integer_null(Episodes::DurationSec))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_episodes_user_id")
                            .from(Episodes::Table, Episodes::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_episodes_podcast_id")
                            .from(Episodes::Table, Episodes::PodcastId)
                            .to(Podcasts::Table, Podcasts::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Create corners table (depends on podcasts and users)
        manager
            .create_table(
                Table::create()
                    .table(Corners::Table)
                    .if_not_exists()
                    .col(pk_uuid(Corners::Id))
                    .col(text(Corners::Title))
                    .col(text(Corners::Description))
                    .col(boolean(Corners::RequestingMail))
                    .col(uuid(Corners::UserId))
                    .col(json_binary(Corners::MailSchema))
                    .col(uuid(Corners::PodcastId))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_corners_user_id")
                            .from(Corners::Table, Corners::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_corners_podcast_id")
                            .from(Corners::Table, Corners::PodcastId)
                            .to(Podcasts::Table, Podcasts::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Create mails table (depends on corners and users)
        manager
            .create_table(
                Table::create()
                    .table(Mails::Table)
                    .if_not_exists()
                    .col(pk_uuid(Mails::Id))
                    .col(json_binary(Mails::Body))
                    .col(uuid(Mails::UserId))
                    .col(uuid(Mails::CornerId))
                    .col(timestamp_with_time_zone(Mails::CreatedAt))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_mails_user_id")
                            .from(Mails::Table, Mails::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_mails_corner_id")
                            .from(Mails::Table, Mails::CornerId)
                            .to(Corners::Table, Corners::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Create scripts table (depends on users)
        manager
            .create_table(
                Table::create()
                    .table(Scripts::Table)
                    .if_not_exists()
                    .col(pk_uuid(Scripts::Id))
                    .col(json_binary(Scripts::Template))
                    .col(uuid(Scripts::UserId))
                    .col(text(Scripts::Title))
                    .col(text_null(Scripts::Description))
                    .col(json_binary(Scripts::Arguments))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_scripts_user_id")
                            .from(Scripts::Table, Scripts::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Create tasks table (depends on users)
        manager
            .create_table(
                Table::create()
                    .table(Tasks::Table)
                    .if_not_exists()
                    .col(pk_uuid(Tasks::Id))
                    .col(custom(Tasks::Status, Alias::new("task_status")))
                    .col(json_binary(Tasks::Args))
                    .col(uuid_null(Tasks::UserId))
                    .col(timestamp_with_time_zone(Tasks::ExecuteAfter))
                    .col(timestamp_with_time_zone_null(Tasks::ExecutedAt))
                    .col(timestamp_with_time_zone_null(Tasks::ExecutedFinishedAt))
                    .col(json_binary_null(Tasks::Result))
                    .col(text_null(Tasks::Cron))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_tasks_user_id")
                            .from(Tasks::Table, Tasks::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop tables in reverse order
        manager
            .drop_table(Table::drop().table(Tasks::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Scripts::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Mails::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Corners::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Episodes::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Podcasts::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Users::Table).to_owned())
            .await?;

        // Drop enum type
        manager
            .drop_type(Type::drop().if_exists().name(TaskStatus::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum Users {
    Table,
    Id,
    AuthId,
    Email,
    Name,
}

#[derive(DeriveIden)]
enum Podcasts {
    Table,
    Id,
    Title,
    UserId,
    Icon,
    CreatedAt,
    Description,
}

#[derive(DeriveIden)]
enum Episodes {
    Table,
    Id,
    Title,
    AudioUrl,
    UserId,
    PodcastId,
    SrtUrl,
    CreatedAt,
    Sections,
    Description,
    DurationSec,
}

#[derive(DeriveIden)]
enum Corners {
    Table,
    Id,
    Title,
    Description,
    RequestingMail,
    UserId,
    MailSchema,
    PodcastId,
}

#[derive(DeriveIden)]
enum Mails {
    Table,
    Id,
    Body,
    UserId,
    CornerId,
    CreatedAt,
}

#[derive(DeriveIden)]
enum Scripts {
    Table,
    Id,
    Template,
    UserId,
    Title,
    Description,
    Arguments,
}

#[derive(DeriveIden)]
enum Tasks {
    Table,
    Id,
    Status,
    Args,
    UserId,
    ExecuteAfter,
    ExecutedAt,
    ExecutedFinishedAt,
    Result,
    Cron,
}

#[derive(DeriveIden)]
enum TaskStatus {
    Table,
    Pending,
    Running,
    Completed,
    Failed,
}

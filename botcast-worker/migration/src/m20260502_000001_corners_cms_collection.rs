use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Add cms_collection_id to corners (CMS Collectionへの参照)
        manager
            .alter_table(
                Table::alter()
                    .table(Corners::Table)
                    .add_column(ColumnDef::new(Corners::CmsCollectionId).string().null())
                    .to_owned(),
            )
            .await?;

        // Drop mails table (メール機能はbotcast-cmsに移譲)
        manager
            .drop_table(Table::drop().table(Mails::Table).if_exists().to_owned())
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Restore mails table
        manager
            .create_table(
                Table::create()
                    .table(Mails::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Mails::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Mails::Body).json_binary().not_null())
                    .col(ColumnDef::new(Mails::UserId).uuid().not_null())
                    .col(ColumnDef::new(Mails::CornerId).uuid().not_null())
                    .col(
                        ColumnDef::new(Mails::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;

        // Remove cms_collection_id from corners
        manager
            .alter_table(
                Table::alter()
                    .table(Corners::Table)
                    .drop_column(Corners::CmsCollectionId)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum Corners {
    Table,
    CmsCollectionId,
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

use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Create vault schema
        manager
            .raw_sql("CREATE SCHEMA IF NOT EXISTS vault;")
            .await?;

        // Create vault.secrets table
        manager
            .raw_sql(
                r#"
                CREATE TABLE IF NOT EXISTS vault.secrets (
                    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                    name TEXT UNIQUE NOT NULL,
                    secret TEXT NOT NULL,
                    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
                );
                "#,
            )
            .await?;

        // Create vault.decrypted_secrets view
        manager
            .raw_sql(
                r#"
                CREATE OR REPLACE VIEW vault.decrypted_secrets AS
                SELECT 
                    id,
                    name,
                    secret,
                    created_at,
                    updated_at
                FROM vault.secrets;
                "#,
            )
            .await?;

        // Create vault functions
        manager
            .raw_sql(
                r#"
                CREATE OR REPLACE FUNCTION vault.create_secret(secret_value TEXT, secret_name TEXT)
                RETURNS UUID
                LANGUAGE plpgsql
                AS $$
                DECLARE
                    secret_id UUID;
                BEGIN
                    INSERT INTO vault.secrets (secret, name)
                    VALUES (secret_value, secret_name)
                    RETURNING id INTO secret_id;
                    
                    RETURN secret_id;
                END;
                $$;
                "#,
            )
            .await?;

        manager
            .raw_sql(
                r#"
                CREATE OR REPLACE FUNCTION vault.update_secret(secret_id UUID, secret_value TEXT, secret_name TEXT)
                RETURNS VOID
                LANGUAGE plpgsql
                AS $$
                BEGIN
                    UPDATE vault.secrets 
                    SET secret = secret_value, 
                        name = secret_name,
                        updated_at = NOW()
                    WHERE id = secret_id;
                END;
                $$;
                "#,
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop vault functions
        manager
            .raw_sql("DROP FUNCTION IF EXISTS vault.create_secret(TEXT, TEXT);")
            .await?;

        manager
            .raw_sql("DROP FUNCTION IF EXISTS vault.update_secret(UUID, TEXT, TEXT);")
            .await?;

        // Drop vault.decrypted_secrets view
        manager
            .raw_sql("DROP VIEW IF EXISTS vault.decrypted_secrets;")
            .await?;

        // Drop vault.secrets table
        manager
            .raw_sql("DROP TABLE IF EXISTS vault.secrets;")
            .await?;

        // Drop vault schema
        manager
            .raw_sql("DROP SCHEMA IF EXISTS vault CASCADE;")
            .await?;

        Ok(())
    }
}

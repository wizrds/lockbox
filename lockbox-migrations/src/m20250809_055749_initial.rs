use sea_orm_migration::prelude::*;


#[derive(DeriveMigrationName)]
pub struct Migration;


#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Tables
        manager
            .create_table(
                Table::create()
                    .table(ApiKey::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(ApiKey::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(ApiKey::TenantId).string().not_null().default("default"))
                    .col(ColumnDef::new(ApiKey::Namespace).string().not_null())
                    .col(ColumnDef::new(ApiKey::Tag).string().null())
                    .col(ColumnDef::new(ApiKey::ShortKey).string().not_null())
                    .col(ColumnDef::new(ApiKey::LongKeyHash).string().not_null())
                    .col(ColumnDef::new(ApiKey::CreatedAt).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(ApiKey::Owner).string().not_null())
                    .col(ColumnDef::new(ApiKey::Scope).string().null())
                    .col(ColumnDef::new(ApiKey::Revoked).boolean().not_null().default(false))
                    .col(ColumnDef::new(ApiKey::RevokedAt).timestamp_with_time_zone().null())
                    .col(ColumnDef::new(ApiKey::ExpiresAt).timestamp_with_time_zone().null())
                    .col(ColumnDef::new(ApiKey::LastUsedAt).timestamp_with_time_zone().null())
                    .col(ColumnDef::new(ApiKey::Metadata).json().not_null().default("{}"))
                    .to_owned()
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(ApiKeyNamespace::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(ApiKeyNamespace::Name).string().not_null())
                    .col(ColumnDef::new(ApiKeyNamespace::TenantId).string().not_null().default("default"))
                    .col(ColumnDef::new(ApiKeyNamespace::CreatedAt).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(ApiKeyNamespace::IsDefault).boolean().not_null().default(false))
                    .primary_key(
                        &mut Index::create()
                            .if_not_exists()
                            .name("pk_api_key_namespace")
                            .col(ApiKeyNamespace::Name)
                            .col(ApiKeyNamespace::TenantId)
                            .to_owned()
                    )
                    .to_owned()
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(ApiKeyTag::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(ApiKeyTag::Name).string().not_null())
                    .col(ColumnDef::new(ApiKeyTag::Namespace).string().not_null())
                    .col(ColumnDef::new(ApiKeyTag::TenantId).string().not_null().default("default"))
                    .col(ColumnDef::new(ApiKeyTag::CreatedAt).timestamp_with_time_zone().not_null())
                    .primary_key(
                        &mut Index::create()
                            .if_not_exists()
                            .name("pk_api_key_tag")
                            .col(ApiKeyTag::Name)
                            .col(ApiKeyTag::Namespace)
                            .col(ApiKeyTag::TenantId)
                            .to_owned()
                    )
                    .to_owned()
            )
            .await?;

        // Indexes
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_api_key_owner")
                    .table(ApiKey::Table)
                    .col(ApiKey::Owner)
                    .to_owned()
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_api_key_tenant_namespace_tag_short_key")
                    .table(ApiKey::Table)
                    .col(ApiKey::TenantId)
                    .col(ApiKey::Namespace)
                    .col(ApiKey::Tag)
                    .col(ApiKey::ShortKey)
                    .to_owned()
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_api_key_active")
                    .table(ApiKey::Table)
                    .col(ApiKey::TenantId)
                    .col(ApiKey::Namespace)
                    .col(ApiKey::Tag)
                    .col(ApiKey::ShortKey)
                    .and_where(Expr::col(ApiKey::Revoked).eq(false))
                    .to_owned()
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_api_key_namespace_default")
                    .table(ApiKeyNamespace::Table)
                    .col(ApiKeyNamespace::TenantId)
                    .and_where(Expr::col(ApiKeyNamespace::IsDefault).eq(true))
                    .to_owned()
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_api_key_tag_namespace_name")
                    .table(ApiKeyTag::Table)
                    .col(ApiKeyTag::TenantId)
                    .col(ApiKeyTag::Namespace)
                    .col(ApiKeyTag::Name)
                    .to_owned()
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_api_key_tag_namespace")
                    .table(ApiKeyTag::Table)
                    .col(ApiKeyTag::TenantId)
                    .col(ApiKeyTag::Namespace)
                    .to_owned()
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(
                Table::drop()
                    .table(ApiKeyNamespace::Table)
                    .to_owned()
            )
            .await?;

        manager
            .drop_table(
                Table::drop()
                    .table(ApiKey::Table)
                    .to_owned()
            )
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum ApiKey {
    Table,
    Id,
    TenantId,
    Namespace,
    Tag,
    ShortKey,
    LongKeyHash,
    CreatedAt,
    Owner,
    Scope,
    Revoked,
    RevokedAt,
    ExpiresAt,
    LastUsedAt,
    Metadata,
}


#[derive(DeriveIden)]
enum ApiKeyNamespace {
    Table,
    TenantId,
    Name,
    CreatedAt,
    IsDefault,
}


#[derive(DeriveIden)]
enum ApiKeyTag {
    Table,
    TenantId,
    Namespace,
    Name,
    CreatedAt,
}
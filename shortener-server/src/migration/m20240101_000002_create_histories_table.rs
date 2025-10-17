use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Histories::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Histories::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Histories::UrlId).integer().not_null())
                    .col(
                        ColumnDef::new(Histories::ShortCode)
                            .string_len(16)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Histories::IpAddress)
                            .string_len(45)
                            .not_null(),
                    )
                    .col(ColumnDef::new(Histories::UserAgent).text().null())
                    .col(ColumnDef::new(Histories::Referer).text().null())
                    .col(ColumnDef::new(Histories::Country).string_len(100).null())
                    .col(ColumnDef::new(Histories::Region).string_len(100).null())
                    .col(ColumnDef::new(Histories::Province).string_len(100).null())
                    .col(ColumnDef::new(Histories::City).string_len(100).null())
                    .col(ColumnDef::new(Histories::Isp).string_len(100).null())
                    .col(ColumnDef::new(Histories::DeviceType).string_len(20).null())
                    .col(ColumnDef::new(Histories::Os).string_len(50).null())
                    .col(ColumnDef::new(Histories::Browser).string_len(50).null())
                    .col(ColumnDef::new(Histories::AccessedAt).timestamp().not_null())
                    .col(
                        ColumnDef::new(Histories::CreatedAt)
                            .timestamp()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_histories_url_id")
                            .from(Histories::Table, Histories::UrlId)
                            .to(Urls::Table, Urls::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::NoAction),
                    )
                    .to_owned(),
            )
            .await?;

        // Create index on url_id
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_histories_url_id")
                    .table(Histories::Table)
                    .col(Histories::UrlId)
                    .to_owned(),
            )
            .await?;

        // Create index on short_code
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_histories_short_code")
                    .table(Histories::Table)
                    .col(Histories::ShortCode)
                    .to_owned(),
            )
            .await?;

        // Create index on accessed_at
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_histories_accessed_at")
                    .table(Histories::Table)
                    .col(Histories::AccessedAt)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Histories::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Histories {
    Table,
    Id,
    UrlId,
    ShortCode,
    IpAddress,
    UserAgent,
    Referer,
    Country,
    Region,
    Province,
    City,
    Isp,
    DeviceType,
    Os,
    Browser,
    AccessedAt,
    CreatedAt,
}

#[derive(DeriveIden)]
enum Urls {
    Table,
    Id,
}

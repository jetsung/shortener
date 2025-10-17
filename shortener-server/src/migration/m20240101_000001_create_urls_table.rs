use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Urls::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Urls::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(Urls::Code)
                            .string_len(16)
                            .not_null()
                            .unique_key(),
                    )
                    .col(ColumnDef::new(Urls::OriginalUrl).text().not_null())
                    .col(ColumnDef::new(Urls::Describe).text().null())
                    .col(ColumnDef::new(Urls::Status).integer().not_null().default(1))
                    .col(
                        ColumnDef::new(Urls::CreatedAt)
                            .timestamp()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(Urls::UpdatedAt)
                            .timestamp()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        // Create index on code
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_urls_code")
                    .table(Urls::Table)
                    .col(Urls::Code)
                    .to_owned(),
            )
            .await?;

        // Create index on status
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_urls_status")
                    .table(Urls::Table)
                    .col(Urls::Status)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Urls::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Urls {
    Table,
    Id,
    Code,
    OriginalUrl,
    Describe,
    Status,
    CreatedAt,
    UpdatedAt,
}

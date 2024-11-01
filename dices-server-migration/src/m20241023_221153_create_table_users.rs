use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(User::Table)
                    .if_not_exists()
                    .col(
                        uuid(User::Id)
                            .primary_key()
                            .default(PgFunc::gen_random_uuid()),
                    )
                    .col(
                        text(User::Name)
                            .unique_key()
                            .check(
                                Expr::col(User::Name)
                                    .not_like("% %")
                                    .and(Expr::col(User::Name).not_like("%\t%"))
                                    .and(Expr::col(User::Name).not_like("%\n%")),
                            )
                            .check(Func::char_length(Expr::col(User::Name)).gt(0)),
                    )
                    .col(text(User::Password))
                    .col(
                        timestamp_with_time_zone(User::CreatedAt)
                            .default(Expr::current_timestamp())
                            .check(Expr::col(User::CreatedAt).lte(Expr::current_timestamp())),
                    )
                    .col(
                        timestamp_with_time_zone(User::LastAccess)
                            .default(Expr::current_timestamp())
                            .check(Expr::col(User::LastAccess).lte(Expr::current_timestamp()))
                            .check(Expr::col(User::CreatedAt).lte(Expr::col(User::LastAccess))),
                    )
                    .take(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(User::Table).if_exists().take())
            .await
    }
}

#[derive(DeriveIden)]
pub(super) enum User {
    Table,
    Id,
    Name,
    Password,
    CreatedAt,
    LastAccess,
}

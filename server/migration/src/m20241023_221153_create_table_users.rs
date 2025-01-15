use sea_orm_migration::{prelude::*, schema::{pk_uuid, text, timestamp_with_time_zone}};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(User::Table)
                    .comment("Registered users of the `dices` server")
                    .if_not_exists()
                    .col(
                        pk_uuid(User::Id)
                            .comment("UUID of the user")
                            .default(PgFunc::gen_random_uuid()),
                    )
                    .col(
                        text(User::Name)
                            .comment("User's unique name, disallowing spaces")
                            .unique_key()
                            .check(
                                Expr::col(User::Name)
                                    .not_like("% %")
                                    .and(Expr::col(User::Name).not_like("%\t%"))
                                    .and(Expr::col(User::Name).not_like("%\n%")),
                            )
                            .check(Func::char_length(Expr::col(User::Name)).gt(0)),
                    )
                    .col(text(User::Password).comment("Hashed representation of the user's password"))
                    .col(
                        timestamp_with_time_zone(User::CreatedAt)
                            .comment("Timestamp marking when the user was created, defaults to the current time")
                            .default(Expr::current_timestamp())
                    )
                    .col(
                        timestamp_with_time_zone(User::LastAccess)
                            .comment("Timestamp of the user's most recent access, defaults to the current time")
                            .default(Expr::current_timestamp())
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

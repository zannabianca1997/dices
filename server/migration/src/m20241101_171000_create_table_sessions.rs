use extension::postgres::Type;
use sea_orm_migration::{prelude::*, schema::*};

use crate::m20241023_221153_create_table_users::User;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Session::Table)
                    .comment("Sessions at which users can connect to play `dices`")
                    .if_not_exists()
                    .col(
                        uuid(Session::Id)
                            .comment("UUID of the session")
                            .primary_key()
                            .default(PgFunc::gen_random_uuid()),
                    )
                    .col(
                        text(Session::Name)
                            .comment("Short, unique name for the session, disallowing leading and trailing whitespace characters, and newlines")
                            .check(
                                Expr::col(Session::Name)
                                    .not_like(" %")
                                    .and(Expr::col(Session::Name).not_like("\t%"))
                                    .and(Expr::col(Session::Name).not_like("% "))
                                    .and(Expr::col(Session::Name).not_like("%\t")),
                            )
                            .check(Expr::col(Session::Name).not_like("%\n%"))
                            .check(Func::char_length(Expr::col(Session::Name)).gt(0)),
                    )
                    .col(
                        text_null(Session::Description)
                            .comment("Optional detailed description of the session"),
                    )
                    .col(
                        timestamp_with_time_zone(Session::CreatedAt)
                            .comment("Timestamp marking when the session was created, defaults to the current time")
                            .default(Expr::current_timestamp())
                            .check(Expr::col(Session::CreatedAt).lte(Expr::current_timestamp())),
                    )
                    .take(),
            )
            .await?;
        manager
            .create_type(
                Type::create()
                    .as_enum(UserRole::Table)
                    .values([UserRole::Admin, UserRole::Player, UserRole::Observer])
                    .to_owned(),
            )
            .await?;
        manager
            .create_table(
                Table::create()
                    .table(SessionUser::Table)
                    .comment("Links of users to sessions they have joined, with roles and timestamps for activity tracking")
                    .if_not_exists()
                    .col(uuid(SessionUser::Session).comment("Identifier of the session the user joined"))
                    .foreign_key(
                        ForeignKey::create()
                            .to(Session::Table, Session::Id)
                            .from(SessionUser::Table, SessionUser::Session)
                            .on_update(ForeignKeyAction::Cascade)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .col(uuid(SessionUser::User).comment("Identifier of the user joining the session"))
                    .foreign_key(
                        ForeignKey::create()
                            .to(User::Table, User::Id)
                            .from(SessionUser::Table, SessionUser::User)
                            .on_update(ForeignKeyAction::Cascade)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .primary_key(
                        Index::create()
                            .col(SessionUser::Session)
                            .col(SessionUser::User),
                    )
                    .col(
                        ColumnDef::new_with_type(
                            SessionUser::Role,
                            ColumnType::custom(UserRole::Table.to_string()),
                        )
                        .not_null()
                        .comment("Role assigned to the user within the session")
                        .take(),
                    )
                    .col(
                        timestamp_with_time_zone(SessionUser::AddedAt)
                            .comment("Timestamp marking when the user was added to the session, defaults to the current time")
                            .default(Expr::current_timestamp())
                            .check(Expr::col(SessionUser::AddedAt).lte(Expr::current_timestamp())),
                    )
                    .col(
                        timestamp_with_time_zone_null(SessionUser::LastAccess)
                            .comment("Timestamp of the user's most recent interaction with the session, must be after they were added")
                            .check(
                                Expr::col(SessionUser::LastAccess).lte(Expr::current_timestamp()),
                            )
                            .check(
                                Expr::col(SessionUser::LastAccess)
                                    .gte(Expr::col(SessionUser::AddedAt)),
                            ),
                    )
                    .take(),
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(SessionUser::Table).if_exists().take())
            .await?;
        manager
            .drop_type(Type::drop().name(UserRole::Table).if_exists().to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Session::Table).if_exists().take())
            .await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
pub enum Session {
    Table,
    Id,
    Name,
    Description,
    CreatedAt,
}

#[derive(DeriveIden)]
enum SessionUser {
    Table,
    Session,
    User,
    Role,
    AddedAt,
    LastAccess,
}

#[derive(DeriveIden)]
enum UserRole {
    Table,
    Admin,
    Player,
    Observer,
}

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
                    .if_not_exists()
                    .col(
                        uuid(Session::Id)
                            .primary_key()
                            .default(PgFunc::gen_random_uuid()),
                    )
                    .col(
                        text(Session::Name)
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
                    .col(text_null(Session::Description))
                    .col(
                        timestamp_with_time_zone(Session::CreatedAt)
                            .default(Expr::current_timestamp())
                            .check(Expr::col(Session::CreatedAt).lte(Expr::current_timestamp())),
                    )
                    .col(binary_null(Session::Image))
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
                    .if_not_exists()
                    .col(uuid(SessionUser::Session))
                    .foreign_key(
                        ForeignKey::create()
                            .to(Session::Table, Session::Id)
                            .from(SessionUser::Table, SessionUser::Session)
                            .on_update(ForeignKeyAction::Cascade)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .col(uuid(SessionUser::User))
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
                        .take(),
                    )
                    .col(
                        timestamp_with_time_zone(SessionUser::AddedAt)
                            .default(Expr::current_timestamp())
                            .check(Expr::col(SessionUser::AddedAt).lte(Expr::current_timestamp())),
                    )
                    .col(
                        timestamp_with_time_zone_null(SessionUser::LastAccess)
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
enum Session {
    Table,
    Id,
    Name,
    Description,
    CreatedAt,
    Image,
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

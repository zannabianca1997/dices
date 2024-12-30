use sea_orm_migration::{prelude::*, schema::*};

use crate::{
    m20241023_221153_create_table_users::User, m20241101_171000_create_table_sessions::Session,
};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .comment("Log item of the sessions")
                    .table(Log::Table)
                    .if_not_exists()
                    .col(big_integer(Log::Id).auto_increment().primary_key().comment("Position of the log item in the chat"))
                    .col(
                        uuid(Log::SessionId)
                            .comment("Identifier of the session to which this log belongs"),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(Log::Table, Log::SessionId)
                            .to(Session::Table, Session::Id)
                            .on_update(ForeignKeyAction::Cascade)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .col(
                        uuid_null(Log::UserId)
                            .comment("Identifier of the session to which this log belongs. `null` stand for a deleted user"),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(Log::Table, Log::UserId)
                            .to(User::Table, User::Id)
                            .on_update(ForeignKeyAction::Cascade)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .col(
                        big_integer_null(Log::SourceId)
                            .comment("Log that caused this log (generally the command that gave this result)"),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(Log::Table, Log::SourceId)
                            .to(Log::Table, Log::Id)
                            .on_update(ForeignKeyAction::Cascade)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .col(
                        timestamp_with_time_zone(Log::CreatedAt)
                            .comment("Timestamp indicating when the log was created, defaults to the current time")
                            .default(Expr::current_timestamp())
                            .check(Expr::col(Log::CreatedAt).lte(Expr::current_timestamp())),
                    )
                    .col(
                       json(Log::Content).comment("The content of the log")
                    )
                    .take(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Log::Table).if_exists().take())
            .await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
enum Log {
    Table,
    Id,
    SessionId,
    UserId,
    SourceId,
    CreatedAt,
    Content,
}

use sea_orm_migration::{prelude::*, schema::{json, pk_uuid, timestamp_with_time_zone, uuid, uuid_null}};

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
                    .col(
                        pk_uuid(Log::Id)
                        .default(PgFunc::gen_random_uuid())
                        .comment("Identifier of the log id")
                    )
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
                            .comment("Identifier of the user that caused this log. `null` stand for a deleted user"),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(Log::Table, Log::UserId)
                            .to(User::Table, User::Id)
                            .on_update(ForeignKeyAction::Cascade)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .col(
                        uuid_null(Log::AnswerTo)
                            .comment("Log that caused this log (generally the command that gave this result)"),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(Log::Table, Log::AnswerTo)
                            .to(Log::Table, Log::Id)
                            .on_update(ForeignKeyAction::Cascade)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .col(
                        timestamp_with_time_zone(Log::CreatedAt)
                            .comment("Timestamp indicating when the log was created, defaults to the current time")
                            .default(Expr::current_timestamp())
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
    AnswerTo,
    CreatedAt,
    Content,
}

use sea_orm_migration::{prelude::*, schema::{binary, pk_uuid, timestamp_with_time_zone, timestamp_with_time_zone_null}};

use crate::m20241101_171000_create_table_sessions::Session;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .comment("Engine states associated with sessions")
                    .table(Engine::Table)
                    .if_not_exists()
                    .col(
                        pk_uuid(Engine::SessionId)
                            .comment("Identifier of the session to which this engine state belongs"),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(Engine::Table, Engine::SessionId)
                            .to(Session::Table, Session::Id)
                            .on_update(ForeignKeyAction::Cascade)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .col(
                        timestamp_with_time_zone(Engine::CreatedAt)
                            .comment("Timestamp indicating when the engine state was created, defaults to the current time")
                            .default(Expr::current_timestamp())
                    )
                    .col(
                        timestamp_with_time_zone_null(Engine::LastCommandAt)
                            .comment("Timestamp of the most recent command executed by the engine, must be after its creation")
                            .check(
                                Expr::col(Engine::LastCommandAt).gte(Expr::col(Engine::CreatedAt)),
                            ),
                    )
                    .col(binary(Engine::State).comment("Serialized binary data representing the current state of the engine"))
                    .take(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Engine::Table).if_exists().take())
            .await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
enum Engine {
    Table,
    SessionId,
    CreatedAt,
    LastCommandAt,
    State,
}

//! # `/sessions/{session}`: API regarding a single session
//!
//! This is the main api with which the user can communicate with a session

use axum::{
    debug_handler,
    extract::{FromRef, State},
};
use chrono::DateTime;
use dices_server_migration::{OnConflict, OnConflictUpdate};
use futures::{FutureExt, StreamExt, TryFutureExt, TryStreamExt};
use nunny::NonEmpty;
use sea_orm::{
    ActiveModelTrait as _, ActiveValue::NotSet, ColumnTrait as _, DatabaseConnection,
    EntityTrait as _, IntoActiveModel, QueryFilter as _, Set, TransactionTrait,
};
use serde_with::chrono::Local;
use tokio::{runtime::Handle, task::spawn_blocking};
use tokio_stream::wrappers::ReceiverStream;
use utoipa_axum::{router::OpenApiRouter, routes};

use dices_ast::Expression;
use dices_server_auth::{Autenticated, AuthKey};
use dices_server_dtos::{
    engine::{Command, CommandRejection, CommandResult},
    session::{
        SessionCreateDto, SessionGetError, SessionPathData, SessionQueryDto, SessionUpdateDto,
        SessionUpdateError,
    },
};
use dices_server_entities::{
    engine::{self, DatabaseEngine},
    log::{self, content::LogContent},
    prelude::*,
    sea_orm_active_enums::UserRole,
    session::SessionId,
    session_user,
    user::UserId,
};
use dices_server_intrisics::ServerIntrisics;

#[utoipa::path(
    get,
    path = "/",
    responses(SessionQueryDto, SessionGetError),
    params(SessionPathData)
)]
#[debug_handler(state = crate::app::App)]
/// Get info about the session
///
/// Get the info about the session requested.
/// This only works if the current user is part of the session.
async fn session_get(
    State(db): State<DatabaseConnection>,
    user_id: Autenticated<UserId>,
    SessionPathData { id: session_id }: SessionPathData,
) -> Result<SessionQueryDto, SessionGetError> {
    let (session, session_user) = fetch_session_data(&db, user_id, session_id).await?;

    Ok(SessionQueryDto {
        session,
        session_user: session_user.into(),
    })
}

#[utoipa::path(
    put,
    path = "/",
    request_body=SessionCreateDto,
    responses(SessionQueryDto, SessionUpdateError),
    params(SessionPathData)
)]
#[debug_handler(state = crate::app::App)]
/// Edit info about the session
///
/// Edit the info about the session.
/// This only works if the current user is part of the session, and has role `Admin`.
async fn session_put(
    State(db): State<DatabaseConnection>,
    user_id: Autenticated<UserId>,
    SessionPathData { id: session_id }: SessionPathData,
    SessionCreateDto { name, description }: SessionCreateDto,
) -> Result<SessionQueryDto, SessionUpdateError> {
    // First, we fetch it, guaranteeing it exist and the user is a member
    let (session, session_user) = fetch_session_data(&db, user_id, session_id).await?;

    // Check the permissions
    if session_user.role < UserRole::Admin {
        return Err(SessionUpdateError::NotAdmin);
    }

    // Apply the update
    let mut session = session.into_active_model();
    session.name = Set(name);
    session.description = Set(description);
    let session = session.update(&db).await?;

    Ok(SessionQueryDto {
        session,
        session_user: session_user.into(),
    })
}

#[utoipa::path(
    patch,
    path = "/",
    request_body=SessionUpdateDto,
    responses(SessionQueryDto, SessionUpdateError),
    params(SessionPathData)
)]
#[debug_handler(state = crate::app::App)]
/// Patch info about the session
///
/// Patch the info about the session, allowing partial editing.
/// This only works if the current user is part of the session, and has role `Admin`.
async fn session_patch(
    State(db): State<DatabaseConnection>,
    user_id: Autenticated<UserId>,
    SessionPathData { id: session_id }: SessionPathData,
    SessionUpdateDto { name, description }: SessionUpdateDto,
) -> Result<SessionQueryDto, SessionUpdateError> {
    // First, we fetch it, guaranteeing it exist and the user is a member
    let (session, session_user) = fetch_session_data(&db, user_id, session_id).await?;

    // Check the permissions
    if session_user.role < UserRole::Admin {
        return Err(SessionUpdateError::NotAdmin);
    }

    // Apply the update
    let mut session = session.into_active_model();
    if let Some(name) = name {
        session.name = Set(name);
    }
    if let Some(description) = description {
        session.description = Set(description);
    }
    let session = session.update(&db).await?;

    Ok(SessionQueryDto {
        session,
        session_user: session_user.into(),
    })
}

#[utoipa::path(
    delete,
    path = "/",
    responses(
        (status= OK, description="Session successfully deleted"),SessionUpdateError),
    params(SessionPathData)
)]
#[debug_handler(state = crate::app::App)]
/// Delete the session
///
/// Delete the session
/// This only works if the current user is part of the session, and has role `Admin`.
async fn session_delete(
    State(db): State<DatabaseConnection>,
    user_id: Autenticated<UserId>,
    SessionPathData { id: session_id }: SessionPathData,
) -> Result<(), SessionUpdateError> {
    // First, we fetch it, guaranteeing it exist and the user is a member
    let (session, session_user) = fetch_session_data(&db, user_id, session_id).await?;

    // Check the permissions
    if session_user.role < UserRole::Admin {
        return Err(SessionUpdateError::NotAdmin);
    }

    // Apply the update
    session.into_active_model().delete(&db).await?;

    Ok(())
}

#[utoipa::path(
    post,
    path = "/command",
    request_body=String,
    responses(CommandResult, CommandRejection),
    params(SessionPathData)
)]
#[debug_handler(state = crate::app::App)]
/// Execute a command in this session
///
/// Execute a command on the engine of the session, and return the resulting logs.
/// This only works if the user is part of the session, and has either role `Admin` or `Player`
async fn command_post(
    State(db): State<DatabaseConnection>,
    user_id: Autenticated<UserId>,
    SessionPathData { id: session_id }: SessionPathData,
    Command { source, value }: Command,
) -> Result<CommandResult, CommandRejection> {
    // First, we fetch it, guaranteeing it exist and the user is a member
    let (_, session_user) = fetch_session_data(&db, user_id, session_id).await?;

    // Check the permissions
    if session_user.role < UserRole::Player {
        return Err(CommandRejection::NotAPlayer);
    }

    // Trace
    tracing::debug!(
        command = ?source,
        user = ?user_id.inner(),
        session = ?session_id,
        "Command issued",
    );

    // Enter a transaction, to ensure the consistency of the engine state
    // This ensures that each command appears before or after each other, never mixing
    // e.g. CommandA is issued, then CommandB. CommandA completes, then CommandB, shadowing
    // in the db the result of A
    let logs = db
        .transaction(|db| {
            Box::pin(async move {
                // Get the engine, or create a new one
                let engine_model = Engine::find_by_id(session_id)
                    .one(db)
                    .await?
                    .map(IntoActiveModel::into_active_model)
                    .unwrap_or_else(|| engine::ActiveModel {
                        session_id: Set(session_id),
                        state: Set(DatabaseEngine(dices_engine::Engine::new())),
                        ..Default::default()
                    });

                // Channel to receive logs to
                let (logs_sender, logs_receiver) = tokio::sync::mpsc::channel(
                    /*
                    Maximum number of simultaneus log awaiting insertion
                    TODO: make this configurable
                    */
                    15,
                );
                let answers = ReceiverStream::new(logs_receiver);

                // Start of evaluation
                let evaluation_start = Local::now();

                // Starting the engine solving thread
                let solve_thread = spawn_blocking(move || {
                    solve_command(engine_model, &value, evaluation_start, logs_sender)
                })
                .map_err(CommandRejection::from)
                .and_then(|engine_model| {
                    Engine::insert(engine_model)
                        .on_conflict(
                            OnConflict::column(engine::Column::SessionId)
                                .update_columns([
                                    engine::Column::LastCommandAt,
                                    engine::Column::State,
                                ])
                                .to_owned(),
                        )
                        .exec_without_returning(db)
                        .map_ok(|n| debug_assert_eq!(n, 1))
                        .map_err(CommandRejection::from)
                })
                .boxed();

                // Insertion of the command log
                let command_log = log::ActiveModel {
                    id: NotSet,
                    answer_to: Set(None),
                    session_id: Set(session_id),
                    user_id: Set(Some(*user_id.inner())),
                    created_at: Set(evaluation_start.fixed_offset()),
                    content: Set(LogContent::Command { command: source }),
                }
                .insert(db)
                // Consecutive insertion of all the other logs
                .and_then(|command_log| {
                    let command_log_id = command_log.id;
                    answers
                        .map(
                            move |dices_server_intrisics::Log {
                                      created_at,
                                      content,
                                  }| {
                                log::ActiveModel {
                                    id: NotSet,
                                    session_id: Set(session_id),
                                    user_id: Set(Some(*user_id.inner())),
                                    answer_to: Set(Some(command_log_id)),
                                    created_at: Set(created_at.fixed_offset()),
                                    content: Set(match content {
                                        dices_server_intrisics::LogContent::Value(value) => {
                                            LogContent::Value { value }
                                        }
                                        dices_server_intrisics::LogContent::Manual(topic) => {
                                            LogContent::Manual { topic }
                                        }
                                        dices_server_intrisics::LogContent::Error {
                                            msg,
                                            sources,
                                        } => LogContent::Error { msg, sources },
                                    }),
                                }
                                .insert(db)
                            },
                        )
                        .buffer_unordered(
                            /*
                            Maximum number of simultaneus log insertion to the database
                            TODO: make this configurable
                            */
                            15,
                        )
                        .try_fold(vec![command_log], |mut logs, answer| async move {
                            let pos = logs
                                .binary_search_by(|other| answer.log_order(other).reverse())
                                .unwrap_err();
                            logs.insert(pos, answer);
                            Ok(logs)
                        })
                })
                .map_err(CommandRejection::from)
                .boxed();

                let ((), logs) = futures::try_join!(solve_thread, command_log)?;

                Ok(logs.into_boxed_slice())
            })
        })
        .await
        .map_err(|err| match err {
            sea_orm::TransactionError::Connection(db_err) => CommandRejection::from(db_err),
            sea_orm::TransactionError::Transaction(err) => err,
        })?;

    // Trace
    tracing::debug!(
        logs_emitted = logs.len(),
        user = ?user_id.inner(),
        session = ?session_id,
        "Command completed",
    );

    Ok(CommandResult(logs))
}

/// Solve a command in a engine
fn solve_command(
    mut engine_model: engine::ActiveModel,
    command: &NonEmpty<[Expression<ServerIntrisics>]>,
    evaluation_start: DateTime<Local>,
    logs_sender: tokio::sync::mpsc::Sender<dices_server_intrisics::Log>,
) -> engine::ActiveModel {
    // Recover the engine and hydrate the data
    // This will clone the value, because we want to check after if it's the same state to avoid updating it in the server
    let mut engine = engine_model
        .state
        .clone()
        .unwrap()
        .0
        .map_injected_intrisics_data(|dry| dry.hydrate(logs_sender));

    // Evaluate the command result
    engine_model.last_command_at = Set(Some(evaluation_start.fixed_offset()));
    let eval_result = engine.eval_multiple(command);

    // Log the result
    engine.injected_intrisics_data_mut().log(match eval_result {
        Ok(v) => dices_server_intrisics::LogContent::Value(v),
        Err(err) => dices_server_intrisics::LogContent::error(err),
    });

    // Put the engine back to be saved
    engine_model.state.set_if_not_equals(DatabaseEngine(
        engine.map_injected_intrisics_data(|wet| wet.dehydrate()),
    ));

    engine_model
}

pub fn router<S: Clone + Send + Sync + 'static>() -> OpenApiRouter<S>
where
    DatabaseConnection: FromRef<S>,
    AuthKey: FromRef<S>,
{
    OpenApiRouter::default()
        .routes(routes!(
            session_get,
            session_put,
            session_patch,
            session_delete
        ))
        .routes(routes!(command_post))
}

async fn fetch_session_data(
    db: &DatabaseConnection,
    user_id: Autenticated<UserId>,
    session_id: SessionId,
) -> Result<(dices_server_entities::session::Model, session_user::Model), SessionGetError> {
    let (session, session_user) = Session::find_by_id(session_id)
        .find_also_related(SessionUser)
        .filter(session_user::Column::User.eq(*user_id.inner()))
        .one(db)
        .await?
        .ok_or(SessionGetError::NotFound)?;

    let session_user = session_user.expect("The query should only return sessions with users");

    Ok((session, session_user))
}

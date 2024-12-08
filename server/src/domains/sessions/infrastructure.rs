use super::domain::models::{
    Session, SessionId, SessionUser, SessionsGetNextError, UserRole, UsersGetError,
    UsersGetNextError,
};
use crate::domains::user::UserId;
use crate::entities::{self};
use chrono::Utc;
use sea_orm::{
    ColumnTrait, ConnectionTrait, DbErr, EntityTrait, IntoActiveModel, QueryFilter, QuerySelect,
    TransactionError, TransactionTrait,
};
use uuid::Uuid;

mod converters;

pub(super) async fn create(
    session: Session,
    db: &(impl ConnectionTrait + TransactionTrait),
    first_user: UserId,
) -> Result<(), DbErr> {
    let session_user = SessionUser {
        session: session.id,
        user: first_user,
        role: UserRole::Admin,
        added_at: Utc::now(),
        last_access: None,
    };
    let model: entities::session::Model = session.into();
    // Running the save and add a user in a transaction so no session without user is ever created
    db.transaction(|db| {
        Box::pin(async move {
            // Create the session
            entities::prelude::Session::insert(model.into_active_model())
                .exec(db)
                .await?;
            // Add this user as the first admin of the session
            create_session_user(session_user, db).await?;
            Ok(())
        })
    })
    .await
    .map_err(|err| match err {
        TransactionError::Connection(err) => err,
        TransactionError::Transaction(err) => err,
    })
}

pub(super) async fn find_by_id(
    db: &impl ConnectionTrait,
    id: SessionId,
    requester: UserId,
) -> Result<Option<(Session, SessionUser)>, DbErr> {
    entities::prelude::Session::find_by_id(Uuid::from(id))
        .find_with_related(entities::prelude::SessionUser)
        .filter(entities::session_user::Column::User.eq(*requester.as_ref()))
        .limit(1)
        .all(db)
        .await?
        .pop()
        .map(|(model, mut users)| {
            debug_assert_eq!(users.len(), 1, "The query should fetch a single user");
            Ok((
                model.try_into().map_err(|err| DbErr::TryIntoErr {
                    from: "entities::session::Model",
                    into: "Session",
                    source: Box::new(err),
                })?,
                users.pop().unwrap().into(),
            ))
        })
        .transpose()
}
pub(super) async fn find_all(
    db: &(impl ConnectionTrait + TransactionTrait),
    requester: UserId,
) -> Result<impl Iterator<Item = Result<(Session, SessionUser), SessionsGetNextError>>, DbErr> {
    Ok(entities::prelude::Session::find()
        .find_with_related(entities::prelude::SessionUser)
        .filter(entities::session_user::Column::User.eq(*requester.as_ref()))
        .all(db)
        .await?
        .into_iter()
        .map(|(model, mut users)| {
            debug_assert_eq!(users.len(), 1, "The query should fetch a single user");
            Ok((
                model.try_into().map_err(|err| DbErr::TryIntoErr {
                    from: "entities::session::Model",
                    into: "Session",
                    source: Box::new(err),
                })?,
                users.pop().unwrap().into(),
            ))
        }))
}

pub(super) async fn fetch_users(
    db: &impl ConnectionTrait,
    session: &Session,
) -> Result<impl Iterator<Item = Result<SessionUser, UsersGetNextError>>, UsersGetError> {
    Ok(
        entities::prelude::Session::find_by_id(Uuid::from(session.id))
            .find_with_related(entities::prelude::SessionUser)
            .all(db)
            .await?
            .pop()
            .expect("All the session calling this method should be present in the database")
            .1
            .into_iter()
            .map(|user| Ok(user.into())),
    )
}

pub(super) async fn create_session_user(
    session_user: SessionUser,
    db: &impl ConnectionTrait,
) -> Result<(), DbErr> {
    let model: entities::session_user::Model = session_user.into();
    entities::prelude::SessionUser::insert(model.into_active_model())
        .exec(db)
        .await?;
    Ok(())
}
pub(super) async fn find_session_user(
    db: &impl ConnectionTrait,
    session: SessionId,
    user: UserId,
) -> Result<Option<SessionUser>, DbErr> {
    entities::prelude::SessionUser::find_by_id((*session.as_ref(), *user.as_ref()))
        .one(db)
        .await
        .map(|o| o.map(Into::into))
}

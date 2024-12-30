//! Content of a log

use dices_server_intrisics::ServerIntrisics;
use sea_orm::FromJsonQueryResult;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, FromJsonQueryResult, PartialEq, Eq)]
#[serde(tag = "type")]
pub enum LogContent {
    /// A user command
    ///
    /// This is stored as a string, preserving comments and formattation
    Command(String),
    /// A value resulting either from a user command, or a `print` intrisic
    Value(dices_ast::Value<ServerIntrisics>),
    /// A manual page, requsted with the `help` intrisic
    Manual(String),
}

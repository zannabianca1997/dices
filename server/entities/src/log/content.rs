//! Content of a log

use std::error::Error;

use dices_server_intrisics::ServerIntrisics;
use sea_orm::FromJsonQueryResult;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Deserialize, Serialize, FromJsonQueryResult, PartialEq, Eq, ToSchema)]
#[serde(tag = "type")]
pub enum LogContent {
    /// A user command
    Command { command: String },
    /// A value resulting either from a user command, or a `print` intrisic
    Value {
        value: dices_ast::Value<ServerIntrisics>,
    },
    /// A runtime error
    Error { msg: String, sources: Box<[String]> },
    /// A manual page, requsted with the `help` intrisic
    Manual { topic: Box<str> },
}

impl From<dices_engine::SolveError<ServerIntrisics>> for LogContent {
    fn from(v: dices_engine::SolveError<ServerIntrisics>) -> Self {
        let msg = v.to_string();

        let mut source: Option<&dyn Error> = v.source();
        let mut sources = vec![];
        while let Some(next_source) = source {
            sources.push(next_source.to_string());
            source = next_source.source();
        }

        Self::Error {
            msg,
            sources: sources.into_boxed_slice(),
        }
    }
}

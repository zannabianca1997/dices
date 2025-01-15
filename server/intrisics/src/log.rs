//! Stuff needed to generate a log on the database

use std::error::Error;

use chrono::{DateTime, Local};

#[derive(Debug, Clone)]
/// A log generated during execution
pub struct Log {
    pub created_at: DateTime<Local>,
    pub content: LogContent,
}

#[derive(Debug, Clone)]
/// Content of the log
pub enum LogContent {
    /// A value resulting either from a user command, or a `print` intrisic
    Value(dices_ast::Value<crate::ServerIntrisics>),
    /// A manual page, requsted with the `help` intrisic
    Manual(Box<str>),
    /// A runtime error
    Error { msg: String, sources: Box<[String]> },
}

impl LogContent {
    pub fn error(err: impl Error) -> Self {
        let msg = err.to_string();
        let mut next_src = err.source();
        let mut sources = vec![];
        while let Some(src) = next_src {
            sources.push(src.to_string());
            next_src = src.source()
        }
        Self::Error {
            msg,
            sources: sources.into_boxed_slice(),
        }
    }
}

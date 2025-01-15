#![allow(clippy::boxed_local)]

use std::time::{SystemTime, UNIX_EPOCH};

use bincode::{Decode, Encode};
use chrono::Local;
use dices_ast::{intrisics::InjectedIntr, value::ValueNull, Value};
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub type ServerRng = rand_xoshiro::Xoshiro256PlusPlus;

mod log;

pub use log::*;

#[derive(Debug, Clone, Decode, Encode, Serialize, Deserialize, PartialEq, Eq)]
/// Server intrisics data that go into the database
pub struct ServerIntrisicsDryData {}

impl ServerIntrisicsDryData {
    pub const fn new() -> Self {
        Self {}
    }

    pub const fn hydrate(self, logs: tokio::sync::mpsc::Sender<Log>) -> ServerIntrisicsWetData {
        let Self {} = self;
        ServerIntrisicsWetData { logs }
    }
}
impl Default for ServerIntrisicsDryData {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
/// Server intrisics data for a running machine
pub struct ServerIntrisicsWetData {
    pub logs: tokio::sync::mpsc::Sender<Log>,
}

impl ServerIntrisicsWetData {
    pub fn dehydrate(self) -> ServerIntrisicsDryData {
        let Self { logs: _ } = self;
        ServerIntrisicsDryData {}
    }

    pub fn log(&mut self, content: LogContent) {
        let created_at = Local::now();

        self.logs
            .blocking_send(Log {
                created_at,
                content,
            })
            .expect("The send should be infallible");
    }
}

#[derive(Debug, Clone, Error, Serialize, Deserialize)]
pub enum ServerIntrisicsError {}

#[derive(
    Debug, Clone, Copy, Hash, PartialEq, Eq, Encode, Decode, Serialize, Deserialize, InjectedIntr,
)]
#[injected_intr(data = "ServerIntrisicsWetData", error = "ServerIntrisicsError")]
pub enum ServerIntrisics {
    /// Print a value
    #[injected_intr(calls = "print", prelude, std("repl."))]
    Print,

    /// Print a manual page
    #[injected_intr(calls = "help", prelude, std("repl."))]
    Help,

    /// Get the server time
    #[injected_intr(calls = "time", prelude, std("sys."))]
    Time,
}

fn print(
    data: &mut ServerIntrisicsWetData,
    params: Box<[Value<ServerIntrisics>]>,
) -> Result<Value<ServerIntrisics>, ServerIntrisicsError> {
    for value in params.into_vec().into_iter() {
        data.log(LogContent::Value(value));
    }
    Ok(Value::Null(ValueNull))
}

/// The page for help about `help`
const HELP_PAGE_FOR_HELP: &str = "std/repl/help";

fn help(
    data: &mut ServerIntrisicsWetData,
    params: Box<[Value<ServerIntrisics>]>,
) -> Result<Value<ServerIntrisics>, ServerIntrisicsError> {
    // the help intrisic never fails, at most it fallback on her help page itself
    let topic = match Box::<[_; 1]>::try_from(params) {
        Ok(s) => match *s {
            [Value::String(value_string)] => value_string.into(),
            _ => HELP_PAGE_FOR_HELP.into(),
        },
        Err(p) if p.is_empty() => "introduction".into(),
        _ => HELP_PAGE_FOR_HELP.into(),
    };
    data.log(LogContent::Manual(topic));
    Ok(Value::Null(ValueNull))
}

fn time(
    _data: &mut ServerIntrisicsWetData,
    _params: Box<[Value<ServerIntrisics>]>,
) -> Result<Value<ServerIntrisics>, ServerIntrisicsError> {
    Ok(Value::Number(
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            .into(),
    ))
}

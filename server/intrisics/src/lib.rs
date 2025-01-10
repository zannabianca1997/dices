use bincode::{Decode, Encode};
use chrono::Local;
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub type ServerRng = rand_xoshiro::Xoshiro256PlusPlus;

mod log;

pub use log::*;
use tokio::runtime::Runtime;

#[derive(Debug, Clone, Decode, Encode, Serialize, Deserialize, PartialEq, Eq)]
/// Server intrisics data that go into the database
pub struct ServerIntrisicsDryData {}

impl ServerIntrisicsDryData {
    pub fn new() -> Self {
        Self {}
    }

    pub fn hydrate(self, logs: tokio::sync::mpsc::Sender<Log>) -> ServerIntrisicsWetData {
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
            .expect("The send should be infallible")
    }
}

#[derive(Debug, Clone, Error, Serialize, Deserialize)]
pub enum ServerIntrisicsError {}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Encode, Decode, Serialize, Deserialize)]
pub enum ServerIntrisics {}

impl dices_ast::intrisics::InjectedIntr for ServerIntrisics {
    type Data = ServerIntrisicsWetData;

    type Error = ServerIntrisicsError;

    fn iter() -> impl IntoIterator<Item = Self> {
        []
    }

    fn name(&self) -> &'static str {
        match *self {}
    }

    fn named(name: &str) -> Option<Self> {
        match name {
            _ => None,
        }
    }

    fn call(
        &self,
        _data: &mut Self::Data,
        _params: Box<[dices_ast::Value<Self>]>,
    ) -> Result<dices_ast::Value<Self>, Self::Error> {
        match *self {}
    }
}

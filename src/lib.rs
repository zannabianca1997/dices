#![feature(extend_one)]
#![feature(string_leak)]
#![feature(iter_intersperse)]

mod cmd;
mod help;
mod parser;
mod throws;

pub use cmd::Cmd;
pub use throws::{Throws, ThrowsError};

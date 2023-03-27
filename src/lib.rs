#![feature(extend_one)]
#![feature(string_leak)]
#![feature(iter_intersperse)]
#![feature(iterator_try_reduce)]

mod cmd;
mod help;
mod parser;
mod throws;

pub use cmd::{Cmd, CmdError, CmdOutput, State};

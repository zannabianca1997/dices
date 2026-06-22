use std::{
    fs,
    io::{self},
};

use dices_values::{Injectable, injectable, string::ValueString};
use time::Time;

mod time;

/// System bindings
#[derive(Debug, Injectable, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Default, Hash)]
pub struct Sys {
    time: Time,
    read: Option<Read>,
    write: Option<Write>,
}

impl Sys {
    pub const fn new(filesystem: bool) -> Self {
        Self {
            time: Time::new(),
            read: if filesystem { Some(Read) } else { None },
            write: if filesystem { Some(Write) } else { None },
        }
    }
}

/// Read a file content
#[injectable]
pub fn Read(path: ValueString) -> io::Result<String> {
    fs::read_to_string(&*path)
}

/// Write a file content
#[injectable]
pub fn Write(path: ValueString, content: ValueString) -> io::Result<()> {
    fs::write(&*path, &*content)
}

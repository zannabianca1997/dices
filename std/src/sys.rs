use std::{
    fs,
    io::{self},
};

use dices_values::{Injectable, injectable, string::ValueString};
use time::Time;

pub mod time;

/// 5.3. Sys
///
/// Methods to access the system the session is running on.
///
/// The content of this module is the one that can most vary with the sandbox
/// setup: both `read` and `write` presence depends if access to the filesystem
/// is turned on or not.
#[derive(Debug, Injectable, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Default, Hash)]
pub struct Sys {
    pub time: Time,
    pub read: Option<Read>,
    pub write: Option<Write>,
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

/// 5.3.2. Read
///
/// Read a file content to a string. The file must be valid utf8.
///
/// ```dices no_run
/// >>> std.sys.read("hello.txt")
/// "hello"
/// ```
#[injectable]
pub fn Read(path: ValueString) -> io::Result<String> {
    fs::read_to_string(&*path)
}

/// 5.3.3. Write
///
/// Write a string to a file.
///
/// ```dices no_run
/// >>> std.sys.write("hello.txt", "hello")
/// >>> std.sys.read("hello.txt")
/// "hello"
/// ```
#[injectable]
pub fn Write(path: ValueString, content: ValueString) -> io::Result<()> {
    fs::write(&*path, &*content)
}

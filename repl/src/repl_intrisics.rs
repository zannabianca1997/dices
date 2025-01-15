//! Intrisics for the REPL
#![allow(clippy::boxed_local)]

use std::{
    fmt::Debug,
    fs, io, mem,
    path::Path,
    rc::Rc,
    time::{SystemTime, UNIX_EPOCH},
};

use derive_more::derive::{Display, Error};
use dices_ast::{
    intrisics::InjectedIntr,
    value::{Value, ValueList, ValueNull},
};
use dices_man::RenderOptions;
use termimad::{crossbeam::epoch::Pointable, crossterm::terminal, MadSkin};

use crate::{print_value, Graphic, ReplFatalError};

#[derive(Debug, Clone)]
pub struct Data {
    // stuff needed to visualize the elements
    graphic: Rc<Graphic>,
    skin: Rc<MadSkin>,

    // mark if the repl was quitted
    quitted: Quitted,
}

#[derive(Debug, Default)]
pub(crate) enum Quitted {
    #[default]
    No,
    Yes(Value<REPLIntrisics>),
    Fatal(ReplFatalError),
}

impl Quitted {
    pub fn take(&mut self) -> Self {
        mem::take(self)
    }
}

impl Clone for Quitted {
    fn clone(&self) -> Self {
        match self {
            Self::No => Self::No,
            Self::Yes(v) => Self::Yes(v.clone()),
            Self::Fatal(err) => panic!(
                "The repl data should not be cloned while a fatal error was running: {err:?}"
            ),
        }
    }
}

impl Data {
    pub fn new(graphic: Rc<Graphic>, skin: Rc<MadSkin>) -> Self {
        Self {
            graphic,
            skin,
            quitted: Quitted::No,
        }
    }

    pub(crate) fn quitted(&self) -> &Quitted {
        &self.quitted
    }
    pub(crate) fn quitted_mut(&mut self) -> &mut Quitted {
        &mut self.quitted
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, InjectedIntr)]
#[injected_intr(data = "Data", error = "REPLIntrisicsError")]
pub enum REPLIntrisics {
    /// Print a value
    #[injected_intr(calls = "print", prelude, std("repl."))]
    Print,
    /// Quit the repl
    #[injected_intr(calls = "quit", prelude, std("repl."))]
    Quit,
    /// Print a manual page
    #[injected_intr(calls = "help", prelude, std("repl."))]
    Help,

    /// Get the system time
    #[injected_intr(calls = "time", prelude, std("sys."))]
    Time,

    /// Read a file as a string
    #[injected_intr(calls = "file_read", std("sys.files."))]
    FileRead,
    /// Write a string to a file
    #[injected_intr(calls = "file_write", std("sys.files."))]
    FileWrite,
}
#[derive(Debug, Display, Error)]
pub enum REPLIntrisicsError {
    /// The `quit` intrisic was called
    Quitting,

    #[display("`file_read` must be called with a single string parameter")]
    FileReadUsage,
    #[display("Error while reading file")]
    FileReadError(io::Error),

    #[display("`file_write` must be called with two string parameters")]
    FileWriteUsage,
    #[display("Error while writing file")]
    FileWriteError(io::Error),
}

fn help(
    data: &mut Data,
    params: Box<[Value<REPLIntrisics>]>,
) -> Result<Value<REPLIntrisics>, REPLIntrisicsError> {
    // the help intrisic never fails, at most it fallback on her help page itself
    let topic = match &*params {
        [] => "introduction",
        [Value::String(s)] => s,
        _ => HELP_PAGE_FOR_HELP,
    };
    // search the manual. If absent, find the index.
    let content = dices_man::search(topic).unwrap_or_else(dices_man::index);
    // render the content, running the examples with the current prompt
    let content = content.rendered(RenderOptions {
        prompt: data.graphic.prompt().to_owned().into(),
        prompt_cont: data.graphic.prompt_cont().to_owned().into(),
        width: terminal::size()
            .map(|(w, _)| w as _)
            .unwrap_or(RenderOptions::default().width),
        ..Default::default()
    });
    // convert the content into a minimad text
    let content =
        mdast2minimad::to_minimad(&content).expect("All help pages should be convertible");
    // print it with the current skin
    println!(
        "{}",
        termimad::FmtText::from_text(
            &data.skin,
            content,
            terminal::size().map(|(w, _)| w as _).ok()
        )
    );
    Ok(Value::Null(ValueNull))
}

fn time(
    _data: &mut Data,
    _params: Box<[Value<REPLIntrisics>]>,
) -> Result<Value<REPLIntrisics>, REPLIntrisicsError> {
    Ok(Value::Number(
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            .into(),
    ))
}

fn file_read(
    _data: &mut Data,
    params: Box<[Value<REPLIntrisics>]>,
) -> Result<Value<REPLIntrisics>, REPLIntrisicsError> {
    let path = match Box::<[_; 1]>::try_from(params).map(|p| *p) {
        Ok([Value::String(path)]) => path,
        _ => return Err(REPLIntrisicsError::FileReadUsage),
    };
    let content =
        fs::read_to_string(Path::new(&**path)).map_err(REPLIntrisicsError::FileReadError)?;
    Ok(Value::String(content.into()))
}

fn file_write(
    _data: &mut Data,
    params: Box<[Value<REPLIntrisics>]>,
) -> Result<Value<REPLIntrisics>, REPLIntrisicsError> {
    let (path, content) = match Box::<[_; 2]>::try_from(params).map(|p| *p) {
        Ok([Value::String(path), Value::String(content)]) => (path, content),
        _ => return Err(REPLIntrisicsError::FileWriteUsage),
    };
    fs::write(Path::new(&**path), &**content).map_err(REPLIntrisicsError::FileWriteError)?;
    Ok(Value::Null(ValueNull))
}

fn print(
    data: &mut Data,
    params: Box<[Value<REPLIntrisics>]>,
) -> Result<Value<REPLIntrisics>, REPLIntrisicsError> {
    for value in params.iter() {
        match print_value(*data.graphic, &data.skin, value, false) {
            Ok(_) => (),
            Err(err) => {
                data.quitted = Quitted::Fatal(err);
                return Err(REPLIntrisicsError::Quitting);
            }
        };
        println!()
    }
    Ok(Value::Null(ValueNull))
}

fn quit(
    data: &mut Data,
    params: Box<[Value<REPLIntrisics>]>,
) -> Result<Value<REPLIntrisics>, REPLIntrisicsError> {
    data.quitted = Quitted::Yes(match Box::<[_; 1]>::try_from(params).map(|p| *p) {
        Ok([v]) => v,
        Err(params) if params.is_empty() => ValueNull.into(),
        Err(params) => ValueList::from_iter(params.into_vec()).into(),
    });
    Err(REPLIntrisicsError::Quitting)
}

/// The page for help about `help`
const HELP_PAGE_FOR_HELP: &str = "std/repl/help";

/// The help for `help` must exist as it is shown when calling `help` with invalid params
#[cfg(test)]
#[test]
fn help_for_help_exist() {
    assert!(dices_man::search(HELP_PAGE_FOR_HELP).is_some())
}

/// The manual must contains the pages relative to the *REPL* intrisics
#[cfg(test)]
#[test]
fn man_has_repl_intrisics() {
    dices_man::std_library_is_represented::<REPLIntrisics>()
}

#[cfg(test)]
#[test]
fn all_names_roundtrip() {
    use dices_ast::intrisics::Intrisic;

    for intrisic in Intrisic::<REPLIntrisics>::iter() {
        let name = intrisic.name();
        let named = Intrisic::<REPLIntrisics>::named(name).unwrap_or_else(|| {
            panic!(
                "Intrisic `{intrisic:?}` gave `{name}` as name, but `named` did not recognize it"
            )
        });
        assert_eq!(intrisic, named, "Intrisic `{name}` did not roundtrip")
    }
}

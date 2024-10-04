//! Intrisics for the REPL

use std::{
    fs, io,
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
use termimad::{crossterm::terminal, MadSkin};

use crate::{print_value, Graphic};

pub struct Data {
    // stuff needed to visualize the elements
    graphic: Rc<Graphic>,
    skin: Rc<MadSkin>,

    // mark if the repl was quitted
    quitted: Quitted,
}

pub enum Quitted {
    No,
    Yes(Value<REPLIntrisics>),
}

impl Data {
    pub fn new(graphic: Rc<Graphic>, skin: Rc<MadSkin>) -> Self {
        Self {
            graphic,
            skin,
            quitted: Quitted::No,
        }
    }

    pub fn quitted(&self) -> &Quitted {
        &self.quitted
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum REPLIntrisics {
    /// Print a value
    Print,
    /// Quit the repl
    Quit,
    /// Print a manual page
    Help,

    /// Get the system time
    Time,

    /// Read a file as a string
    FileRead,
    /// Write a string to a file
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

macro_rules! repetitive_impls {
    (
        $(
            $variant:ident <=> $str:literal
        ),*
    ) => {
        fn iter() -> impl IntoIterator<Item = Self> {
            [$(Self::$variant),*]
        }

        fn name(&self) -> &'static str {
            match self {
              $(
                Self::$variant => $str
              ),*
            }
            .into()
        }

        fn named(name: &str) -> Option<Self> {
            Some(match name {
                $(
                  $str => Self::$variant,
                )*
                _ => return None,
            })
        }
    };
}

impl InjectedIntr for REPLIntrisics {
    type Data = Data;
    type Error = REPLIntrisicsError;

    repetitive_impls! {
        Print <=> "print",
        Quit <=> "quit",
        Help <=> "help",
        Time <=> "time",
        FileRead <=> "file_read",
        FileWrite <=> "file_write"
    }

    fn std_paths(&self) -> &[&[&'static str]] {
        match self {
            REPLIntrisics::Print => {
                &[&["prelude", "print"] as &[&str], &["repl", "print"]] as &[&[&str]]
            }
            REPLIntrisics::Quit => &[&["prelude", "quit"] as &[&str], &["repl", "quit"]],
            REPLIntrisics::Help => &[&["prelude", "help"] as &[&str], &["repl", "help"]],
            REPLIntrisics::Time => &[&["prelude", "time"] as &[&str], &["sys", "time"]],
            REPLIntrisics::FileRead => &[&["sys", "files", "read"] as &[&str]],
            REPLIntrisics::FileWrite => &[&["sys", "files", "write"] as &[&str]],
        }
    }

    fn call(
        &self,
        data: &mut Self::Data,
        params: Box<[Value<Self>]>,
    ) -> Result<Value<Self>, Self::Error> {
        match self {
            REPLIntrisics::Print => {
                for value in params.iter() {
                    print_value(*data.graphic, &data.skin, value, false);
                    println!()
                }
                Ok(Value::Null(ValueNull))
            }
            REPLIntrisics::Quit => {
                data.quitted = Quitted::Yes(match Box::<[Value<Self>; 1]>::try_from(params) {
                    Ok(box [v]) => v,
                    Err(box []) => ValueNull.into(),
                    Err(params) => ValueList::from_iter(params.into_vec()).into(),
                });
                Err(REPLIntrisicsError::Quitting)
            }
            REPLIntrisics::Help => {
                // the help intrisic never fails, at most fall on her help page itself
                let topic = match &*params {
                    [] => "introduction",
                    [Value::String(s)] => &*s,
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
                let content = mdast2minimad::to_minimad(&*content)
                    .expect("All help pages should be convertible");
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
            REPLIntrisics::Time => Ok(Value::Number(
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs()
                    .into(),
            )),
            REPLIntrisics::FileRead => {
                let path = match Box::<[Value<Self>; 1]>::try_from(params) {
                    Ok(box [Value::String(path)]) => path,
                    _ => return Err(REPLIntrisicsError::FileReadUsage),
                };
                let content = fs::read_to_string(Path::new(&**path))
                    .map_err(REPLIntrisicsError::FileReadError)?;
                Ok(Value::String(content.into()))
            }
            REPLIntrisics::FileWrite => {
                let (path, content) = match Box::<[Value<Self>; 2]>::try_from(params) {
                    Ok(box [Value::String(path), Value::String(content)]) => (path, content),
                    _ => return Err(REPLIntrisicsError::FileWriteUsage),
                };
                fs::write(Path::new(&**path), &**content)
                    .map_err(REPLIntrisicsError::FileWriteError)?;
                Ok(Value::Null(ValueNull))
            }
        }
    }
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
        let named = Intrisic::<REPLIntrisics>::named(&name).expect(&format!(
            "Intrisic `{intrisic:?}` gave `{name}` as name, but `named` did not recognize it"
        ));
        assert_eq!(intrisic, named, "Intrisic `{name}` did not roundtrip")
    }
}

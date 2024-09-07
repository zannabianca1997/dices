//! Intrisics for the REPL

use std::{borrow::Cow, rc::Rc};

use derive_more::derive::{Display, Error};
use dices_ast::{
    intrisics::InjectedIntr,
    values::{Value, ValueList, ValueNull},
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
}
#[derive(Debug, Clone, Display, Error)]
pub enum REPLIntrisicsError {
    /// The `quit` intrisic was called
    Quitting,
}

impl InjectedIntr for REPLIntrisics {
    type Data = Data;

    type Error = REPLIntrisicsError;

    fn iter() -> impl IntoIterator<Item = Self> {
        [Self::Print, Self::Quit, Self::Help]
    }

    fn name(&self) -> std::borrow::Cow<str> {
        match self {
            REPLIntrisics::Print => "print",
            REPLIntrisics::Quit => "quit",
            REPLIntrisics::Help => "help",
        }
        .into()
    }

    fn std_paths(&self) -> impl IntoIterator<Item = std::borrow::Cow<[std::borrow::Cow<str>]>> {
        match self {
            REPLIntrisics::Print => [
                Cow::Borrowed(&[Cow::Borrowed("prelude"), Cow::Borrowed("print")] as _),
                Cow::Borrowed(&[Cow::Borrowed("repl"), Cow::Borrowed("print")] as _),
            ],
            REPLIntrisics::Quit => [
                Cow::Borrowed(&[Cow::Borrowed("prelude"), Cow::Borrowed("quit")] as _),
                Cow::Borrowed(&[Cow::Borrowed("repl"), Cow::Borrowed("quit")] as _),
            ],
            REPLIntrisics::Help => [
                Cow::Borrowed(&[Cow::Borrowed("prelude"), Cow::Borrowed("help")] as _),
                Cow::Borrowed(&[Cow::Borrowed("repl"), Cow::Borrowed("help")] as _),
            ],
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

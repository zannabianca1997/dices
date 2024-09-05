//! Intrisics for the REPL

use std::{borrow::Cow, rc::Rc};

use derive_more::derive::{Display, Error};
use dices_ast::{
    intrisics::InjectedIntr,
    values::{Value, ValueList, ValueNull},
};
use termimad::MadSkin;

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
        [Self::Print, Self::Quit]
    }

    fn name(&self) -> std::borrow::Cow<str> {
        match self {
            REPLIntrisics::Print => "print".into(),
            REPLIntrisics::Quit => "quit".into(),
        }
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
                    print_value(*data.graphic, &data.skin, value)
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
        }
    }
}

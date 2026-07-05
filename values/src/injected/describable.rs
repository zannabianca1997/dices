use std::fmt::{self, Display, Formatter};

use derive_more::Display;

use crate::injected::Injectable;

pub trait Describable {
    /// Human readable description of this object
    fn description(&self) -> impl Display + '_;
}

pub struct Description<'a>(pub(super) &'a dyn Injectable);

impl Display for Description<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.0.dyn_description(f)
    }
}

#[derive(Display)]
enum OptionalDescription<T> {
    #[display("{_0}")]
    Present(T),
    #[display("not injected")]
    Missing,
}

impl<T: Describable> Describable for Option<T> {
    fn description(&self) -> impl Display + '_ {
        match self.as_ref() {
            Some(t) => OptionalDescription::Present(t.description()),
            None => OptionalDescription::Missing,
        }
    }
}

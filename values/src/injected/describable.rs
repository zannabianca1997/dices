use std::fmt::{self, Display, Formatter};

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

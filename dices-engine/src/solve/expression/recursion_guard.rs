//! A type used to avoid type level recursion on trait solution
//!
//! When a library like `thiserror` or `derive_more` implements `Error` on a trait it adds
//! a trait constraint for each possible source `Source: Error`. This makes the trait evaluation
//! overflown when a parametrized error is used in the error source chain of himself.
//!
//! This module define a newtype `RecursionGuard<T>` that wraps the source and define error
//! for some `T`, skipping the constraint. While this is ugly, it's better than deriving `Error`
//! manually on 20-something variants enum.
//!
//! This whole ordeal can be removed when [issue 317](https://github.com/dtolnay/thiserror/issues/317)
//!  on `thiserror` will be resolved.

use std::{
    error::Error,
    fmt::{Debug, Display},
};

use derive_more::derive::{AsMut, AsRef, Constructor, From};
use dices_ast::intrisics::InjectedIntr;

use super::IntrisicError;

#[derive(AsRef, AsMut, From, Clone, Copy, Constructor)]
#[repr(transparent)]
pub struct RecursionGuard<T>(pub T);

impl<T> Debug for RecursionGuard<T>
where
    T: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}
impl<T> Display for RecursionGuard<T>
where
    T: Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl<Injected> Error for RecursionGuard<IntrisicError<Injected>>
where
    Injected: InjectedIntr + Debug + 'static,
{
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        <IntrisicError<Injected> as Error>::source(&self.0)
    }
}

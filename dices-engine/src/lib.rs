#![feature(assert_matches)]
#![feature(never_type)]
#![feature(iterator_try_collect)]
#![feature(box_patterns)]

pub use context::Context;
use dices_ast::values::Value;

pub mod context;

trait Solvable {
    type Error;

    fn solve<R>(&self, context: &mut Context<R>) -> Result<Value, Self::Error>;
}

mod solve;

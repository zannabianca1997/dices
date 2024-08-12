//! Trivial implementations of `Solvable` for Values

use dices_ast::values::*;

macro_rules! trivial_impl {
        ( $( $type:ty ),* ) => {
            $(
                impl crate::Solvable for $type {
                    type Error = !;

                    fn solve<R>(
                        &self,
                        _context: &mut crate::Context<R>,
                    ) -> Result<Value, Self::Error> {
                        Ok(self.clone().into())
                    }
                }
            )*
        };
    }

trivial_impl!(
    Value,
    ValueBool,
    ValueIntrisic,
    ValueList,
    ValueMap,
    ValueNumber,
    ValueString,
    ValueNull
);

impl crate::Solvable for ValueClosure {
    type Error = !;

    fn solve<R>(&self, _context: &mut crate::Context<R>) -> Result<Value, Self::Error> {
        Ok(Box::new(self.clone()).into())
    }
}

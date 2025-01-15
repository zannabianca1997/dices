//! Trivial implementations of `Solvable` for Values

use dices_ast::{intrisics::InjectedIntr, value::{Value, ValueBool, ValueClosure, ValueIntrisic, ValueList, ValueMap, ValueNull, ValueNumber, ValueString}};

macro_rules! trivial_impl {
        ( $( $type:ty ),* ) => {
            $(
                impl<InjectedIntrisic: InjectedIntr> crate::solve::Solvable<InjectedIntrisic> for $type {
                    type Error = std::convert::Infallible;

                    fn solve<R>(
                        &self,
                        _context: &mut crate::Context<R, InjectedIntrisic, InjectedIntrisic::Data>,
                    ) -> Result<Value<InjectedIntrisic>, Self::Error> {
                        Ok(self.clone().into())
                    }
                }
            )*
        };
    }

trivial_impl!(
    Value<InjectedIntrisic>,
    ValueBool,
    ValueIntrisic<InjectedIntrisic>,
    ValueList<InjectedIntrisic>,
    ValueMap<InjectedIntrisic>,
    ValueNumber,
    ValueString,
    ValueNull
);

impl<InjectedIntrisic: InjectedIntr> crate::solve::Solvable<InjectedIntrisic>
    for ValueClosure<InjectedIntrisic>
{
    type Error = std::convert::Infallible;

    fn solve<R>(
        &self,
        _context: &mut crate::Context<R, InjectedIntrisic, InjectedIntrisic::Data>,
    ) -> Result<Value<InjectedIntrisic>, Self::Error> {
        Ok(Box::new(self.clone()).into())
    }
}

use std::ops::{Deref, DerefMut};

use derive_more::derive::Into;
use nunny::NonEmpty;

use super::Expression;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Into)]
pub struct ExpressionScope<InjectedIntrisic>(Box<NonEmpty<[Expression<InjectedIntrisic>]>>);

impl<InjectedIntrisic> Deref for ExpressionScope<InjectedIntrisic> {
    type Target = NonEmpty<[Expression<InjectedIntrisic>]>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<InjectedIntrisic> DerefMut for ExpressionScope<InjectedIntrisic> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<InjectedIntrisic> ExpressionScope<InjectedIntrisic> {
    pub fn new(exprs: Box<NonEmpty<[Expression<InjectedIntrisic>]>>) -> Self {
        Self(exprs)
    }
}

#[cfg(feature = "bincode")]
impl<InjectedIntrisic> bincode::Encode for ExpressionScope<InjectedIntrisic>
where
    InjectedIntrisic: crate::intrisics::InjectedIntr,
{
    fn encode<E: bincode::enc::Encoder>(
        &self,
        encoder: &mut E,
    ) -> Result<(), bincode::error::EncodeError> {
        let inner: &[Expression<InjectedIntrisic>] = &self.0;
        inner.encode(encoder)
    }
}
#[cfg(feature = "bincode")]
impl<InjectedIntrisic> bincode::Decode for ExpressionScope<InjectedIntrisic>
where
    InjectedIntrisic: crate::intrisics::InjectedIntr,
{
    fn decode<D: bincode::de::Decoder>(
        decoder: &mut D,
    ) -> Result<Self, bincode::error::DecodeError> {
        let inner: Box<[Expression<InjectedIntrisic>]> = bincode::Decode::decode(decoder)?;
        Ok(Self(
            nunny::Vec::new(inner.into_vec())
                .map_err(|_| bincode::error::DecodeError::Other("Invalid empty scope"))?
                .into_boxed_slice(),
        ))
    }
}
#[cfg(feature = "bincode")]
impl<'de, InjectedIntrisic> bincode::BorrowDecode<'de> for ExpressionScope<InjectedIntrisic>
where
    InjectedIntrisic: crate::intrisics::InjectedIntr,
{
    fn borrow_decode<D: bincode::de::BorrowDecoder<'de>>(
        decoder: &mut D,
    ) -> Result<Self, bincode::error::DecodeError> {
        let inner: Box<[Expression<InjectedIntrisic>]> =
            bincode::BorrowDecode::borrow_decode(decoder)?;
        Ok(Self(
            nunny::Vec::new(inner.into_vec())
                .map_err(|_| bincode::error::DecodeError::Other("Invalid empty scope"))?
                .into_boxed_slice(),
        ))
    }
}

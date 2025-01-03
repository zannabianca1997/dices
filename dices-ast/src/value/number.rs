use std::iter::Step;

use derive_more::derive::{
    Add, AddAssign, Display, Div, DivAssign, Error, From, Into, Mul, MulAssign, Neg, Rem,
    RemAssign, Sub, SubAssign,
};
use num_bigint::{BigInt, ToBigInt};

use super::list::ValueList;

/// A signed integer value
#[derive(
    // display helper
    Debug,
    Display,
    // cloning
    Clone,
    // comparisons
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    // number operations
    Add,
    AddAssign,
    Sub,
    SubAssign,
    Mul,
    MulAssign,
    Div,
    DivAssign,
    Neg,
    Rem,
    RemAssign,
    From,
    Into,
)]
#[mul(forward)]
#[rem(forward)]
#[div(forward)]
pub struct ValueNumber(pub(super) BigInt);

impl ValueNumber {
    pub const ZERO: Self = ValueNumber(BigInt::ZERO);

    pub fn to_number(self) -> Result<ValueNumber, super::ToNumberError> {
        Ok(self)
    }

    pub fn to_list<InjectedIntrisic>(
        self,
    ) -> Result<ValueList<InjectedIntrisic>, super::ToListError> {
        Ok(ValueList::from_iter([self.into()]))
    }

    pub fn abs(self) -> Self {
        Self(BigInt::from_biguint(
            num_bigint::Sign::Plus,
            self.0.into_parts().1,
        ))
    }
}
macro_rules! impl_lesser_nums {
    ( $( $n:ty ) *) => {
        $(
            impl From<$n> for ValueNumber {
                fn from(value: $n) -> Self {
                    ValueNumber(value.into())
                }
            }

            impl TryFrom<ValueNumber> for $n {
                type Error = num_bigint::TryFromBigIntError<BigInt>;

                fn try_from(ValueNumber(value): ValueNumber) -> Result<Self, Self::Error> {
                    value.try_into()
                }
            }
        )*
    };
}
impl_lesser_nums! {i8 u8 i16 u16 i32 u32 i64 u64 i128 u128 isize usize}
#[derive(Debug, Clone, Copy, Error, Display)]
#[display("The float {_0} is too big to be represented")]
pub struct FloatTooBig<F>(#[error(not(source))] F);

macro_rules! impl_floating_nums {
    ( $( $n:ty ) *) => {
        $(
            impl TryFrom<$n> for ValueNumber {
                type Error = FloatTooBig<$n>;

                fn try_from(value: $n) -> Result<Self, Self::Error> {
                    value.to_bigint().map(ValueNumber).ok_or(FloatTooBig(value))
                }
            }
        )*
    };
}
impl_floating_nums! {f32 f64}

impl Step for ValueNumber {
    fn steps_between(start: &Self, end: &Self) -> (usize, Option<usize>) {
        match (&end.0 - &start.0).try_into() {
            Ok(n) => (n, Some(n)),
            Err(_) => (usize::MAX, None),
        }
    }

    fn forward_checked(start: Self, count: usize) -> Option<Self> {
        Some(Self(start.0 + count))
    }

    fn backward_checked(start: Self, count: usize) -> Option<Self> {
        Some(Self(start.0 - count))
    }
}

#[cfg(feature = "pretty")]
impl<'a, D, A> pretty::Pretty<'a, D, A> for &'a ValueNumber
where
    A: 'a,
    D: ?Sized + pretty::DocAllocator<'a, A>,
{
    fn pretty(self, allocator: &'a D) -> pretty::DocBuilder<'a, D, A> {
        allocator.text(self.to_string())
    }
}

#[cfg(feature = "rand")]
impl rand::distributions::uniform::SampleUniform for ValueNumber {
    type Sampler = ValueNumberSampler;
}
#[cfg(feature = "rand")]
pub struct ValueNumberSampler(<BigInt as rand::distributions::uniform::SampleUniform>::Sampler);
#[cfg(feature = "rand")]
impl rand::distributions::uniform::UniformSampler for ValueNumberSampler {
    type X = ValueNumber;

    fn new<B1, B2>(low: B1, high: B2) -> Self
    where
        B1: rand::distributions::uniform::SampleBorrow<Self::X> + Sized,
        B2: rand::distributions::uniform::SampleBorrow<Self::X> + Sized,
    {
        Self(
            <BigInt as rand::distributions::uniform::SampleUniform>::Sampler::new(
                &low.borrow().0,
                &high.borrow().0,
            ),
        )
    }

    fn new_inclusive<B1, B2>(low: B1, high: B2) -> Self
    where
        B1: rand::distributions::uniform::SampleBorrow<Self::X> + Sized,
        B2: rand::distributions::uniform::SampleBorrow<Self::X> + Sized,
    {
        Self(
            <BigInt as rand::distributions::uniform::SampleUniform>::Sampler::new_inclusive(
                &low.borrow().0,
                &high.borrow().0,
            ),
        )
    }

    fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> Self::X {
        ValueNumber(self.0.sample(rng))
    }
}

#[cfg(feature = "bincode")]
impl bincode::Encode for ValueNumber {
    fn encode<E: bincode::enc::Encoder>(
        &self,
        encoder: &mut E,
    ) -> Result<(), bincode::error::EncodeError> {
        self.0.to_signed_bytes_le().encode(encoder)
    }
}
#[cfg(feature = "bincode")]
impl bincode::Decode for ValueNumber {
    fn decode<D: bincode::de::Decoder>(
        decoder: &mut D,
    ) -> Result<Self, bincode::error::DecodeError> {
        Vec::decode(decoder).map(|digits| Self(BigInt::from_signed_bytes_le(&digits)))
    }
}
#[cfg(feature = "bincode")]
bincode::impl_borrow_decode! {ValueNumber}

#[cfg(feature = "serde")]
mod serde {

    use num_bigint::{BigInt, Sign};
    use serde::{Deserialize, Serialize};
    use serde_bytes::ByteBuf;

    use super::ValueNumber;

    #[derive(Deserialize, Serialize)]
    #[serde(tag = "$type")]
    enum Serialized {
        #[serde(rename = "number")]
        Nested {
            #[serde(rename = "$sign")]
            sign: Sign,
            #[serde(rename = "$bytes")]
            bytes: ByteBuf,
        },
        #[serde(untagged)]
        Small(i64),
    }

    impl Serialize for ValueNumber {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            match i64::try_from(&self.0) {
                Ok(small) => Serialized::Small(small),
                Err(_) => {
                    let (sign, bytes) = self.0.to_bytes_le();
                    Serialized::Nested {
                        sign,
                        bytes: ByteBuf::from(bytes),
                    }
                }
            }
            .serialize(serializer)
        }
    }
    impl<'de> Deserialize<'de> for ValueNumber {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            let value = match Serialized::deserialize(deserializer)? {
                Serialized::Nested { sign, bytes } => BigInt::from_bytes_le(sign, &*bytes),
                Serialized::Small(small) => small.into(),
            };
            Ok(ValueNumber(value))
        }
    }
}

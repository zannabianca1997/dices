//! List of the language intrisics

use std::{
    borrow::Cow,
    error::Error,
    fmt::{Debug, Display},
    hash::Hash,
    ops::{Deref, DerefMut},
};

use crate::value::{map::ValueMap, Value, ValueIntrisic};

#[derive(
    // display helper
    Debug,
    // cloning
    Clone,
    Copy,
    // comparisons
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
)]
pub enum Intrisic<Injected> {
    /// `+`: sum multiple values, flattening lists,maps, and converting booleans
    Sum,
    /// `~`: join multiple list or maps, upgrading scalars to list if joined to a list
    Join,

    /// `*`: multiply multiple values, distributing scalars over lists or maps
    Mult,

    /// Try to convert a value to a number
    ToNumber,
    /// Try to convert a value to a list
    ToList,
    /// Convert a value to a string
    ToString,
    /// Parse a string into a value
    Parse,

    /// Call its first parameter with the arguments given by the second, converted to a list
    Call,

    /// Convert its param to a json string
    ToJson,
    /// Convert its param from a json string
    FromJson,

    /// Seed the RNG
    SeedRNG,
    /// Save the RNG state
    SaveRNG,
    /// Restore the RNG state
    RestoreRNG,

    /// Injected intrisic
    ///
    /// Intrisics that came from the enviroment (files, printing, exiting the shell, etc)
    Injected(Injected),
}

macro_rules! repetitive_impl {
    (
        $(
            $variant:ident <=> $str:literal
        ),*
    ) => {
        impl<Injected> Intrisic<Injected>
        where
            Injected: InjectedIntr,
        {
            /// Iter all possible intrisics
            pub fn iter() -> impl IntoIterator<Item = Self> {
                [
                    $(
                        Self::$variant
                    ),*
                ]
                .into_iter()
                .chain(Injected::iter().into_iter().map(Self::Injected))
            }

            pub fn name(&self) -> &'static str {
                match self {
                    $(
                        Self::$variant => $str.into(),
                    )*
                    Intrisic::Injected(injected) => injected.name(),
                }
            }

            pub fn named(name: &str) -> Option<Self> {
                Some(match name {
                    $(
                        $str => Self::$variant,
                    )*
                    _ => return Injected::named(name).map(Intrisic::Injected),
                })
            }
        }
        impl Intrisic<NoInjectedIntrisics> {
            pub fn with_arbitrary_injected_intrisics<II>(self) -> Intrisic<II> {
                match self {
                    $(
                        Intrisic::$variant => Intrisic::$variant,
                    )*
                    // This last case never happens
                    Intrisic::Injected(injected) => *injected,
                }
            }
        }
    };
}

repetitive_impl! {
    Sum <=> "sum",
    Join <=> "join",
    Mult <=> "mult",
    ToNumber <=> "to_number",
    ToList <=> "to_list",
    ToString <=> "to_string",
    Parse <=> "parse",
    Call <=> "call",
    ToJson <=> "to_json",
    FromJson <=> "from_json",
    SeedRNG <=> "seed_rng",
    SaveRNG <=> "save_rng",
    RestoreRNG <=> "restore_rng"
}

impl<Injected> Intrisic<Injected>
where
    Injected: InjectedIntr,
{
    /// Build a module containing all the intrisics, to include in the standard library
    pub fn all() -> ValueMap<Injected> {
        ValueMap::from_iter(Self::iter().into_iter().map(|v| {
            (
                v.name().to_string().into_boxed_str().into(),
                ValueIntrisic::from(v).into(),
            )
        }))
    }
}

#[cfg(test)]
#[test]
fn all_names_roundtrip() {
    for intrisic in Intrisic::<NoInjectedIntrisics>::iter() {
        let name = intrisic.name();
        let named = Intrisic::<NoInjectedIntrisics>::named(&name).expect(&format!(
            "Intrisic `{intrisic:?}` gave `{name}` as name, but `named` did not recognize it"
        ));
        assert_eq!(intrisic, named, "Intrisic `{name}` did not roundtrip")
    }
}

pub trait InjectedIntr: Sized + Clone + 'static + Hash {
    /// The data used by the injected intrisics
    type Data;
    /// The error type given by calling this intrisic
    type Error: Error + 'static;

    /// Iter all possible intrisics that must be injected
    fn iter() -> impl IntoIterator<Item = Self>;
    /// Give a name for this intrisic
    fn name(&self) -> &'static str;
    /// Get the intrisic from the name
    fn named(name: &str) -> Option<Self>;
    /// Give all the paths in the std library this intrisic should be injected to
    fn std_paths(&self) -> &[&[&'static str]] {
        // default to not injecting anywhere
        &[]
    }
    /// Call this intrisic
    fn call<'d>(
        &self,
        data: &mut Self::Data,
        params: Box<[Value<Self>]>,
    ) -> Result<Value<Self>, Self::Error>;
}

/// No injected intrisics
#[derive(Clone, Copy)]
pub struct NoInjectedIntrisics(!);

impl Deref for NoInjectedIntrisics {
    type Target = !;

    fn deref(&self) -> &Self::Target {
        self.0
    }
}
impl DerefMut for NoInjectedIntrisics {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0
    }
}

impl Debug for NoInjectedIntrisics {
    fn fmt(&self, _: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0
    }
}
impl Display for NoInjectedIntrisics {
    fn fmt(&self, _: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0
    }
}
impl PartialEq for NoInjectedIntrisics {
    fn eq(&self, _: &Self) -> bool {
        self.0
    }
}
impl Eq for NoInjectedIntrisics {}
impl PartialOrd for NoInjectedIntrisics {
    fn partial_cmp(&self, _: &Self) -> Option<std::cmp::Ordering> {
        self.0
    }
}
impl Ord for NoInjectedIntrisics {
    fn cmp(&self, _: &Self) -> std::cmp::Ordering {
        self.0
    }
}
impl Hash for NoInjectedIntrisics {
    fn hash<H: std::hash::Hasher>(&self, _: &mut H) {
        self.0
    }
}

impl InjectedIntr for NoInjectedIntrisics {
    type Data = ();
    type Error = !;

    fn iter() -> impl IntoIterator<Item = Self> {
        []
    }

    fn name(&self) -> &'static str {
        self.0
    }

    fn call<'d>(
        &self,
        _: &mut Self::Data,
        _: Box<[Value<Self>]>,
    ) -> Result<Value<Self>, Self::Error> {
        self.0
    }

    fn named(_: &str) -> Option<Self> {
        None
    }
}

#[cfg(feature = "bincode")]
impl<II> bincode::Encode for Intrisic<II>
where
    II: InjectedIntr,
{
    fn encode<E: bincode::enc::Encoder>(
        &self,
        encoder: &mut E,
    ) -> Result<(), bincode::error::EncodeError> {
        self.name().encode(encoder)
    }
}

#[cfg(feature = "bincode")]
impl<II> bincode::Decode for Intrisic<II>
where
    II: InjectedIntr,
{
    fn decode<D: bincode::de::Decoder>(
        decoder: &mut D,
    ) -> Result<Self, bincode::error::DecodeError> {
        let name: Cow<str> = bincode::Decode::decode(decoder)?;
        Self::named(&*name).ok_or_else(|| {
            bincode::error::DecodeError::OtherString(format!("Unknow intrisic {name}"))
        })
    }
}

#[cfg(feature = "bincode")]
impl<'de, II> bincode::BorrowDecode<'de> for Intrisic<II>
where
    II: InjectedIntr,
{
    fn borrow_decode<D: bincode::de::BorrowDecoder<'de>>(
        decoder: &mut D,
    ) -> Result<Self, bincode::error::DecodeError> {
        let name: Cow<str> = bincode::BorrowDecode::borrow_decode(decoder)?;
        Self::named(&*name).ok_or_else(|| {
            bincode::error::DecodeError::OtherString(format!("Unknow intrisic {name}"))
        })
    }
}

#[cfg(feature = "serde")]
mod serde {
    use std::borrow::Cow;

    use serde::{Deserialize, Serialize};

    use super::Intrisic;
    use crate::intrisics::InjectedIntr;

    impl<II> Serialize for Intrisic<II>
    where
        II: InjectedIntr,
    {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            self.name().serialize(serializer)
        }
    }
    impl<'de, II> Deserialize<'de> for Intrisic<II>
    where
        II: InjectedIntr,
    {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            let name = Cow::<str>::deserialize(deserializer)?;
            Self::named(&name).ok_or_else(|| {
                <D::Error as serde::de::Error>::custom(format!("Unknow intrisic {name}"))
            })
        }
    }
}

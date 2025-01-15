//! Context essential to evaluate a `dices` expression

use std::{collections::BTreeMap, fmt::Debug, mem};

use dices_ast::{ident::IdentStr, value::Value};
use nunny::NonEmpty;

type Scope<InjectedIntrisic> = BTreeMap<Box<IdentStr>, Value<InjectedIntrisic>>;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Context<RNG, InjectedIntrisic, InjectedIntrisicData> {
    /// the stack of variables
    scopes: NonEmpty<Vec<Scope<InjectedIntrisic>>>,
    /// The random number generator
    rng: RNG,
    /// The data for the injected intrisics
    injected_intrisics_data: InjectedIntrisicData,
}

#[cfg(feature = "bincode")]
impl<RNG, InjectedIntrisic, InjectedIntrisicData> bincode::Encode
    for Context<RNG, InjectedIntrisic, InjectedIntrisicData>
where
    RNG: serde::Serialize,
    Value<InjectedIntrisic>: bincode::Encode + 'static,
    InjectedIntrisicData: bincode::Encode,
{
    fn encode<E: bincode::enc::Encoder>(
        &self,
        encoder: &mut E,
    ) -> core::result::Result<(), bincode::error::EncodeError> {
        bincode::Encode::encode(self.scopes.as_slice(), encoder)?;
        bincode::Encode::encode(&bincode::serde::Compat(&self.rng), encoder)?;
        bincode::Encode::encode(&self.injected_intrisics_data, encoder)?;
        Ok(())
    }
}

#[cfg(feature = "bincode")]
impl<RNG, InjectedIntrisic, InjectedIntrisicData> bincode::Decode
    for Context<RNG, InjectedIntrisic, InjectedIntrisicData>
where
    RNG: serde::de::DeserializeOwned,
    Value<InjectedIntrisic>: bincode::Decode + 'static,
    InjectedIntrisicData: bincode::Decode,
{
    fn decode<D: bincode::de::Decoder>(
        decoder: &mut D,
    ) -> core::result::Result<Self, bincode::error::DecodeError> {
        Ok(Self {
            scopes: NonEmpty::<Vec<_>>::new(bincode::Decode::decode(decoder)?)
                .map_err(|_| bincode::error::DecodeError::Other("Empty scopes stack"))?,
            rng: bincode::serde::Compat::decode(decoder)?.0,
            injected_intrisics_data: bincode::Decode::decode(decoder)?,
        })
    }
}

#[cfg(feature = "bincode")]
impl<'de, RNG, InjectedIntrisic, InjectedIntrisicData> bincode::BorrowDecode<'de>
    for Context<RNG, InjectedIntrisic, InjectedIntrisicData>
where
    RNG: serde::de::Deserialize<'de>,
    Value<InjectedIntrisic>: bincode::de::BorrowDecode<'de>,
    InjectedIntrisicData: bincode::de::BorrowDecode<'de>,
{
    fn borrow_decode<D: bincode::de::BorrowDecoder<'de>>(
        decoder: &mut D,
    ) -> core::result::Result<Self, bincode::error::DecodeError> {
        Ok(Self {
            scopes: NonEmpty::<Vec<_>>::new(bincode::BorrowDecode::borrow_decode(decoder)?)
                .map_err(|_| bincode::error::DecodeError::Other("Empty scopes stack"))?,
            rng: bincode::serde::BorrowCompat::borrow_decode(decoder)?.0,
            injected_intrisics_data: bincode::BorrowDecode::borrow_decode(decoder)?,
        })
    }
}

impl<RNG, InjectedIntrisic, InjectedIntrisicData>
    Context<RNG, InjectedIntrisic, InjectedIntrisicData>
{
    pub fn new(rng: RNG, injected_intrisics_data: InjectedIntrisicData) -> Self {
        Self {
            scopes: nunny::vec![Scope::new()],
            rng,
            injected_intrisics_data,
        }
    }

    /// run code in a local scope, with the same RNG and no local variables
    pub fn scoped<R>(&mut self, f: impl FnOnce(&mut Self) -> R) -> R {
        self.scopes.push(Scope::new());
        let res = f(self);
        unsafe {
            // SAFETY: pushing and popping is balanced.
            // We just pushed on a non empty vector, so we can
            // pop without emptying it.
            self.scopes.as_mut_vec().pop()
        };
        res
    }

    /// run code in a jail, with the same RNG but no variables
    pub fn jailed<R>(&mut self, f: impl FnOnce(&mut Self) -> R) -> R {
        let old_scopes = mem::replace(&mut self.scopes, nunny::vec![Scope::new()]);
        let res = f(self);
        self.scopes = old_scopes;
        res
    }

    /// Obtain a readonly handle to the variables
    pub fn vars(&self) -> Vars<InjectedIntrisic> {
        Vars(&self.scopes)
    }
    /// Obtain an handle to the variables
    pub fn vars_mut(&mut self) -> VarsMut<InjectedIntrisic> {
        VarsMut(&mut self.scopes)
    }

    /// Obtain an handle to the rng
    pub fn rng(&mut self) -> &mut RNG {
        &mut self.rng
    }

    /// Handler to the data used by the injected intrisics
    pub const fn injected_intrisics_data(&self) -> &InjectedIntrisicData {
        &self.injected_intrisics_data
    }

    /// Mutable handler to the data used by the injected intrisics
    pub fn injected_intrisics_data_mut(&mut self) -> &mut InjectedIntrisicData {
        &mut self.injected_intrisics_data
    }

    /// Obtain all the separate mutable handlers
    ///
    /// This enable an algorithm to keep mutable handlers to the non overlapping parts of the context
    pub fn handlers_mut(
        &mut self,
    ) -> (
        VarsMut<InjectedIntrisic>,
        &mut RNG,
        &mut InjectedIntrisicData,
    ) {
        let Self {
            scopes,
            rng,
            injected_intrisics_data,
        } = self;
        (VarsMut(scopes), rng, injected_intrisics_data)
    }

    pub(crate) fn map_injected_intrisics_data<NewInjectedIntrisicData>(
        self,
        f: impl FnOnce(InjectedIntrisicData) -> NewInjectedIntrisicData,
    ) -> Context<RNG, InjectedIntrisic, NewInjectedIntrisicData> {
        let Self {
            scopes,
            rng,
            injected_intrisics_data,
        } = self;
        Context {
            scopes,
            rng,
            injected_intrisics_data: f(injected_intrisics_data),
        }
    }
}

#[derive(Debug, Clone, Copy)]
/// Readonly vars handler
///
/// This is a handler that enable one to read the variable values
pub struct Vars<'c, InjectedIntrisic>(&'c NonEmpty<[Scope<InjectedIntrisic>]>);

impl<InjectedIntrisic> Vars<'_, InjectedIntrisic> {
    /// Find the value of a variable
    pub fn get(&self, name: &IdentStr) -> Option<&Value<InjectedIntrisic>> {
        // find the last scope that contains that variable
        self.0.iter().rev().find_map(|s| s.get(name))
    }
}
impl<'c, InjectedIntrisic> From<VarsMut<'c, InjectedIntrisic>> for Vars<'c, InjectedIntrisic> {
    fn from(value: VarsMut<'c, InjectedIntrisic>) -> Self {
        Self(&*value.0)
    }
}
impl<'c, InjectedIntrisic> From<&'c VarsMut<'c, InjectedIntrisic>> for Vars<'c, InjectedIntrisic> {
    fn from(value: &'c VarsMut<'c, InjectedIntrisic>) -> Self {
        Self(&*value.0)
    }
}
impl<'c, InjectedIntrisic> From<&'c mut VarsMut<'c, InjectedIntrisic>>
    for Vars<'c, InjectedIntrisic>
{
    fn from(value: &'c mut VarsMut<'c, InjectedIntrisic>) -> Self {
        Self(&*value.0)
    }
}

#[derive(Debug)]
/// Mutable vars handles
///
/// This is a handler that enable one to read and modify the variable values
pub struct VarsMut<'c, InjectedIntrisic>(&'c mut NonEmpty<[Scope<InjectedIntrisic>]>);

impl<InjectedIntrisic> VarsMut<'_, InjectedIntrisic> {
    /// Let a variable be, setting its value if present in the current scope, or creating it
    pub fn let_(&mut self, name: Box<IdentStr>, value: Value<InjectedIntrisic>) {
        self.0.last_mut().insert(name, value);
    }
    /// Find the value of a variable
    pub fn get(&self, name: &IdentStr) -> Option<&Value<InjectedIntrisic>> {
        // find the last scope that contains that variable
        self.0.iter().rev().find_map(|s| s.get(name))
    }
    /// Find the value of a variable, and permit to modify it
    pub fn get_mut(&mut self, name: &IdentStr) -> Option<&mut Value<InjectedIntrisic>> {
        // find the last scope that contains that variable
        self.0.iter_mut().rev().find_map(|s| s.get_mut(name))
    }
}

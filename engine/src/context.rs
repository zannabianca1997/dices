//! Context essential to evaluate a `dices` expression

use std::{collections::BTreeMap, fmt::Debug, mem};

use dices_ast::{ident::IdentStr, intrisics::InjectedIntr, value::Value};
use nunny::NonEmpty;

type Scope<InjectedIntrisic> = BTreeMap<Box<IdentStr>, Value<InjectedIntrisic>>;

pub struct Context<RNG, InjectedIntrisic: InjectedIntr> {
    /// the stack of variables
    scopes: NonEmpty<Vec<Scope<InjectedIntrisic>>>,
    /// The random number generator
    rng: RNG,
    /// The data for the injected intrisics
    injected_intrisics_data: <InjectedIntrisic as InjectedIntr>::Data,
}

#[cfg(feature = "bincode")]
impl<RNG, InjectedIntrisic: InjectedIntr> bincode::Encode for Context<RNG, InjectedIntrisic>
where
    RNG: serde::Serialize,
    InjectedIntrisic: bincode::Encode,
    InjectedIntrisic::Data: bincode::Encode,
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
impl<RNG, InjectedIntrisic: InjectedIntr> bincode::Decode for Context<RNG, InjectedIntrisic>
where
    RNG: serde::de::DeserializeOwned,
    InjectedIntrisic: bincode::Decode,
    InjectedIntrisic::Data: bincode::Decode,
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
impl<'de, RNG, InjectedIntrisic: InjectedIntr> bincode::BorrowDecode<'de>
    for Context<RNG, InjectedIntrisic>
where
    RNG: serde::de::Deserialize<'de>,
    InjectedIntrisic: bincode::de::BorrowDecode<'de>,
    InjectedIntrisic::Data: bincode::de::BorrowDecode<'de>,
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

impl<RNG: std::fmt::Debug, InjectedIntrisic: InjectedIntr + std::fmt::Debug> std::fmt::Debug
    for Context<RNG, InjectedIntrisic>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let debuggable_data: &dyn Debug = match InjectedIntrisic::data_debug_fmt() {
            Some(debug_data_fn) => {
                struct DebuggableDataWrapper<'d, II>(
                    &'d II,
                    fn(&II, &mut std::fmt::Formatter<'_>) -> std::fmt::Result,
                );
                impl<II> Debug for DebuggableDataWrapper<'_, II> {
                    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        (self.1)(self.0, f)
                    }
                }

                &DebuggableDataWrapper(&self.injected_intrisics_data, debug_data_fn)
            }
            None => &"...",
        };

        f.debug_struct("Context")
            .field("scopes", &self.scopes)
            .field("rng", &self.rng)
            .field("injected_intrisics_data", debuggable_data)
            .finish()
    }
}

impl<RNG: Clone, InjectedIntrisic: InjectedIntr + Clone> Clone for Context<RNG, InjectedIntrisic>
where
    <InjectedIntrisic as dices_ast::intrisics::InjectedIntr>::Data: Clone,
{
    fn clone(&self) -> Self {
        Self {
            scopes: self.scopes.clone(),
            rng: self.rng.clone(),
            injected_intrisics_data: self.injected_intrisics_data.clone(),
        }
    }
}

impl<RNG, InjectedIntrisic: InjectedIntr> Context<RNG, InjectedIntrisic> {
    pub fn new(
        rng: RNG,
        injected_intrisics_data: <InjectedIntrisic as InjectedIntr>::Data,
    ) -> Self {
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

    pub fn injected_intrisics_data(&self) -> &<InjectedIntrisic as InjectedIntr>::Data {
        &self.injected_intrisics_data
    }

    pub fn injected_intrisics_data_mut(&mut self) -> &mut <InjectedIntrisic as InjectedIntr>::Data {
        &mut self.injected_intrisics_data
    }
}

#[derive(Debug, Clone, Copy)]
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

#![feature(assert_matches)]
#![feature(never_type)]
#![feature(iterator_try_collect)]
#![feature(iterator_try_reduce)]
#![feature(box_patterns)]
#![feature(type_changing_struct_update)]

use std::borrow::Cow;

use dices_version::Version;
use nunny::NonEmpty;
use rand::{Rng, SeedableRng};

use dices_ast::{
    ident::IdentStr,
    intrisics::{InjectedIntr, NoInjectedIntrisics},
    Expression, Value,
};

use serde::{de::DeserializeOwned, Serialize};
use solve::{solve_multiple, Solvable};

pub use context::Context;
pub use dices_std::std as dices_std;
pub use solve::{IntrisicError, SolveError};

mod context;
mod dices_std;
mod solve;

pub struct EngineBuilder<RNG = (), InjectedIntrisic: InjectedIntr = NoInjectedIntrisics> {
    rng: RNG,
    std: Option<Cow<'static, IdentStr>>,
    prelude: bool,
    injected_intrisics_data: <InjectedIntrisic as InjectedIntr>::Data,
}
impl EngineBuilder<(), NoInjectedIntrisics> {
    /// Start building a new engine
    pub fn new() -> Self {
        Self {
            rng: (),
            std: Some(Cow::Borrowed(IdentStr::new("std").unwrap())),
            prelude: true,
            injected_intrisics_data: (),
        }
    }
}
impl Default for EngineBuilder<(), NoInjectedIntrisics> {
    fn default() -> Self {
        Self::new()
    }
}
impl<RNG, InjectedIntrisic: InjectedIntr> EngineBuilder<RNG, InjectedIntrisic> {
    /// Add an RNG
    pub fn with_rng<NewRNG>(self, rng: NewRNG) -> EngineBuilder<NewRNG, InjectedIntrisic> {
        EngineBuilder { rng, ..self }
    }

    /// Add an RNG, seeding it from entropy
    pub fn with_rng_from_entropy<NewRNG>(self) -> EngineBuilder<NewRNG, InjectedIntrisic>
    where
        NewRNG: SeedableRng,
    {
        EngineBuilder {
            rng: NewRNG::from_entropy(),
            ..self
        }
    }

    /// Inject the intrisics
    pub fn inject_intrisics<NewInjected: InjectedIntr>(self) -> EngineBuilder<RNG, NewInjected>
    where
        NewInjected::Data: Default,
    {
        EngineBuilder {
            injected_intrisics_data: Default::default(),
            ..self
        }
    }

    /// Inject the intrisics with data
    pub fn inject_intrisics_with_data<NewInjected: InjectedIntr>(
        self,
        data: NewInjected::Data,
    ) -> EngineBuilder<RNG, NewInjected> {
        EngineBuilder {
            injected_intrisics_data: data,
            ..self
        }
    }

    /// Put the std library in the engine
    pub fn with_std(self) -> Self {
        Self {
            std: Some(Cow::Borrowed(IdentStr::new("std").unwrap())),
            ..self
        }
    }

    /// Put the std library in the engine, with a different name
    ///
    /// This will make most script break
    pub fn with_std_named(self, name: impl Into<Cow<'static, IdentStr>>) -> Self {
        Self {
            std: Some(name.into()),
            ..self
        }
    }

    /// Do not put the std library in the engine
    ///
    /// This will make most of the intrisics unreachable
    pub fn without_std(self) -> Self {
        Self { std: None, ..self }
    }

    /// Import the prelude in the engine
    pub fn with_prelude(self) -> Self {
        Self {
            prelude: true,
            ..self
        }
    }

    /// Do not import the prelude in the engine
    ///
    /// This will make some script break
    pub fn without_prelude(self) -> Self {
        Self {
            prelude: false,
            ..self
        }
    }

    /// Build the engine
    pub fn build(self) -> Engine<RNG, InjectedIntrisic>
    where
        InjectedIntrisic: Clone,
    {
        let Self {
            rng,
            std,
            prelude,
            injected_intrisics_data,
        } = self;
        // build context
        let mut context = Context::new(rng, injected_intrisics_data);
        // adding std and prelude
        if let Some(std_name) = std {
            // generating the std library
            let std = dices_std::std();
            // adding the prelude
            if prelude {
                let Some(Value::Map(prelude)) = std.get("prelude") else {
                    panic!("`std` should always contains a map called `prelude`")
                };
                for (name, value) in prelude.iter() {
                    let name = IdentStr::new_boxed(name.clone().into()).expect(
                        "The values in `prelude` should all be named with valid identifiers",
                    );
                    context.vars_mut().let_(name, value.clone())
                }
            }
            // adding the std library
            context.vars_mut().let_(std_name.into_owned(), std.into());
        }

        Engine { context }
    }
}

#[derive(Debug)]
#[cfg_attr(
    feature = "bincode",
    derive(bincode::Encode, bincode::Decode),
    bincode(
        encode_bounds = "RNG: serde::Serialize, InjectedIntrisic: bincode::Encode, InjectedIntrisic::Data: bincode::Encode",
        decode_bounds = "RNG: serde::de::DeserializeOwned, InjectedIntrisic: bincode::Decode, InjectedIntrisic::Data: bincode::Decode",
        borrow_decode_bounds = "RNG: serde::Deserialize<'__de>, InjectedIntrisic: bincode::BorrowDecode<'__de>, InjectedIntrisic::Data: bincode::BorrowDecode<'__de>"
    )
)]
pub struct Engine<RNG, InjectedIntrisic: InjectedIntr> {
    context: Context<RNG, InjectedIntrisic>,
}

impl<RNG: Clone, InjectedIntrisic: InjectedIntr + Clone> Clone for Engine<RNG, InjectedIntrisic>
where
    InjectedIntrisic::Data: Clone,
{
    fn clone(&self) -> Self {
        Self {
            context: self.context.clone(),
        }
    }
}

#[cfg(feature = "eval_str")]
/// Error during evaluation of a string
pub type EvalStrError<InjectedIntrisic> =
    either::Either<dices_ast::expression::ParseError, SolveError<InjectedIntrisic>>;

impl<RNG, InjectedIntrisic: InjectedIntr> Engine<RNG, InjectedIntrisic> {
    /// Initialize a new engine
    ///
    /// This will use the entropy to initialize the rng
    pub fn new() -> Self
    where
        RNG: SeedableRng,
        InjectedIntrisic::Data: Default,
    {
        EngineBuilder::new()
            .inject_intrisics()
            .with_rng_from_entropy()
            .build()
    }

    /// Initialize a new engine
    pub fn new_with_rng(rng: RNG) -> Self
    where
        InjectedIntrisic: Clone,
        InjectedIntrisic::Data: Default,
    {
        EngineBuilder::new()
            .inject_intrisics()
            .with_rng(rng)
            .build()
    }

    /// Evaluate the result of an expression
    pub fn eval(
        &mut self,
        expr: &Expression<InjectedIntrisic>,
    ) -> Result<Value<InjectedIntrisic>, SolveError<InjectedIntrisic>>
    where
        RNG: DicesRng,
        InjectedIntrisic: Clone,
    {
        expr.solve(&mut self.context)
    }

    /// Evaluate the result of multiple expressions, returning the last one
    pub fn eval_multiple(
        &mut self,
        exprs: &NonEmpty<[Expression<InjectedIntrisic>]>,
    ) -> Result<Value<InjectedIntrisic>, SolveError<InjectedIntrisic>>
    where
        RNG: DicesRng,
        InjectedIntrisic: Clone,
    {
        solve_multiple(exprs, &mut self.context)
    }

    #[cfg(feature = "eval_str")]
    /// Evaluate a command string
    pub fn eval_str(
        &mut self,
        cmd: &str,
    ) -> Result<Value<InjectedIntrisic>, EvalStrError<InjectedIntrisic>>
    where
        RNG: DicesRng,
        InjectedIntrisic: Clone,
    {
        let exprs = dices_ast::parse_file(cmd).map_err(either::Either::Left)?;
        self.eval_multiple(&exprs).map_err(either::Either::Right)
    }

    pub fn injected_intrisics_data(&self) -> &<InjectedIntrisic as InjectedIntr>::Data {
        self.context.injected_intrisics_data()
    }

    pub fn injected_intrisics_data_mut(&mut self) -> &mut <InjectedIntrisic as InjectedIntr>::Data {
        self.context.injected_intrisics_data_mut()
    }
}

impl<RNG: SeedableRng, InjectedIntrisic: InjectedIntr> Default for Engine<RNG, InjectedIntrisic>
where
    InjectedIntrisic::Data: Default,
{
    fn default() -> Self {
        Self::new()
    }
}

pub trait DicesRng: Rng + SeedableRng + Serialize + DeserializeOwned {}
impl<T> DicesRng for T where T: Rng + SeedableRng + Serialize + DeserializeOwned {}

pub const VERSION: Version = Version::new(
    env!("CARGO_PKG_VERSION_MAJOR"),
    env!("CARGO_PKG_VERSION_MINOR"),
    env!("CARGO_PKG_VERSION_PATCH"),
);

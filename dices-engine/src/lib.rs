#![feature(assert_matches)]
#![feature(never_type)]
#![feature(iterator_try_collect)]
#![feature(box_patterns)]
#![feature(type_changing_struct_update)]

use std::borrow::Cow;

use either::Either::{self, Left, Right};
use nunny::NonEmpty;
use rand::{Rng, SeedableRng};

use dices_ast::{expression::Expression, ident::IdentStr, parse::parse_file, values::Value};

use solve::{solve_multiple, Solvable};

pub use context::Context;
pub use solve::SolveError;

mod context;
mod dices_std;
mod solve;

pub struct EngineBuilder<RNG = ()> {
    rng: RNG,
    std: Option<Cow<'static, IdentStr>>,
    prelude: bool,
}
impl EngineBuilder<()> {
    /// Start building a new engine
    pub fn new() -> Self {
        Self {
            rng: (),
            std: Some(Cow::Borrowed(IdentStr::new("std").unwrap())),
            prelude: true,
        }
    }
}
impl<RNG> EngineBuilder<RNG> {
    /// Add an RNG
    pub fn with_rng<NewRNG>(self, rng: NewRNG) -> EngineBuilder<NewRNG> {
        EngineBuilder { rng, ..self }
    }

    /// Add an RNG, seeding it from entropy
    pub fn with_rng_from_entropy<NewRNG>(self) -> EngineBuilder<NewRNG>
    where
        NewRNG: SeedableRng,
    {
        EngineBuilder {
            rng: NewRNG::from_entropy(),
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

    pub fn build(self) -> Engine<RNG> {
        let Self { rng, std, prelude } = self;
        // build context
        let mut context = Context::new(rng);
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
                        "The values in `prelude` should all be named with valid ientifiers",
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

pub struct Engine<RNG> {
    context: Context<RNG>,
}

impl<RNG> Engine<RNG> {
    /// Initialize a new engine
    ///
    /// This will use the entropy to initialize the rng
    pub fn new() -> Self
    where
        RNG: SeedableRng,
    {
        EngineBuilder::new().with_rng_from_entropy().build()
    }

    /// Initialize a new engine
    pub fn new_with_rng(rng: RNG) -> Self {
        EngineBuilder::new().with_rng(rng).build()
    }

    /// Evaluate the result of an expression
    pub fn eval(&mut self, expr: &Expression) -> Result<Value, SolveError>
    where
        RNG: Rng,
    {
        expr.solve(&mut self.context)
    }

    /// Evaluate the result of multiple expressions, returning the last one
    pub fn eval_multiple(&mut self, exprs: &NonEmpty<[Expression]>) -> Result<Value, SolveError>
    where
        RNG: Rng,
    {
        solve_multiple(exprs, &mut self.context)
    }

    /// Evaluate a command string
    pub fn eval_str(
        &mut self,
        cmd: &str,
    ) -> Result<Value, Either<dices_ast::parse::Error, SolveError>>
    where
        RNG: Rng,
    {
        let exprs = parse_file(cmd).map_err(Left)?;
        self.eval_multiple(&exprs).map_err(Right)
    }
}

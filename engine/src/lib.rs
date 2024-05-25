//! Engine for the dices programming language
#![feature(never_type)]
#![feature(iterator_try_collect)]
#![feature(iterator_try_reduce)]
#![feature(assert_matches)]
#![feature(unwrap_infallible)]
#![feature(box_patterns)]
#![feature(ascii_char)]

use either::Either;

pub mod identifier;

pub mod namespace;
use namespace::Namespace;

pub mod expr;
pub use expr::{EvalError, Expr};

pub mod value;
use rand::{Rng, SeedableRng};
pub use value::Value;

#[cfg(feature = "parse")]
pub mod parser;
#[cfg(feature = "parse")]
pub use parser::parse_exprs;

#[cfg(feature = "pretty")]
mod display;

#[derive(Debug, Clone)]
/// The `dices` engine.
pub struct Engine<RNG> {
    /// The root namespace for this engine
    namespace: Namespace<'static>,
    /// The random number generator
    rng: RNG,
}

impl<RNG: SeedableRng> Engine<RNG> {
    /// Create a new engine
    pub fn new() -> Self {
        Self {
            namespace: Namespace::root(),
            rng: SeedableRng::from_entropy(),
        }
    }
    /// Create a new engine with the given seed for the RNG
    pub fn new_seeded(seed: RNG::Seed) -> Self {
        Self {
            namespace: Namespace::root(),
            rng: SeedableRng::from_seed(seed),
        }
    }
    /// Create a new engine with the given random seed
    pub fn new_seeded_from_u64(seed: u64) -> Self {
        Self {
            namespace: Namespace::root(),
            rng: SeedableRng::seed_from_u64(seed),
        }
    }
}
impl<RNG: Rng> Engine<RNG> {
    /// Evaluate an expression
    pub fn eval(&mut self, expr: &Expr) -> Result<Value, EvalError> {
        expr.eval(&mut self.namespace, &mut self.rng)
    }

    #[cfg(feature = "parse")]
    /// Evaluate a REPL line, discarding all values except the last
    pub fn eval_line(
        &mut self,
        line: &str,
    ) -> Result<Value, Either<peg::error::ParseError<peg::str::LineCol>, EvalError>> {
        let exprs = parse_exprs(line).map_err(Either::Left)?;
        let Some((last, init)) = exprs.split_last() else {
            return Ok(Value::Null);
        };
        for expr in init {
            self.eval(expr).map_err(Either::Right)?;
        }
        self.eval(last).map_err(Either::Right)
    }
}

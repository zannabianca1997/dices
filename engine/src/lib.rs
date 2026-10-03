#![doc = include_str!("../README.md")]

use dices_ast::{expr::scope::ScopeInner, identifier::Identifier};
use dices_std::Std;
use dices_values::{
    Value,
    cast::{CastInjectedError, CastIntoIntError},
    injected::{CallError, typed::TypedValueInjected},
};
use rand::SeedableRng;
use snafu::Snafu;

use crate::context::EngineContext;

pub mod context;
mod eval;
pub mod ui;
mod var_use;

/// Temporary raw-fd marker used while bringing the badge build up: stdio and
/// std's stderr produce no output on the device, so debug prints go straight to
/// fd 1. Compiled out everywhere except the ESP-IDF target.
#[cfg(target_os = "espidf")]
pub(crate) fn badge_debug_mark(s: &str) {
    unsafe extern "C" {
        fn write(fd: i32, buf: *const u8, n: usize) -> isize;
    }

    unsafe {
        let _ = write(1, s.as_ptr(), s.len());
    }
}

#[cfg(not(target_os = "espidf"))]
pub(crate) fn badge_debug_mark(_s: &str) {}

pub trait Evaluator {
    fn eval<Ui: ui::Ui>(&mut self, stmt: &ScopeInner, ui: Ui) -> Result<Value, EvalError>;
}

/// An engine evaluating dices statements
#[derive(Debug, Clone)]
pub struct Engine {
    /// Random number generator
    rng: rand_pcg::Lcg64Xsh32,
    /// Global variables
    globals: context::Scope,
    /// Standard library
    std: TypedValueInjected<Std>,
}

impl Engine {
    /// Create a new engine
    pub fn new(seed: [u8; 16], std: Std) -> Self {
        badge_debug_mark("E:injected\n");
        let std = TypedValueInjected::new(std);
        badge_debug_mark("E:injected-done\n");
        let globals = context::Scope::new_global(std.clone());
        badge_debug_mark("E:scope-done\n");
        let rng = SeedableRng::from_seed(seed);
        badge_debug_mark("E:rng-done\n");
        Self { rng, globals, std }
    }

    /// Create a new engine with a static standard library
    pub fn new_static_std(seed: [u8; 16], std: &'static Std) -> Self {
        let std = TypedValueInjected::new_static(std);
        let globals = context::Scope::new_global(std.clone());
        Self {
            rng: SeedableRng::from_seed(seed),
            globals,
            std,
        }
    }
}

impl Evaluator for Engine {
    fn eval<Ui: ui::Ui>(&mut self, stmt: &ScopeInner, ui: Ui) -> Result<Value, EvalError> {
        EngineContext::new(self, ui).eval(stmt)
    }
}

#[derive(Debug, Snafu)]
pub enum EvalError {
    #[snafu(transparent)]
    CastInjected { source: CastInjectedError },
    #[snafu(transparent)]
    CastIntoInt { source: CastIntoIntError },
    #[snafu(display("Division by zero"))]
    DivisionByZero,
    #[snafu(display("Multiplication between two non scalar"))]
    MulBetweenNonScalars {
        lhs: CastIntoIntError,
        rhs: CastIntoIntError,
    },
    #[snafu(display("Cannot compare value of type {} with value of type {}", lhs.typ(), rhs.typ()))]
    IncomparableValues { lhs: Value, rhs: Value },
    #[snafu(display("Unknown variable {name}"))]
    VariableDoNotExists { name: Identifier },
    #[snafu(display("Error in calling value of type {}", value.typ()))]
    Call { value: Value, source: CallError },
    #[snafu(display("Cannot index value of type {}", container.typ()))]
    NonIndexable { container: Value },
    #[snafu(display("Cannot index value of type {} with value of type {}", container.typ(), index.typ()))]
    NonIndexableWith { container: Value, index: Value },
    #[snafu(display("Cannot index with a list of lenght different that 2 (found {len})"))]
    IndexingWithListNeedLenghtTwo { len: usize },
}

//! Engine for the dices programming language
#![feature(never_type)]
#![feature(iterator_try_collect)]
#![feature(iterator_try_reduce)]
#![feature(assert_matches)]
#![feature(unwrap_infallible)]
#![feature(box_patterns)]
#![feature(ascii_char)]
#![feature(extract_if)]
#![feature(type_changing_struct_update)]
#![feature(box_into_inner)]

use std::{collections::HashMap, rc::Rc};

use either::Either;

pub mod identifier;

pub mod namespace;
use expr::EvalInterrupt;
use identifier::IdentStr;
use namespace::Namespace;

pub mod expr;
pub use expr::{EvalError, Expr};

pub mod value;
use rand::Rng;
use std_lib::std_lib;
pub use value::Value;

pub mod intrisics;

pub mod std_lib;

#[cfg(feature = "parse")]
pub mod parser;
#[cfg(feature = "parse")]
pub use parser::{parse_exprs, ParseError};

#[cfg(feature = "pretty")]
pub mod pretty;

/// Context of an expression evaluation
enum EvalContext<'e, 'n, RNG, P> {
    Engine {
        namespace: &'e mut Namespace<'n>,
        rng: &'e mut RNG,
        callbacks: &'e mut Callbacks<P>,
    },
    Const {
        namespace: &'e mut Namespace<'n>,
    },
}
impl<'e, 'n, RNG, P> EvalContext<'e, 'n, RNG, P> {
    pub fn namespace(&mut self) -> &mut Namespace<'n> {
        match self {
            EvalContext::Engine { namespace, .. } => *namespace,
            EvalContext::Const { namespace, .. } => *namespace,
        }
    }
    pub fn rng(&mut self) -> Option<&mut RNG> {
        match self {
            EvalContext::Engine { rng, .. } => Some(*rng),
            EvalContext::Const { .. } => None,
        }
    }

    /// Returns `true` if the eval context is [`Const`].
    ///
    /// [`Const`]: EvalContext::Const
    #[must_use]
    pub fn is_const(&self) -> bool {
        matches!(self, Self::Const { .. })
    }
}

#[derive(Debug, Clone)]
pub struct EngineBuilder<RNG = NotAvailable, P = NotAvailable> {
    vars: HashMap<Rc<IdentStr>, Value>,
    std: Option<Rc<IdentStr>>,
    prelude: bool,
    rng: RNG,
    print: P,
}
impl EngineBuilder {
    pub fn new() -> Self {
        Self {
            vars: HashMap::new(),
            rng: NotAvailable,
            std: Some(IdentStr::new("std").unwrap().into()),
            prelude: false,
            print: NotAvailable,
        }
    }
}

impl<RNG: Rng, P: Printer> EngineBuilder<RNG, P> {
    pub fn build(self) -> Engine<RNG, P> {
        let Self {
            vars,
            rng,
            std,
            prelude,
            print,
        } = self;
        // building the namespace
        let mut namespace = Namespace::root_with_vars(vars);
        if prelude {
            // adding common elements
            for (ident, value) in std_lib::prelude::<RNG, P>() {
                namespace.let_(ident, value)
            }
        }
        if let Some(std) = std {
            namespace.let_(std, std_lib::<RNG, P>());
        }
        // callbacks
        let callbacks = Callbacks { print };
        Engine {
            namespace,
            rng,
            callbacks,
        }
    }
}
impl<RNG, P> EngineBuilder<RNG, P> {
    pub fn prelude(self, prelude: bool) -> Self {
        Self { prelude, ..self }
    }
    pub fn with_prelude(self) -> Self {
        self.prelude(true)
    }
    pub fn no_prelude(self) -> Self {
        self.prelude(false)
    }

    pub fn rng<NewRNG>(self, rng: NewRNG) -> EngineBuilder<NewRNG, P> {
        EngineBuilder { rng, ..self }
    }

    pub fn std(self, name: Option<impl Into<Rc<IdentStr>>>) -> Self {
        Self {
            std: name.map(|n| n.into()),
            ..self
        }
    }
    pub fn no_std(self) -> Self {
        self.std(Option::<Rc<IdentStr>>::None)
    }
    pub fn with_std(self, name: impl Into<Rc<IdentStr>>) -> Self {
        self.std(Some(name))
    }

    pub fn var(mut self, name: impl Into<Rc<IdentStr>>, value: impl Into<Value>) -> Self {
        self.vars.insert(name.into(), value.into());
        self
    }
    pub fn vars(
        mut self,
        vars: impl IntoIterator<Item = (impl Into<Rc<IdentStr>>, impl Into<Value>)>,
    ) -> Self {
        self.vars
            .extend(vars.into_iter().map(|(n, v)| (n.into(), v.into())));
        self
    }

    pub fn print<NewPrint: Printer>(self, print: NewPrint) -> EngineBuilder<RNG, NewPrint> {
        EngineBuilder { print, ..self }
    }
    pub fn no_print(self) -> EngineBuilder<RNG, NotAvailable> {
        self.print(NotAvailable)
    }
}

#[derive(Debug, Clone)]
/// The `dices` engine.
pub struct Engine<RNG, P> {
    /// The root namespace for this engine
    namespace: Namespace<'static>,
    /// The random number generator
    rng: RNG,
    /// Callbacks to interface capabilities
    callbacks: Callbacks<P>,
}
impl<RNG: Rng, P: Printer> Engine<RNG, P> {
    /// Evaluate an expression
    pub fn eval(&mut self, expr: &Expr) -> Result<EvalResult, EvalError> {
        match expr.eval(&mut EvalContext::Engine {
            namespace: &mut self.namespace,
            rng: &mut self.rng,
            callbacks: &mut self.callbacks,
        }) {
            Ok(value) => Ok(EvalResult::Ok(value)),
            Err(EvalInterrupt::Quitted(params)) => Ok(EvalResult::Quitted(params)),
            Err(EvalInterrupt::Error(err)) => Err(err),
            Err(EvalInterrupt::CannotEvalInConst(_)) => unreachable!("Context was not constant"),
        }
    }

    #[cfg(feature = "parse")]
    /// Evaluate a REPL line, discarding all values except the last
    pub fn eval_line(&mut self, line: &str) -> Result<EvalResult, ParseEvalError> {
        let exprs = parse_exprs(line).map_err(Either::Left)?;
        let Some((last, init)) = exprs.split_last() else {
            return Ok(EvalResult::Ok(Value::Null));
        };
        for expr in init {
            if let r @ EvalResult::Quitted(_) = self.eval(expr).map_err(Either::Right)? {
                // fast return if quitted
                return Ok(r);
            };
        }
        self.eval(last).map_err(Either::Right)
    }
}

#[cfg(feature = "parse")]
pub type ParseEvalError = Either<peg::error::ParseError<peg::str::LineCol>, EvalError>;

#[derive(Debug, Clone)]
/// Possible results of an evaluation
pub enum EvalResult {
    /// Evaluation terminated normally
    Ok(Value),
    /// `quit` intrisic called with the returned params
    Quitted(Box<[Value]>),
}

impl EvalResult {
    /// Returns `true` if the eval result is [`Quitted`].
    ///
    /// [`Quitted`]: EvalResult::Quitted
    #[must_use]
    pub fn is_quitted(&self) -> bool {
        matches!(self, Self::Quitted(..))
    }
}

#[derive(Debug, Clone)]
/// Callbacks to interface capabilities
pub struct Callbacks<P> {
    /// Printing callback
    print: P,
}

pub trait Printer {
    const AVAILABLE: bool;
    fn print(&mut self, value: Value);
}
impl<T> Printer for T
where
    T: FnMut(Value),
{
    const AVAILABLE: bool = true;
    fn print(&mut self, value: Value) {
        self(value)
    }
}
/// Mark a callback as not available
pub struct NotAvailable;
impl Printer for NotAvailable {
    const AVAILABLE: bool = false;
    fn print(&mut self, _value: Value) {
        unreachable!("Printer is not available")
    }
}

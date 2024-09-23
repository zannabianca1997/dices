#![doc = include_str!("../README.md")]
#![feature(box_patterns)]
#![feature(never_type)]
#![feature(step_trait)]

pub mod fmt;
pub mod ident;
pub mod intrisics;

pub mod value;
pub use value::Value;

pub mod expression;
#[cfg(feature = "parse_expression")]
pub use expression::parse_file;
pub use expression::Expression;

#[cfg(feature = "matcher")]
pub mod matcher;
#[cfg(feature = "matcher")]
pub use matcher::Matcher;

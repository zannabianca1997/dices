//! Evaluation context

use crate::Engine;

/// Evaluation context
pub struct Context<'engine> {
    engine: &'engine mut Engine,
}

impl<'engine> Context<'engine> {
    pub fn new(engine: &'engine mut Engine) -> Self {
        Self { engine }
    }
}

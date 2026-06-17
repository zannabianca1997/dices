//! Evaluation context

use dices_values::int::ValueInt;
use num::traits::ConstOne;
use rand::Rng;

use crate::Engine;

/// Evaluation context
pub struct Context<'engine> {
    engine: &'engine mut Engine,
}

impl<'engine> Context<'engine> {
    /// Create a new context
    pub(crate) fn new(engine: &'engine mut Engine) -> Self {
        Self { engine }
    }

    /// Throw a dice
    pub fn dice(&mut self, faces: ValueInt) -> ValueInt {
        let range = if faces > ValueInt::ONE {
            ValueInt::ONE..=faces
        } else {
            faces..=ValueInt::ONE
        };
        self.engine.rng.gen_range(range)
    }
}

//! Static analysis of variable usage
//!
//! Needed for closure captures

use std::collections::BTreeSet;

use dices_ast::identifier::Identifier;
use itertools::Itertools;

pub struct VarUse {
    /// Variables that are read
    pub reads: BTreeSet<Identifier>,
    /// Variables that are set
    ///
    /// If a variable is here but not in `read`, the variable is set without
    /// reading its previous value.
    pub sets: BTreeSet<Identifier>,
    /// Variables that are declared
    ///
    /// If a variable is here but not in `read`, the variable is declared
    /// without reading its previous value.
    ///
    /// If a variable is both here and in `sets`, the variable is first set,
    /// then shadowed.
    pub lets: BTreeSet<Identifier>,
}

impl VarUse {
    /// No variable use
    pub fn none() -> Self {
        Self {
            reads: BTreeSet::new(),
            sets: BTreeSet::new(),
            lets: BTreeSet::new(),
        }
    }
    /// Just read
    pub fn read(name: Identifier) -> Self {
        Self {
            reads: BTreeSet::from([name]),
            sets: BTreeSet::new(),
            lets: BTreeSet::new(),
        }
    }
    /// Just set
    pub fn set(name: Identifier) -> Self {
        Self {
            reads: BTreeSet::new(),
            sets: BTreeSet::from([name]),
            lets: BTreeSet::new(),
        }
    }
    /// Just define
    pub fn r#let(name: Identifier) -> Self {
        Self {
            reads: BTreeSet::new(),
            sets: BTreeSet::new(),
            lets: BTreeSet::from([name]),
        }
    }

    /// Variable use of a sequence of instructions
    ///
    /// This is associative: `a.then(b).then(c) == a.then(b.then(c))`
    ///
    /// This is also idempotent: `a.then(a) == a`
    pub fn then(self, then: Self) -> Self {
        // Read are all the variables that the first reads, plus the ones
        // read by the second except if set or defined by the first
        let self_controls_values_of = self.sets.union(&self.lets).cloned().collect();
        let reads = self
            .reads
            .iter()
            .chain(then.reads.difference(&self_controls_values_of))
            .cloned()
            .collect();

        // Written are all the variable the first writes, plus the ones the
        // second writes, except if defined by the first
        let sets = self
            .sets
            .iter()
            .chain(then.sets.difference(&self.lets))
            .cloned()
            .collect();

        // Defined are what both defines
        let lets = self.lets.union(&then.lets).cloned().collect();

        Self { reads, sets, lets }
    }

    /// Sequence of operation
    ///
    /// Shorthand for `.reduce(VarUse::then)`
    pub fn sequence(seq: impl IntoIterator<Item = Self>) -> Self {
        seq.into_iter()
            .tree_reduce(Self::then)
            .unwrap_or_else(Self::none)
    }

    /// Scope an operation
    pub fn scoped(self) -> Self {
        Self {
            lets: BTreeSet::new(),
            ..self
        }
    }
}

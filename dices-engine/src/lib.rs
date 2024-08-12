#![feature(assert_matches)]

mod context {
    //! Context essential to evaluate a `dices` expression

    use std::{assert_matches::debug_assert_matches, collections::BTreeMap, mem};

    use dices_ast::{ident::IdentStr, values::Value};
    use nonempty::NonEmpty;

    #[derive(Debug, Clone)]
    pub(crate) struct Context<RNG> {
        /// the stack of variables
        scopes: NonEmpty<BTreeMap<Box<IdentStr>, Value>>,
        /// The random number generator
        rng: RNG,
    }

    impl<RNG> Context<RNG> {
        pub fn new(rng: RNG) -> Self {
            Self {
                scopes: NonEmpty::new(BTreeMap::new()),
                rng,
            }
        }

        /// Let a variable be, setting its value if present in the current scope, or creating it
        pub fn let_var(&mut self, name: Box<IdentStr>, value: Value) {
            self.scopes.last_mut().insert(name, value);
        }
        /// Find the value of a variable
        pub fn get_var(&self, name: &IdentStr) -> Option<&Value> {
            // find the last scope that contains that variable
            self.scopes.iter().rev().find_map(|s| s.get(name))
        }
        /// Find the value of a variable, and permit to modify it
        pub fn get_var_mut(&mut self, name: &IdentStr) -> Option<&mut Value> {
            // find the last scope that contains that variable
            self.scopes.iter_mut().rev().find_map(|s| s.get_mut(name))
        }

        /// run code in a local scope, with the same RNG and no local variables
        pub fn scoped<R>(&mut self, f: impl FnOnce(&mut Self) -> R) -> R {
            self.scopes.push(BTreeMap::new());
            let res = f(self);
            let locals = self.scopes.pop();
            debug_assert_matches!(locals, Some(_), "The pop and pushes should be balanced");
            res
        }

        /// run code in a jail, with the same RNG but no variables
        pub fn jailed<R>(&mut self, f: impl FnOnce(&mut Self) -> R) -> R {
            let old_scopes = mem::replace(&mut self.scopes, NonEmpty::new(BTreeMap::new()));
            let res = f(self);
            debug_assert_matches!(
                self.scopes.tail(),
                &[],
                "All the additional scopes should have popped when the jailed context is destroyied"
            );
            self.scopes = old_scopes;
            res
        }

        pub fn rng(&mut self) -> &mut RNG {
            &mut self.rng
        }
    }
}

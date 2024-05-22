//! Implementation of namespace
//!
//! # Safety
//!
//! A trivial implementation of stack allocated namespaces runs into variance issues.
//! ```no_run
//! # use std::collections::HashMap;
//! struct Namespace<'r> {
//!     vars: HashMap<String, i64>,
//!     parent: Option<&'r mut Namespace<'r>>
//! }
//! ```
//! `Namespace<'r>` is invariant on `'r` because `&mut T` is invariant on `T`. This is
//! because Rust is scared we evily exchange the parent for a shorter lived one:
//! ```compile_fail
//! # use std::collections::HashMap;
//! # struct Namespace<'r> {
//! #     vars: HashMap<String, i64>,
//! #     parent: Option<&'r mut Namespace<'r>>
//! # }
//!
//! impl Namespace<'_> {
//!     pub fn new() -> Namespace<'static> {
//!         Namespace {
//!             vars: HashMap::new(),
//!             parent: None
//!         }
//!     }
//!     pub fn child(&mut self) -> Namespace {
//!         Namespace {
//!             vars: HashMap::new(),
//!             parent: Some(self)
//!         }
//!     }
//! }
//!
//! let mut n1: Namespace<'static> = Namespace::new();
//! {                                                                                              // +
//!     let mut n2: Namespace = n1.child(); // we forget that `n2.parent` is `'static`             // |
//!     let mut evil_root: Namespace = Namespace::new(); // this is only `'a`                      // |
//!     n2    // this is `Namespace<'a>`, and forgot the true lifetime of the parent               // | 'a
//!         .parent.as_mut().unwrap()  // this is now `&'a mut Namespace<'a>`, pointing to `n1`    // |
//!         .parent = Some(&mut evil_root); // so this is okay, given `evil_root` is `'a`          // |
//!     std::mem::drop(n2); std::mem::drop(evil_root); // ending `'a` lifetimes                    // +
//! }
//! let freed = n1.parent.unwrap(); // This is `evil_root`, OUTSIDE its lifetime!
//! ```
//! But we are good boys, and pinky promise to never change the parent namespace. `parent` is a
//! private field, and there are no methods that change its value. All the creation method either
//! set `parent` to `None`, or strictly limit the child lifetime to be less than the parent one.
//! So `parent` is always either `None`, or a valid pointer to a unique borrowed namespace.
//! In particular those should never compile:
//! ```compile_fail,E0499
//! # use engine::{namespace::Namespace, value::Value}
//! let mut root = Namespace::root();
//! // creating a child
//! let mut child = root.child();
//! // try to use them both
//! root.let_("a".try_into().unwrap(), Value::Number(42));
//! child.let_("a".try_into().unwrap(), Value::Bool(true));
//! ```
//! ```compile_fail,E0499
//! # use engine::{namespace::Namespace, value::Value}
//! let mut root = Namespace::root();
//! // creating two childs at the same time
//! let mut child1 = root.child();
//! let mut child2 = root.child();
//! // try to use them both
//! child1.let_("a".try_into().unwrap(), Value::Number(42));
//! child2.let_("a".try_into().unwrap(), Value::Bool(true));
//! ```

use std::{collections::HashMap, marker::PhantomData, ptr::NonNull as NonNullPtr};

use crate::{identifier::DIdentifier, value::Value};

/// A namespace
pub struct Namespace<'r> {
    /// Variable stored in this namespace
    vars: HashMap<DIdentifier, Value>,
    /// Storing a raw pointer because variance issues (see the safety disclaimer)
    parent: Option<NonNullPtr<Namespace<'r>>>,
    /// This is what logically we are containing
    phantom: PhantomData<&'r mut [HashMap<DIdentifier, Value>]>,
}
impl Namespace<'_> {
    /// Obtain the value of a variable
    ///
    /// ```
    /// # use engine::{namespace::Namespace, value::Value::Number};
    /// # let mut namespace = Namespace::root();
    /// namespace.let_("a".try_into().unwrap(), Number(42));
    /// assert_eq!(namespace.get(&"a".try_into().unwrap()), Some(&Number(42)));
    /// ```
    pub fn get<'s>(&'s self, name: &DIdentifier) -> Option<&'s Value> {
        self.vars.get(name).or_else(|| {
            self.parent.as_ref().and_then(|p| {
                unsafe {
                    // SAFETY: see top-module explanation
                    p.as_ref()
                }
                .get(name)
            })
        })
    }

    /// Obtains a mutable reference to the value of a variable
    ///
    /// ```
    /// # use engine::{namespace::Namespace, value::Value::{Number, Bool}};
    /// # let mut namespace = Namespace::root();
    /// namespace.let_("a".try_into().unwrap(), Number(42));
    /// *namespace.get_mut(&"a".try_into().unwrap()).unwrap() = Bool(true);
    /// assert_eq!(namespace.get(&"a".try_into().unwrap()), Some(&Bool(true)));
    /// ```
    pub fn get_mut<'s>(&'s mut self, name: &DIdentifier) -> Option<&'s mut Value> {
        self.vars.get_mut(name).or_else(|| {
            self.parent.as_mut().and_then(|p| {
                unsafe {
                    // SAFETY: see top-module explanation
                    p.as_mut()
                }
                .get_mut(name)
            })
        })
    }

    /// Generate a new empty root namespace
    ///
    /// ```
    /// # use engine::{namespace::Namespace, value::Value::{Number, Bool}};
    /// let namespace = Namespace::root();
    /// assert_eq!(namespace.get(&"a".try_into().unwrap()), None);
    /// ```
    pub fn root() -> Namespace<'static> {
        Namespace {
            vars: HashMap::new(),
            parent: None,
            phantom: PhantomData,
        }
    }
    /// Generate a new root namespace with the given variables
    ///
    /// ```
    /// # use std::collections::HashMap;
    /// # use engine::{namespace::Namespace, value::Value::{Number, Bool}};
    /// let namespace = Namespace::root_with_vars(HashMap::from([("a".try_into().unwrap(), Number(42))]));
    /// assert_eq!(namespace.get(&"a".try_into().unwrap()), Some(&Number(42)));
    /// ```
    pub fn root_with_vars(vars: HashMap<DIdentifier, Value>) -> Namespace<'static> {
        Namespace {
            vars,
            parent: None,
            phantom: PhantomData,
        }
    }

    /// Generate a child namespace that refers to this one, and with no locals
    ///
    /// ```
    /// # use std::collections::HashMap;
    /// # use engine::{namespace::Namespace, value::Value::{Number, Bool}};
    /// let mut root = Namespace::root_with_vars(HashMap::from([
    ///     ("parent_a".try_into().unwrap(), Number(42)),
    ///     ("shadowed".try_into().unwrap(), Number(-42)),
    /// ]));
    /// {
    ///     let mut child = root.child();
    ///     
    ///     // child can read parent vars
    ///     assert_eq!(child.get(&"parent_a".try_into().unwrap()), Some(&Number(42)));
    ///
    ///     // child can change parent vars
    ///     *child.get_mut(&"parent_a".try_into().unwrap()).unwrap() = Bool(false);
    ///     
    ///     // child can define its vars
    ///     child.let_("child_b".try_into().unwrap(), Number(101));
    ///     assert_eq!(child.get(&"child_b".try_into().unwrap()), Some(&Number(101)));
    ///
    ///     // child vars can shadow the parent
    ///     child.let_("shadowed".try_into().unwrap(), Number(-666));
    ///     assert_eq!(child.get(&"shadowed".try_into().unwrap()), Some(&Number(-666)));
    ///
    ///     // shadowing vars change do not affect parent
    ///     *child.get_mut(&"shadowed".try_into().unwrap()).unwrap() = Bool(true);
    ///     assert_eq!(child.get(&"shadowed".try_into().unwrap()), Some(&Bool(true)));
    /// }
    ///
    /// // parent variable changed in child change in parent
    /// assert_eq!(root.get(&"parent_a".try_into().unwrap()), Some(&Bool(false)));
    /// // variable shadowed in child do not change in parent
    /// assert_eq!(root.get(&"shadowed".try_into().unwrap()), Some(&Number(-42)));
    /// // parent do not gain child vars
    /// assert_eq!(root.get(&"child_b".try_into().unwrap()), None);
    /// ```
    pub fn child<'s, 'c>(&'s mut self) -> Namespace<'c>
    where
        's: 'c,
    {
        Namespace {
            vars: HashMap::new(),
            parent: Some(self.into()),
            phantom: PhantomData,
        }
    }

    /// Create a new varible, eventually shadowing the ones present in the parent
    pub fn let_(&mut self, ident: DIdentifier, value: Value) {
        self.vars.insert(ident, value);
    }
}

impl Extend<(DIdentifier, Value)> for Namespace<'_> {
    fn extend<T: IntoIterator<Item = (DIdentifier, Value)>>(&mut self, iter: T) {
        self.vars.extend(iter)
    }
}

impl FromIterator<(DIdentifier, Value)> for Namespace<'static> {
    fn from_iter<T: IntoIterator<Item = (DIdentifier, Value)>>(iter: T) -> Self {
        Self::root_with_vars(HashMap::from_iter(iter))
    }
}

#[cfg(test)]
mod tests {

    mod flat {
        use super::super::Namespace;
        use crate::value::Value::{Bool, Number};

        #[test]
        /// Checks we can declare variable and recover their value
        fn let_() {
            let mut n = Namespace::root();

            n.let_("a".try_into().unwrap(), Number(3));
            n.let_("b".try_into().unwrap(), Bool(true));

            assert_eq!(n.get(&"a".try_into().unwrap()), Some(&Number(3)));
            assert_eq!(n.get(&"b".try_into().unwrap()), Some(&Bool(true)));
            assert_eq!(n.get(&"c".try_into().unwrap()), Option::None)
        }

        #[test]
        /// Checks we can change variable value
        fn set() {
            let mut n = Namespace::root();

            n.let_("b".try_into().unwrap(), Bool(true));

            *n.get_mut(&"b".try_into().unwrap()).unwrap() = Number(42);

            assert_eq!(n.get(&"b".try_into().unwrap()), Some(&Number(42)));
        }
    }

    mod nested {

        use super::super::Namespace;
        use crate::value::Value::{Bool, Number};

        #[test]
        /// Check we can read the parent values
        fn read() {
            let mut root = Namespace::root();

            root.let_("a".try_into().unwrap(), Number(3));

            {
                let child = root.child();

                assert_eq!(child.get(&"a".try_into().unwrap()), Some(&Number(3)));
                assert_eq!(child.get(&"c".try_into().unwrap()), Option::None);

                std::mem::drop(child)
            }

            assert_eq!(root.get(&"a".try_into().unwrap()), Some(&Number(3)));
            assert_eq!(root.get(&"c".try_into().unwrap()), Option::None);
        }

        #[test]
        /// Check we can write to the parent values
        fn set_parent() {
            let mut root = Namespace::root();

            root.let_("b".try_into().unwrap(), Bool(true));

            {
                let mut child = root.child();

                *child.get_mut(&"b".try_into().unwrap()).unwrap() = Number(42);

                std::mem::drop(child)
            }

            assert_eq!(root.get(&"b".try_into().unwrap()), Some(&Number(42)));
        }

        #[test]
        /// Check we can shadow the parent values
        fn shadow() {
            let mut root = Namespace::root();

            root.let_("b".try_into().unwrap(), Bool(true));

            {
                let mut child = root.child();

                child.let_("b".try_into().unwrap(), Number(42));

                assert_eq!(child.get(&"b".try_into().unwrap()), Some(&Number(42)));

                std::mem::drop(child)
            }

            assert_eq!(root.get(&"b".try_into().unwrap()), Some(&Bool(true)));
        }

        #[test]
        /// Check we can change a variable that shadow the parent values, without changing the parent
        fn shadow_set() {
            let mut root = Namespace::root();

            root.let_("b".try_into().unwrap(), Bool(true));

            {
                let mut child = root.child();

                child.let_("b".try_into().unwrap(), Number(42));

                *child.get_mut(&"b".try_into().unwrap()).unwrap() = Number(76);

                assert_eq!(child.get(&"b".try_into().unwrap()), Some(&Number(76)));

                std::mem::drop(child)
            }

            assert_eq!(root.get(&"b".try_into().unwrap()), Some(&Bool(true)));
        }
    }
}

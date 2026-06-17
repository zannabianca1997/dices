use std::{
    any::TypeId,
    cmp::Ordering,
    fmt::{self, Debug, Display, Formatter},
    hash::{Hash, Hasher},
};

use crate::injected::describable::Describable;

/// All traits needed to an injectable object, and that are provided by other
/// traits
///
/// Implement this by implementing all traits listed in the blanked impl.
pub trait RequiredTraits: Debug + 'static {
    // dynamic dispatch friendly methods

    fn type_id(&self) -> TypeId;

    fn dyn_eq(&self, other: &dyn RequiredTraits) -> bool;
    fn dyn_partial_cmp(&self, other: &dyn RequiredTraits) -> Option<Ordering>;
    fn dyn_hash(&self, state: &mut dyn Hasher);

    fn dyn_description(&self, f: &mut Formatter<'_>) -> fmt::Result;
}
impl<T> RequiredTraits for T
where
    T: Eq + PartialOrd + Hash + Debug + Describable + 'static,
{
    fn type_id(&self) -> TypeId {
        TypeId::of::<T>()
    }

    fn dyn_eq(&self, other: &dyn RequiredTraits) -> bool {
        if let Some(other) = other.downcast_ref::<Self>() {
            self == other
        } else {
            false
        }
    }

    fn dyn_partial_cmp(&self, other: &dyn RequiredTraits) -> Option<Ordering> {
        if let Some(other) = other.downcast_ref::<Self>() {
            self.partial_cmp(other)
        } else {
            None
        }
    }

    fn dyn_hash(&self, state: &mut dyn Hasher) {
        struct Wrapper<'a>(&'a mut dyn Hasher);

        macro_rules! pass {
        ( $($ty:ident)* ) => {
            paste::paste!{
                $(
                    fn [< write_ $ty >] ( &mut self, i: $ty ) {
                        self.0. [< write_ $ty >] (i)
                    }
                )*
            }
        };
    }

        impl Hasher for Wrapper<'_> {
            fn finish(&self) -> u64 {
                self.0.finish()
            }

            fn write(&mut self, bytes: &[u8]) {
                self.0.write(bytes);
            }

            pass! {
                u8 u16 u32 u64 u128 usize
                i8 i16 i32 i64 i128 isize
            }
        }

        self.hash(&mut Wrapper(state));
    }

    fn dyn_description(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.description().fmt(f)
    }
}

impl dyn RequiredTraits {
    /// Reimplementation of some of the "Any" machinery to avoid an indirection
    fn downcast_ref<T: RequiredTraits>(&self) -> Option<&T> {
        if TypeId::of::<T>() == self.type_id() {
            // SAFETY: just checked whether we are pointing to the correct type,
            // and we can rely on that check for memory safety because
            // RequiredTraits is not reachable and cannot be implemented outside
            // this crate.
            unsafe { Some(&*(self as *const dyn RequiredTraits as *const T)) }
        } else {
            None
        }
    }
}

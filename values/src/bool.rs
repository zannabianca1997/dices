//! Boolean value

use std::ops::{BitAnd, BitOr, BitXor, Not};

use derive_more::{Display, From, Into};

/// Boolean value
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Display, From, Into,
)]
#[display("{_0}")]
pub struct ValueBool(bool);

impl ValueBool {
    /// The `true` value
    pub const TRUE: Self = Self(true);
    /// The `false` value
    pub const FALSE: Self = Self(false);

    /// The wrapped boolean
    pub const fn get(self) -> bool {
        self.0
    }
}

impl Not for ValueBool {
    type Output = Self;

    fn not(self) -> Self::Output {
        Self(!self.0)
    }
}

/// Implement a binary logical operator (owned and `&`-rhs), delegating to the
/// matching `bool` operator.
macro_rules! impl_op {
    ($trait:ident, $method:ident) => {
        impl $trait for ValueBool {
            type Output = ValueBool;

            fn $method(self, rhs: Self) -> Self::Output {
                Self($trait::$method(self.0, rhs.0))
            }
        }

        impl $trait<&ValueBool> for ValueBool {
            type Output = ValueBool;

            fn $method(self, rhs: &ValueBool) -> Self::Output {
                Self($trait::$method(self.0, rhs.0))
            }
        }
    };
}

impl_op!(BitAnd, bitand);
impl_op!(BitOr, bitor);
impl_op!(BitXor, bitxor);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display() {
        assert_eq!(ValueBool::TRUE.to_string(), "true");
        assert_eq!(ValueBool::FALSE.to_string(), "false");
    }

    #[test]
    fn conversions() {
        assert_eq!(ValueBool::from(true), ValueBool::TRUE);
        assert_eq!(bool::from(ValueBool::TRUE), true);
        assert_eq!(ValueBool::TRUE.get(), true);
    }

    #[test]
    fn default_is_false() {
        assert_eq!(ValueBool::default(), ValueBool::FALSE);
    }

    #[test]
    fn logical_ops() {
        assert_eq!(!ValueBool::TRUE, ValueBool::FALSE);
        assert_eq!(ValueBool::TRUE & ValueBool::FALSE, ValueBool::FALSE);
        assert_eq!(ValueBool::TRUE | ValueBool::FALSE, ValueBool::TRUE);
        assert_eq!(ValueBool::TRUE ^ ValueBool::TRUE, ValueBool::FALSE);
        // `&`-rhs forms delegate to the same logic.
        assert_eq!(ValueBool::TRUE & &ValueBool::TRUE, ValueBool::TRUE);
    }
}

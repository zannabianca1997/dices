//! Integer type

use std::{
    cmp::Reverse,
    fmt::{self, Display},
    hash::Hash,
    num::IntErrorKind,
    ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Rem, RemAssign, Sub, SubAssign},
    str::FromStr,
    sync::Arc,
};

use num::{
    BigInt, BigUint, FromPrimitive, Integer, Num, NumCast, One, Signed, ToPrimitive, Zero,
    bigint::{RandBigInt, Sign},
    traits::{ConstOne, ConstZero},
};
use rand::distributions::uniform::{SampleBorrow, SampleUniform, UniformSampler};
use snafu::{ResultExt, Snafu};

/// A boundless integer
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct ValueInt(Inner);

/// Content of an integer
///
/// This is _aggressively_ inlined, as it is an error to ever have a
/// `Inner::Heap` that does fit in a `i64`
// This does a double dereference (Inner -> Arc -> BigInt inner vec) but
// makes inner smaller and we do not expect a lot of big integers
#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
enum Inner {
    /// Heap allocated negative big integer
    BigNegative(Reverse<Arc<BigUint>>),
    /// Inline small integer
    ///
    /// we expect most of the integers being like this
    Inline(i64),
    /// Heap allocated positive big integer
    BigPositive(Arc<BigUint>),
}

impl ValueInt {
    /// Materialize the value as a [`BigInt`].
    ///
    /// Cheap for the heap variants (an [`Arc`] clone of the magnitude), and
    /// only reached on the cold path where an inline operation overflowed an
    /// `i64`.
    fn to_bigint(&self) -> BigInt {
        match &self.0 {
            Inner::Inline(a) => BigInt::from(*a),
            Inner::BigPositive(m) => BigInt::from_biguint(Sign::Plus, (**m).clone()),
            Inner::BigNegative(Reverse(m)) => BigInt::from_biguint(Sign::Minus, (**m).clone()),
        }
    }

    /// Build a [`ValueInt`] from a [`BigInt`], preserving the inline invariant.
    ///
    /// A heap variant is only ever produced when the value genuinely does not
    /// fit in an `i64`.
    fn from_bigint(b: BigInt) -> Self {
        if let Some(i) = b.to_i64() {
            return Self(Inner::Inline(i));
        }
        let (sign, mag) = b.into_parts();
        match sign {
            Sign::Plus => Self(Inner::BigPositive(Arc::new(mag))),
            Sign::Minus => Self(Inner::BigNegative(Reverse(Arc::new(mag)))),
            // 0 always fits in an `i64`, so it is handled above.
            Sign::NoSign => Self(Inner::Inline(0)),
        }
    }

    fn add_ref(&self, rhs: &Self) -> Self {
        if let (Inner::Inline(a), Inner::Inline(b)) = (&self.0, &rhs.0)
            && let Some(r) = a.checked_add(*b)
        {
            return Self(Inner::Inline(r));
        }
        Self::from_bigint(self.to_bigint() + rhs.to_bigint())
    }

    fn sub_ref(&self, rhs: &Self) -> Self {
        if let (Inner::Inline(a), Inner::Inline(b)) = (&self.0, &rhs.0)
            && let Some(r) = a.checked_sub(*b)
        {
            return Self(Inner::Inline(r));
        }
        Self::from_bigint(self.to_bigint() - rhs.to_bigint())
    }

    fn mul_ref(&self, rhs: &Self) -> Self {
        if let (Inner::Inline(a), Inner::Inline(b)) = (&self.0, &rhs.0)
            && let Some(r) = a.checked_mul(*b)
        {
            return Self(Inner::Inline(r));
        }
        Self::from_bigint(self.to_bigint() * rhs.to_bigint())
    }

    fn div_ref(&self, rhs: &Self) -> Self {
        // `checked_div` is `None` on division by zero (the BigInt path then
        // panics, matching primitive `/`) and on `i64::MIN / -1` (the BigInt
        // path yields the correct big result).
        if let (Inner::Inline(a), Inner::Inline(b)) = (&self.0, &rhs.0)
            && let Some(r) = a.checked_div(*b)
        {
            return Self(Inner::Inline(r));
        }
        Self::from_bigint(self.to_bigint() / rhs.to_bigint())
    }

    fn rem_ref(&self, rhs: &Self) -> Self {
        if let (Inner::Inline(a), Inner::Inline(b)) = (&self.0, &rhs.0)
            && let Some(r) = a.checked_rem(*b)
        {
            return Self(Inner::Inline(r));
        }
        Self::from_bigint(self.to_bigint() % rhs.to_bigint())
    }
}

impl PartialEq<ValueInt> for &ValueInt {
    fn eq(&self, other: &ValueInt) -> bool {
        <&ValueInt as PartialEq>::eq(self, &other)
    }
}
impl PartialEq<&ValueInt> for ValueInt {
    fn eq(&self, other: &&ValueInt) -> bool {
        <&ValueInt as PartialEq>::eq(&self, other)
    }
}

impl Zero for ValueInt {
    fn zero() -> Self {
        Self::ZERO
    }

    fn is_zero(&self) -> bool {
        self == Self::ZERO
    }
}

impl Default for Inner {
    fn default() -> Self {
        ValueInt::ZERO.0
    }
}

impl ConstZero for ValueInt {
    const ZERO: Self = Self(Inner::Inline(0));
}

impl One for ValueInt {
    fn one() -> Self {
        Self::ONE
    }
}

impl ConstOne for ValueInt {
    const ONE: Self = Self(Inner::Inline(1));
}

/// Implement a binary operator (owned and `&`-rhs) in terms of an `*_ref`
/// helper, mirroring `impl_assign`.
macro_rules! impl_op {
    ($trait:ident, $method:ident, $helper:ident) => {
        impl $trait for ValueInt {
            type Output = ValueInt;

            fn $method(self, rhs: Self) -> Self::Output {
                self.$helper(&rhs)
            }
        }

        impl $trait<&ValueInt> for ValueInt {
            type Output = ValueInt;

            fn $method(self, rhs: &ValueInt) -> Self::Output {
                self.$helper(rhs)
            }
        }
    };
}

impl_op!(Add, add, add_ref);
impl_op!(Sub, sub, sub_ref);
impl_op!(Mul, mul, mul_ref);
impl_op!(Div, div, div_ref);
impl_op!(Rem, rem, rem_ref);

impl Neg for ValueInt {
    type Output = Self;

    fn neg(self) -> Self::Output {
        match &self.0 {
            Inner::Inline(a) => match a.checked_neg() {
                Some(v) => Self(Inner::Inline(v)),
                // `i64::MIN`, whose magnitude does not fit in an `i64`.
                None => Self::from_bigint(-self.to_bigint()),
            },
            // A `BigNegative` magnitude is always `> 2^63`, so negating it is
            // always a valid heap positive: reuse the `Arc`.
            Inner::BigNegative(Reverse(m)) => Self(Inner::BigPositive(m.clone())),
            // A `BigPositive` may hold exactly `2^63`, whose negation is
            // `i64::MIN` and re-inlines; normalize to be safe.
            Inner::BigPositive(_) => Self::from_bigint(-self.to_bigint()),
        }
    }
}

macro_rules! impl_assign {
    ($trait:ident, $method:ident, $helper:ident) => {
        impl $trait for ValueInt {
            fn $method(&mut self, rhs: Self) {
                *self = self.$helper(&rhs);
            }
        }

        impl $trait<&ValueInt> for ValueInt {
            fn $method(&mut self, rhs: &ValueInt) {
                *self = self.$helper(rhs);
            }
        }
    };
}

impl_assign!(AddAssign, add_assign, add_ref);
impl_assign!(SubAssign, sub_assign, sub_ref);
impl_assign!(MulAssign, mul_assign, mul_ref);
impl_assign!(DivAssign, div_assign, div_ref);
impl_assign!(RemAssign, rem_assign, rem_ref);

impl Signed for ValueInt {
    fn abs(&self) -> Self {
        match &self.0 {
            Inner::Inline(a) => match a.checked_abs() {
                Some(v) => Self(Inner::Inline(v)),
                // `i64::MIN`, whose magnitude does not fit in an `i64`.
                None => Self::from_bigint(self.to_bigint().abs()),
            },
            // Magnitude is already a valid heap value, reuse the `Arc`.
            Inner::BigPositive(_) => self.clone(),
            Inner::BigNegative(Reverse(m)) => Self(Inner::BigPositive(m.clone())),
        }
    }

    fn abs_sub(&self, other: &Self) -> Self {
        if self <= other {
            Self::ZERO
        } else {
            self.sub_ref(other)
        }
    }

    fn signum(&self) -> Self {
        match &self.0 {
            Inner::Inline(a) => Self(Inner::Inline(a.signum())),
            Inner::BigPositive(_) => Self::ONE,
            Inner::BigNegative(_) => Self(Inner::Inline(-1)),
        }
    }

    fn is_positive(&self) -> bool {
        match &self.0 {
            Inner::Inline(a) => *a > 0,
            Inner::BigPositive(_) => true,
            Inner::BigNegative(_) => false,
        }
    }

    fn is_negative(&self) -> bool {
        match &self.0 {
            Inner::Inline(a) => *a < 0,
            Inner::BigPositive(_) => false,
            Inner::BigNegative(_) => true,
        }
    }
}

impl ToPrimitive for ValueInt {
    fn to_i64(&self) -> Option<i64> {
        match &self.0 {
            Inner::Inline(a) => Some(*a),
            // By the inline invariant the heap variants never fit in an `i64`.
            _ => None,
        }
    }

    fn to_u64(&self) -> Option<u64> {
        match &self.0 {
            Inner::Inline(a) => a.to_u64(),
            // Magnitudes in `2^63..2^64` fit in a `u64` but not an `i64`.
            Inner::BigPositive(m) => m.to_u64(),
            Inner::BigNegative(_) => None,
        }
    }

    fn to_i128(&self) -> Option<i128> {
        match &self.0 {
            Inner::Inline(a) => Some(*a as i128),
            _ => self.to_bigint().to_i128(),
        }
    }

    fn to_u128(&self) -> Option<u128> {
        match &self.0 {
            Inner::Inline(a) => a.to_u128(),
            Inner::BigPositive(m) => m.to_u128(),
            Inner::BigNegative(_) => None,
        }
    }

    fn to_f64(&self) -> Option<f64> {
        if let Inner::Inline(a) = &self.0 {
            Some(*a as f64)
        } else {
            self.to_bigint().to_f64()
        }
    }
}

impl FromPrimitive for ValueInt {
    fn from_i64(n: i64) -> Option<Self> {
        Some(Self(Inner::Inline(n)))
    }

    fn from_u64(n: u64) -> Option<Self> {
        Some(if let Some(a) = n.to_i64() {
            Self(Inner::Inline(a))
        } else {
            Self::from_bigint(BigInt::from(n))
        })
    }

    fn from_usize(n: usize) -> Option<Self> {
        Some(if let Some(a) = n.to_i64() {
            Self(Inner::Inline(a))
        } else {
            Self::from_bigint(BigInt::from(n))
        })
    }

    fn from_i128(n: i128) -> Option<Self> {
        Some(if let Some(a) = n.to_i64() {
            Self(Inner::Inline(a))
        } else {
            Self::from_bigint(BigInt::from(n))
        })
    }

    fn from_u128(n: u128) -> Option<Self> {
        Some(if let Some(a) = n.to_i64() {
            Self(Inner::Inline(a))
        } else {
            Self::from_bigint(BigInt::from(n))
        })
    }

    fn from_f32(n: f32) -> Option<Self> {
        BigInt::from_f32(n).map(Self::from_bigint)
    }

    fn from_f64(n: f64) -> Option<Self> {
        BigInt::from_f64(n).map(Self::from_bigint)
    }
}

impl NumCast for ValueInt {
    fn from<T: ToPrimitive>(n: T) -> Option<Self> {
        if let Some(i) = n.to_i64() {
            return Some(Self(Inner::Inline(i)));
        }
        if let Some(i) = n.to_i128() {
            return Some(Self::from_bigint(BigInt::from(i)));
        }
        if let Some(u) = n.to_u128() {
            return Some(Self::from_bigint(BigInt::from(u)));
        }
        None
    }
}

impl Display for ValueInt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.0 {
            Inner::Inline(a) => write!(f, "{a}"),
            Inner::BigPositive(m) => write!(f, "{m}"),
            Inner::BigNegative(Reverse(m)) => write!(f, "-{m}"),
        }
    }
}

#[derive(Debug, Snafu)]
pub enum FromStrRadixErr {
    Inline {
        source: <i64 as Num>::FromStrRadixErr,
    },
    Heap {
        source: <BigInt as Num>::FromStrRadixErr,
    },
}

impl Num for ValueInt {
    type FromStrRadixErr = FromStrRadixErr;

    fn from_str_radix(str: &str, radix: u32) -> Result<Self, Self::FromStrRadixErr> {
        match i64::from_str_radix(str, radix) {
            Ok(v) => Ok(Self(Inner::Inline(v))),
            // Only retry on the big path when the string was a valid number that
            // simply overflowed `i64`; genuine parse errors stay inline errors.
            Err(e)
                if matches!(
                    e.kind(),
                    IntErrorKind::PosOverflow | IntErrorKind::NegOverflow
                ) =>
            {
                let big = BigInt::from_str_radix(str, radix).context(HeapSnafu)?;
                Ok(Self::from_bigint(big))
            }
            Err(source) => Err(source).context(InlineSnafu),
        }
    }
}

impl FromStr for ValueInt {
    type Err = FromStrRadixErr;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_str_radix(s, 10)
    }
}

impl Integer for ValueInt {
    fn div_floor(&self, other: &Self) -> Self {
        if let (Inner::Inline(a), Inner::Inline(b)) = (&self.0, &other.0)
            && *b != 0
            && !(*a == i64::MIN && *b == -1)
        {
            return Self(Inner::Inline(a.div_floor(b)));
        }
        Self::from_bigint(self.to_bigint().div_floor(&other.to_bigint()))
    }

    fn mod_floor(&self, other: &Self) -> Self {
        if let (Inner::Inline(a), Inner::Inline(b)) = (&self.0, &other.0)
            && *b != 0
            && !(*a == i64::MIN && *b == -1)
        {
            return Self(Inner::Inline(a.mod_floor(b)));
        }
        Self::from_bigint(self.to_bigint().mod_floor(&other.to_bigint()))
    }

    fn gcd(&self, other: &Self) -> Self {
        Self::from_bigint(self.to_bigint().gcd(&other.to_bigint()))
    }

    fn lcm(&self, other: &Self) -> Self {
        Self::from_bigint(self.to_bigint().lcm(&other.to_bigint()))
    }

    fn is_multiple_of(&self, other: &Self) -> bool {
        self.to_bigint().is_multiple_of(&other.to_bigint())
    }

    fn is_even(&self) -> bool {
        match &self.0 {
            Inner::Inline(a) => *a & 1 == 0,
            // Parity of the value matches the parity of its magnitude.
            Inner::BigPositive(m) | Inner::BigNegative(Reverse(m)) => m.is_even(),
        }
    }

    fn is_odd(&self) -> bool {
        !self.is_even()
    }

    fn div_rem(&self, other: &Self) -> (Self, Self) {
        if let (Inner::Inline(a), Inner::Inline(b)) = (&self.0, &other.0)
            && let (Some(q), Some(r)) = (a.checked_div(*b), a.checked_rem(*b))
        {
            return (Self(Inner::Inline(q)), Self(Inner::Inline(r)));
        }
        let (q, r) = self.to_bigint().div_rem(&other.to_bigint());
        (Self::from_bigint(q), Self::from_bigint(r))
    }

    fn dec(&mut self) {
        match &mut self.0 {
            Inner::BigNegative(Reverse(magnitude)) => Arc::make_mut(magnitude).inc(),
            Inner::Inline(value) => {
                if let Some(dec) = value.checked_sub(1) {
                    *value = dec
                } else {
                    // fell out of inline range

                    let mut magnitude = BigInt::from(*value);
                    magnitude.dec();
                    *self = Self::from_bigint(magnitude)
                }
            }
            Inner::BigPositive(magnitude) => {
                let magnitude = Arc::make_mut(magnitude);
                magnitude.dec();
                if let Some(magnitude) = magnitude.to_i64() {
                    // fell into inline range

                    self.0 = Inner::Inline(magnitude)
                }
            }
        }
    }
}

impl SampleUniform for ValueInt {
    type Sampler = Sampler;
}

pub struct Sampler(SamplerInner);
enum SamplerInner {
    Small(<i64 as SampleUniform>::Sampler),
    Big { lbound: BigInt, ubound: BigInt },
}

impl UniformSampler for Sampler {
    type X = ValueInt;

    fn new<B1, B2>(low: B1, high: B2) -> Self
    where
        B1: SampleBorrow<Self::X> + Sized,
        B2: SampleBorrow<Self::X> + Sized,
    {
        Self(
            if let (Inner::Inline(low), Inner::Inline(high)) = (&low.borrow().0, &high.borrow().0) {
                SamplerInner::Small(UniformSampler::new(*low, *high))
            } else {
                let lbound = low.borrow().to_bigint();
                let ubound = high.borrow().to_bigint();

                SamplerInner::Big { lbound, ubound }
            },
        )
    }

    fn new_inclusive<B1, B2>(low: B1, high: B2) -> Self
    where
        B1: SampleBorrow<Self::X> + Sized,
        B2: SampleBorrow<Self::X> + Sized,
    {
        Self(
            if let (Inner::Inline(low), Inner::Inline(high)) = (&low.borrow().0, &high.borrow().0) {
                SamplerInner::Small(UniformSampler::new_inclusive(*low, *high))
            } else {
                let lbound = low.borrow().to_bigint();
                let ubound = high.borrow().to_bigint() + 1;

                SamplerInner::Big { lbound, ubound }
            },
        )
    }

    fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> Self::X {
        match &self.0 {
            SamplerInner::Small(sampler) => ValueInt(Inner::Inline(sampler.sample(rng))),
            SamplerInner::Big { lbound, ubound } => {
                ValueInt::from_bigint(RandBigInt::gen_bigint_range(rng, lbound, ubound))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn vi(n: i64) -> ValueInt {
        ValueInt(Inner::Inline(n))
    }

    #[test]
    fn inline_heap_boundary() {
        let over = vi(i64::MAX) + vi(1);
        assert!(matches!(over.0, Inner::BigPositive(_)));
        assert_eq!(over.to_string(), "9223372036854775808");

        // Coming back down across the boundary must re-inline.
        let back = over - vi(1);
        assert!(matches!(back.0, Inner::Inline(_)));
        assert_eq!(back, vi(i64::MAX));
    }

    #[test]
    fn neg_and_abs_of_min() {
        let min = vi(i64::MIN);

        let neg = -min.clone();
        assert!(matches!(neg.0, Inner::BigPositive(_)));
        assert_eq!(neg.to_string(), "9223372036854775808");

        let abs = min.abs();
        assert!(matches!(abs.0, Inner::BigPositive(_)));
        assert_eq!(abs, neg);

        // Round trip back through negation re-inlines.
        assert_eq!(-neg, min);
    }

    #[test]
    fn div_rem_truncates_floor_rounds() {
        let a = vi(-7);
        let b = vi(2);

        assert_eq!(a.clone() / b.clone(), vi(-3));
        assert_eq!(a.clone() % b.clone(), vi(-1));

        assert_eq!(a.div_floor(&b), vi(-4));
        assert_eq!(a.mod_floor(&b), vi(1));

        assert_eq!(a.div_rem(&b), (vi(-3), vi(-1)));
    }

    #[test]
    fn to_u64_in_unsigned_only_range() {
        let v = vi(i64::MAX) + vi(1); // 2^63
        assert!(matches!(v.0, Inner::BigPositive(_)));
        assert_eq!(v.to_u64(), Some(9_223_372_036_854_775_808));
        assert_eq!(v.to_i64(), None);
    }

    #[test]
    fn parsing_across_the_boundary() {
        let small: ValueInt = "42".parse().unwrap();
        assert_eq!(small, vi(42));
        assert!(matches!(small.0, Inner::Inline(_)));

        let big = "99999999999999999999999999".parse::<ValueInt>().unwrap();
        assert!(matches!(big.0, Inner::BigPositive(_)));
        assert_eq!(big.to_string(), "99999999999999999999999999");

        let neg_big = "-99999999999999999999999999".parse::<ValueInt>().unwrap();
        assert!(matches!(neg_big.0, Inner::BigNegative(_)));

        assert!("not a number".parse::<ValueInt>().is_err());
    }

    #[test]
    fn parity() {
        assert!(vi(4).is_even());
        assert!(vi(-3).is_odd());

        let big_even = vi(i64::MAX) + vi(1); // 2^63, even
        assert!(big_even.is_even());
        let big_odd = big_even + vi(1);
        assert!(big_odd.is_odd());
    }

    #[test]
    fn ref_arithmetic_and_assign() {
        let mut x = vi(10);
        x += &vi(5);
        x *= vi(2);
        assert_eq!(x, vi(30));
        assert_eq!(vi(30) + &vi(12), vi(42));
    }

    #[test]
    #[should_panic]
    fn division_by_zero_panics() {
        let _ = vi(1) / vi(0);
    }
}

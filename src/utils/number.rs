//! Traits regarding numbers.
use crate::utils::{One, Zero};
use core::fmt::{Debug, Display};
use core::hash::Hash;
use core::ops::{
    Add, AddAssign, BitAnd, BitAndAssign, BitOr, BitOrAssign, Div, Mul, Neg, Not, Rem, Shl, Shr,
    Sub, SubAssign,
};

/// A trait for numbers.
pub trait Number:
    Copy
    + Add<Self, Output = Self>
    + Sub<Self, Output = Self>
    + Div<Self, Output = Self>
    + Mul<Self, Output = Self>
    + AddAssign
    + SubAssign
    + Debug
    + Display
    + Zero
    + One
    + From<bool>
    + PartialOrd
    + Send
    + Sync
{
    #[must_use]
    /// A method to subtract the second number from the first number, returning zero if the result is negative.
    fn saturating_zero_sub(self, other: Self) -> Self;
}

/// A trait for positive integer numbers.
pub trait PositiveInteger:
    Number
    + Eq
    + Into<u64>
    + From<u8>
    + BitAnd<Output = Self>
    + BitAndAssign
    + BitOr
    + BitOrAssign
    + Ord
    + Rem<Output = Self>
    + Shl<u8, Output = Self>
    + Shr<u8, Output = Self>
    + Hash
    + Not<Output = Self>
{
}

/// A trait for floating point numbers.
pub(crate) trait FloatOps: Number + Neg<Output = Self> {
    /// Returns the value of 2^(-register), with strict positivite register.
    fn integer_exp2_minus(register: u8) -> Self;

    /// Returns the value of 2^(-register), including negative registers.
    fn integer_exp2_minus_signed(register: i16) -> Self;

    /// Returns the value of 2^(register)
    fn integer_exp2(register: u8) -> Self;

    /// Returns the natural logarithm of a strictly positive value, computed without `std` (the core
    /// crate is `no_std` and transcendental-free outside the `mle` feature).
    fn natural_log(self) -> Self;

    /// Returns `e^self`, computed without `std`. Used by the hash-list occupancy estimator to
    /// evaluate the survival term `(1 - p)^n = exp(n * ln(1 - p))`.
    fn exp(self) -> Self;

    /// Returns the square root of a non-negative value, computed without `std`.
    fn sqrt(self) -> Self;

    /// Returns the absolute value, computed without `std`.
    fn abs(self) -> Self;

    /// Returns the largest integer less than or equal to `self`, computed without `std`.
    fn floor(self) -> Self;

    /// Returns the base-2 logarithm of a strictly positive value, computed without `std`.
    fn log2(self) -> Self;

    /// Returns `self` raised to an integer power, computed without `std`.
    fn powi(self, exponent: i32) -> Self;

    /// Returns `ln(1 + self)` accurately for small `self`, computed without `std`.
    fn ln_1p(self) -> Self;

    /// Returns `e^self - 1` accurately for small `self`, computed without `std`.
    fn exp_m1(self) -> Self;

    /// Returns the larger of `self` and `other` (assuming neither is NaN), computed without `std`.
    fn maximum(self, other: Self) -> Self;

    #[must_use]
    #[inline]
    /// Computes the saturating division of two numbers that are expected to be positive.
    /// and at most equal to one.
    fn saturating_one_div(self, other: Self) -> Self {
        debug_assert!(self >= Self::ZERO, "The dividend must be positive.");
        debug_assert!(other >= Self::ZERO, "The divisor must be positive.");
        if self >= other {
            Self::ONE
        } else {
            self / other
        }
    }
}

/// A trait for numbers.
macro_rules! impl_number {
    ($($t:ty),*) => {
        $(
            impl Number for $t {
                #[inline]
                fn saturating_zero_sub(self, other: Self) -> Self {
                    debug_assert!(self >= Self::ZERO, "The first number must be positive, got: {}", self);
                    debug_assert!(other >= Self::ZERO, "The second number must be positive, got: {}", other);
                    if self < other {
                        Self::ZERO
                    } else {
                        self - other
                    }
                }
            }
        )*
    };
}

impl_number!(u8, u16, u32, u64, usize);
impl_number!(i32);
impl_number!(f64);

/// A trait for signed numbers.
macro_rules! impl_positive_integer_number {
    ($($t:ty),*) => {
        $(
            impl PositiveInteger for $t {
            }
        )*
    };
}

impl_positive_integer_number!(u8, u16, u32, u64);

impl FloatOps for f64 {
    #[inline]
    fn integer_exp2_minus(register: u8) -> Self {
        f64::from_le_bytes((u64::from(1023_u16 - u16::from(register)) << 52).to_le_bytes())
    }

    #[inline]
    fn integer_exp2_minus_signed(register: i16) -> Self {
        debug_assert!(
            register > -1024,
            "The register must be greater than -1024, got: {register}",
        );
        f64::from_le_bytes((u64::try_from(1023_i16 - register).unwrap() << 52).to_le_bytes())
    }

    #[inline]
    fn integer_exp2(register: u8) -> Self {
        f64::from_le_bytes((u64::from(1023_u16 + u16::from(register)) << 52).to_le_bytes())
    }

    #[inline]
    fn natural_log(self) -> Self {
        debug_assert!(
            self > 0.0,
            "natural_log requires a strictly positive argument, got {self}"
        );
        // Decompose self = mantissa * 2^exponent with mantissa in [1, 2).
        let bits = self.to_bits();
        let mut exponent = ((bits >> 52) & 0x7ff) as i64 - 1023;
        let mut mantissa = f64::from_bits((bits & 0x000f_ffff_ffff_ffff) | 0x3ff0_0000_0000_0000);
        // Center the mantissa around 1 (into [1/sqrt(2), sqrt(2))) so the series converges fast.
        if mantissa > core::f64::consts::SQRT_2 {
            mantissa *= 0.5;
            exponent += 1;
        }
        // ln(mantissa) via the atanh series with s = (m - 1) / (m + 1):
        // ln(m) = 2 (s + s^3/3 + s^5/5 + ...). |s| <= 0.172 here, so the odd reciprocals up to
        // 1/15 (Horner over s^2) reach full f64 accuracy across our [1, 2^18] argument range.
        let s = (mantissa - 1.0) / (mantissa + 1.0);
        let s2 = s * s;
        let mut series = 1.0 / 15.0;
        series = series * s2 + 1.0 / 13.0;
        series = series * s2 + 1.0 / 11.0;
        series = series * s2 + 1.0 / 9.0;
        series = series * s2 + 1.0 / 7.0;
        series = series * s2 + 1.0 / 5.0;
        series = series * s2 + 1.0 / 3.0;
        series = series * s2 + 1.0;
        exponent as f64 * core::f64::consts::LN_2 + 2.0 * s * series
    }

    #[inline]
    fn exp(self) -> Self {
        // exp(x) = 2^(x * log2 e) = 2^k * 2^f, with k = floor(y) and the fraction f in [0, 1).
        let y = self * core::f64::consts::LOG2_E;
        // Underflow guard: below this the result is zero to f64 precision, and the exponent assembly
        // below would otherwise go out of range. The occupancy estimator only feeds x <= 0.
        if y <= -1022.0 {
            return 0.0;
        }
        // floor(y) without `std`: truncation rounds toward zero, so step down for negative fractions.
        let mut k = y as i64;
        if (k as f64) > y {
            k -= 1;
        }
        let f = y - k as f64; // in [0, 1)
                              // 2^f = exp(f * ln 2) with the argument in [0, ln 2); Taylor over the factorials (Horner)
                              // reaches full f64 accuracy on this short interval.
        let t = f * core::f64::consts::LN_2;
        let mut series = 1.0 / 479_001_600.0; // 1/12!
        series = series * t + 1.0 / 39_916_800.0; // 1/11!
        series = series * t + 1.0 / 3_628_800.0; // 1/10!
        series = series * t + 1.0 / 362_880.0; // 1/9!
        series = series * t + 1.0 / 40_320.0; // 1/8!
        series = series * t + 1.0 / 5_040.0; // 1/7!
        series = series * t + 1.0 / 720.0; // 1/6!
        series = series * t + 1.0 / 120.0; // 1/5!
        series = series * t + 1.0 / 24.0; // 1/4!
        series = series * t + 1.0 / 6.0; // 1/3!
        series = series * t + 0.5; // 1/2!
        series = series * t + 1.0; // 1/1!
        series = series * t + 1.0; // 1/0!
        let two_k = if k >= 0 {
            f64::integer_exp2(k as u8)
        } else {
            f64::integer_exp2_minus_signed(-k as i16)
        };
        two_k * series
    }

    #[inline]
    fn sqrt(self) -> Self {
        if self < 0.0 {
            return f64::NAN;
        }
        if self == 0.0 || !self.is_finite() {
            return self;
        }
        // Decompose self = mantissa * 2^exponent with mantissa in [1, 2), then sqrt = sqrt(mantissa)
        // * 2^(exponent/2). Fold an odd exponent into the mantissa so the power of two is integral.
        let bits = self.to_bits();
        let mut exponent = ((bits >> 52) & 0x7ff) as i64 - 1023;
        let mut mantissa = f64::from_bits((bits & 0x000f_ffff_ffff_ffff) | 0x3ff0_0000_0000_0000);
        if exponent & 1 != 0 {
            mantissa *= 2.0; // now in [2, 4)
            exponent -= 1; // now even
        }
        // Newton's method for sqrt(mantissa) with mantissa in [1, 4): quadratic convergence reaches
        // full f64 precision from the fixed seed 1.5 in a handful of steps.
        let mut root = 1.5;
        root = 0.5 * (root + mantissa / root);
        root = 0.5 * (root + mantissa / root);
        root = 0.5 * (root + mantissa / root);
        root = 0.5 * (root + mantissa / root);
        root = 0.5 * (root + mantissa / root);
        root = 0.5 * (root + mantissa / root);
        let half_exponent = exponent / 2;
        let scale = if half_exponent >= 0 {
            f64::integer_exp2(half_exponent as u8)
        } else {
            f64::integer_exp2_minus_signed((-half_exponent) as i16)
        };
        root * scale
    }

    #[inline]
    fn abs(self) -> Self {
        f64::from_bits(self.to_bits() & 0x7fff_ffff_ffff_ffff)
    }

    #[inline]
    fn floor(self) -> Self {
        // Truncation toward zero, stepped down by one for negative non-integers.
        let truncated = self as i64 as f64;
        if truncated > self {
            truncated - 1.0
        } else {
            truncated
        }
    }

    #[inline]
    fn log2(self) -> Self {
        self.natural_log() * core::f64::consts::LOG2_E
    }

    #[inline]
    fn powi(self, exponent: i32) -> Self {
        if exponent < 0 {
            return 1.0 / FloatOps::powi(self, -exponent);
        }
        // Exponentiation by squaring.
        let mut result = 1.0;
        let mut base = self;
        let mut e = exponent as u32;
        while e > 0 {
            if e & 1 == 1 {
                result *= base;
            }
            e >>= 1;
            if e > 0 {
                base *= base;
            }
        }
        result
    }

    #[inline]
    fn ln_1p(self) -> Self {
        // ln(1 + x) with Kahan's correction for the rounding in `1 + x`, so small `x` keeps full
        // precision (where `ln((1 + x))` alone would lose it).
        let u = 1.0 + self;
        if u == 1.0 {
            return self;
        }
        let ln_u = u.natural_log();
        let denominator = u - 1.0;
        if denominator == self {
            ln_u
        } else {
            ln_u * (self / denominator)
        }
    }

    #[inline]
    fn exp_m1(self) -> Self {
        // e^x - 1 refined as (u - 1) * x / ln(u) with u = e^x (the standard libm trick), which keeps
        // full precision for small `x` where `e^x - 1` alone would cancel catastrophically.
        let u = FloatOps::exp(self);
        if u == 1.0 {
            return self;
        }
        let u_minus_one = u - 1.0;
        if u_minus_one == -1.0 {
            return -1.0;
        }
        u_minus_one * (self / u.natural_log())
    }

    #[inline]
    fn maximum(self, other: Self) -> Self {
        if self >= other {
            self
        } else {
            other
        }
    }
}

#[cfg(test)]
mod test_natural_log {
    use super::*;

    #[test]
    fn test_natural_log_matches_std() {
        // The no_std atanh-series implementation must match std's ln across the range of arguments
        // linear counting produces, m / zeros for m up to 2^18 and zeros in [1, m], i.e. [1, 2^18].
        let mut value = 1.0_f64;
        while value <= (1u64 << 18) as f64 {
            let approx = value.natural_log();
            let exact = value.ln();
            assert!(
                (approx - exact).abs() <= 1e-9 * exact.abs().max(1.0),
                "natural_log({value}) = {approx}, std ln = {exact}"
            );
            value *= 1.0009765625; // 1 + 2^-10, a fine geometric sweep
        }
        // Exact landmarks.
        assert!((1.0_f64.natural_log()).abs() < 1e-12);
        assert!((core::f64::consts::E.natural_log() - 1.0).abs() < 1e-12);
    }
}

#[cfg(test)]
mod test_exp {
    use super::*;

    #[test]
    fn test_exp_matches_std() {
        // The occupancy estimator evaluates exp on n * ln(1 - p) <= 0; sweep that range densely and a
        // little above zero for good measure, against std's exp.
        let mut x = -40.0_f64;
        while x <= 5.0 {
            let approx = FloatOps::exp(x);
            let exact = x.exp();
            assert!(
                (approx - exact).abs() <= 1e-9 * exact.abs().max(1e-12),
                "exp({x}) = {approx}, std exp = {exact}"
            );
            x += 0.001;
        }
        // Exact landmarks and the underflow guard.
        assert!((FloatOps::exp(0.0_f64) - 1.0).abs() < 1e-12);
        assert!((FloatOps::exp(1.0_f64) - core::f64::consts::E).abs() < 1e-12);
        assert_eq!(FloatOps::exp(-2000.0_f64), 0.0);
    }
}

#[cfg(test)]
mod test_float_ops_primitives {
    use super::*;

    #[test]
    fn test_sqrt_matches_std() {
        // Sweep both magnitudes (odd and even exponents) and the fractional range against std.
        let mut x = 1e-9_f64;
        while x <= 1e12 {
            let approx = FloatOps::sqrt(x);
            let exact = x.sqrt();
            assert!(
                (approx - exact).abs() <= 1e-12 * exact.max(1e-12),
                "sqrt({x}) = {approx}, std = {exact}"
            );
            x *= 1.0009765625;
        }
        assert_eq!(FloatOps::sqrt(0.0_f64), 0.0);
        assert_eq!(FloatOps::sqrt(1.0_f64), 1.0);
        assert_eq!(FloatOps::sqrt(4.0_f64), 2.0);
    }

    #[test]
    fn test_abs_floor_log2_powi_match_std() {
        for &v in &[-3.7_f64, -1.0, -0.25, 0.0, 0.25, 1.0, 3.7, 1024.5, -1024.5] {
            assert_eq!(FloatOps::abs(v), v.abs(), "abs({v})");
            assert_eq!(FloatOps::floor(v), v.floor(), "floor({v})");
        }
        let mut x = 1e-6_f64;
        while x <= 1e9 {
            assert!(
                (FloatOps::log2(x) - x.log2()).abs() <= 1e-9 * x.log2().abs().max(1.0),
                "log2({x})"
            );
            x *= 1.01;
        }
        for base in [0.5_f64, 1.5, 2.0, 10.0, 0.1] {
            for exp in -6..=6 {
                assert!(
                    (FloatOps::powi(base, exp) - base.powi(exp)).abs()
                        <= 1e-9 * base.powi(exp).abs().max(1e-12),
                    "powi({base}, {exp})"
                );
            }
        }
    }

    #[test]
    fn test_ln_1p_exp_m1_match_std() {
        // Sweep across small and large magnitudes, both signs, against std's accurate versions.
        let mut x = -0.9_f64;
        while x <= 50.0 {
            assert!(
                (FloatOps::ln_1p(x) - x.ln_1p()).abs() <= 1e-12 * x.ln_1p().abs().max(1e-12),
                "ln_1p({x})"
            );
            assert!(
                (FloatOps::exp_m1(x) - x.exp_m1()).abs() <= 1e-12 * x.exp_m1().abs().max(1e-12),
                "exp_m1({x})"
            );
            x += 0.0007;
        }
        // Tiny arguments where naive forms cancel: the refined forms stay near the underlying
        // natural_log/exp accuracy (about 1e-12 relative) rather than degrading to the cancellation
        // error of the naive `ln(1 + x)` / `exp(x) - 1`.
        for &x in &[1e-300_f64, 1e-18, 1e-12, -1e-12, 1e-6, -1e-6] {
            assert!((FloatOps::ln_1p(x) - x.ln_1p()).abs() <= 1e-10 * x.abs().max(1e-300));
            assert!((FloatOps::exp_m1(x) - x.exp_m1()).abs() <= 1e-10 * x.abs().max(1e-300));
        }
        assert_eq!(FloatOps::maximum(2.0_f64, 1.0), 2.0);
        assert_eq!(FloatOps::maximum(1.0_f64, 2.0), 2.0);
    }
}

#[cfg(test)]
mod test_integer_exp2_minus {
    use super::*;

    #[test]
    fn test_integer_exp2_minus() {
        // At the most, we create registers with 6 bits, which
        // means that the maximum values is 2^7 - 1 = 127.
        for bits in 1..=8 {
            for register_value in 0..(1 << bits) {
                assert_eq!(
                    2.0_f64.powf(-(register_value as f64)),
                    f64::integer_exp2_minus(register_value as u8),
                    "Expected: 2^(-{}), Got: {}",
                    register_value,
                    f64::integer_exp2_minus(register_value as u8)
                );
                assert_eq!(
                    f64::from_bits(
                        u64::max_value().wrapping_sub(u64::from(register_value as u64)) << 54 >> 2
                    ),
                    f64::integer_exp2_minus(register_value as u8),
                    "Expected: 2^(-{}), Got: {}",
                    register_value,
                    f64::integer_exp2_minus(register_value as u8)
                );
            }
        }
    }
}

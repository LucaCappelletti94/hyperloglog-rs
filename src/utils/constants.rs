//! Constants for common values.

/// The zero value for this type.
pub trait Zero {
    /// The zero value for this type.
    const ZERO: Self;
    /// Whether the value is zero.
    fn is_zero(&self) -> bool;
}

/// The half value for this type.
pub trait Half {
    /// The half value for this type.
    const HALF: Self;
}

/// The one value for this type.
pub trait One {
    /// The one value for this type.
    const ONE: Self;
    /// Whether the value is one.
    fn is_one(&self) -> bool;
}

/// The two value for this type.
pub trait Two {
    /// The two value for this type.
    const TWO: Self;
}

/// The three value for this type.
pub trait Three {
    /// The three value for this type.
    const THREE: Self;
}

/// The five value for this type.
pub trait Five {
    /// The five value for this type.
    const FIVE: Self;
}

/// The six value for this type.
pub trait Six {
    /// The six value for this type.
    const SIX: Self;
}

/// The seven value for this type.
pub trait Seven {
    /// The seven value for this type.
    const SEVEN: Self;
}

/// The eight value for this type.
pub trait Eight {
    /// The eight value for this type.
    const EIGHT: Self;
}

/// The nine value for this type.
pub trait Nine {
    /// The nine value for this type.
    const NINE: Self;
}

/// The ten value for this type.
pub trait Ten {
    /// The ten value for this type.
    const TEN: Self;
}

/// The one hundred value for this type.
pub trait OneHundred {
    /// The one hundred value for this type.
    const ONE_HUNDRED: Self;
}

/// The one thousand value for this type.
pub trait OneThousand {
    /// The one thousand value for this type.
    const ONE_THOUSAND: Self;
}

/// Macro implementing several constants for integers.
macro_rules! impl_constants {
    ($($t:ty)*) => ($(
        impl One for $t {
            const ONE: Self = 1;
            #[inline(always)]
            fn is_one(&self) -> bool { *self == 1 }
        }
        impl Zero for $t {
            const ZERO: Self = 0;
            #[inline(always)]
            fn is_zero(&self) -> bool { *self == 0 }
        }
        impl Two for $t {
            const TWO: Self = 2;
        }
        impl Three for $t {
            const THREE: Self = 3;
        }
        impl Five for $t {
            const FIVE: Self = 5;
        }
        impl Six for $t {
            const SIX: Self = 6;
        }
        impl Seven for $t {
            const SEVEN: Self = 7;
        }
        impl Eight for $t {
            const EIGHT: Self = 8;
        }
        impl Nine for $t {
            const NINE: Self = 9;
        }
        impl Ten for $t {
            const TEN: Self = 10;
        }
        impl OneHundred for $t {
            const ONE_HUNDRED: Self = 100;
        }
    )*)
}

impl_constants! { u8 u16 u32 u64 u128 usize }
impl_constants! { i8 i16 i32 i64 i128 isize }


impl Half for f64 {
    const HALF: Self = 0.5;
}
impl One for f64 {
    const ONE: Self = 1.0;
    #[inline]
    fn is_one(&self) -> bool { (*self - 1.0).abs() < Self::EPSILON }
}
impl Zero for f64 {
    const ZERO: Self = 0.0;
    #[inline]
    fn is_zero(&self) -> bool { *self == 0.0 }
}
impl Two for f64 {
    const TWO: Self = 2.0;
}
impl Three for f64 {
    const THREE: Self = 3.0;
}
impl Five for f64 {
    const FIVE: Self = 5.0;
}
impl Six for f64 {
    const SIX: Self = 6.0;
}
impl Seven for f64 {
    const SEVEN: Self = 7.0;
}
impl Eight for f64 {
    const EIGHT: Self = 8.0;
}
impl Nine for f64 {
    const NINE: Self = 9.0;
}
impl Ten for f64 {
    const TEN: Self = 10.0;
}
impl OneHundred for f64 {
    const ONE_HUNDRED: Self = 100.0;
}
impl OneThousand for f64 {
    const ONE_THOUSAND: Self = 1_000.0;
}
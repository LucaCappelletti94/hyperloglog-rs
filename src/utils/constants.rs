//! This module provides a trait to define the ONE const for all unsigned
//! types and the method is_one(). This is not a trait in the core library,
//! but we are aware that it is available in other crates - we do not intend
//! to use them as dependencies, as we want to keep the dependencies to the
//! very bare minimum.

pub trait Half {
    /// The half value for this type.
    const HALF: Self;
}

pub trait One {
    /// The one value for this type.
    const ONE: Self;
    /// Whether the value is one.
    fn is_one(&self) -> bool;
}

pub trait Zero {
    /// The zero value for this type.
    const ZERO: Self;
    /// Whether the value is zero.
    fn is_zero(&self) -> bool;
}

pub trait Two {
    /// The two value for this type.
    const TWO: Self;
}

pub trait Three {
    /// The three value for this type.
    const THREE: Self;
}

pub trait Five {
    /// The five value for this type.
    const FIVE: Self;
}

pub trait Six {
    /// The six value for this type.
    const SIX: Self;
}

pub trait Seven {
    /// The seven value for this type.
    const SEVEN: Self;
}

pub trait Eight {
    /// The eight value for this type.
    const EIGHT: Self;
}

pub trait Nine {
    /// The nine value for this type.
    const NINE: Self;
}

pub trait Ten {
    /// The ten value for this type.
    const TEN: Self;
}

pub trait OneThousand {
    /// The one thousand value for this type.
    const ONE_THOUSAND: Self;
}

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
    )*)
}

impl_constants! { u8 u16 u32 u64 u128 usize }
impl_constants! { i8 i16 i32 i64 i128 isize }

macro_rules! impl_constants_float {
    ($($t:ty)*) => ($(
        impl Half for $t {
            const HALF: Self = 0.5;
        }
        impl One for $t {
            const ONE: Self = 1.0;
            #[inline(always)]
            fn is_one(&self) -> bool { *self == 1.0 }
        }
        impl Zero for $t {
            const ZERO: Self = 0.0;
            #[inline(always)]
            fn is_zero(&self) -> bool { *self == 0.0 }
        }
        impl Two for $t {
            const TWO: Self = 2.0;
        }
        impl Three for $t {
            const THREE: Self = 3.0;
        }
        impl Five for $t {
            const FIVE: Self = 5.0;
        }
        impl Six for $t {
            const SIX: Self = 6.0;
        }
        impl Seven for $t {
            const SEVEN: Self = 7.0;
        }
        impl Eight for $t {
            const EIGHT: Self = 8.0;
        }
        impl Nine for $t {
            const NINE: Self = 9.0;
        }
        impl Ten for $t {
            const TEN: Self = 10.0;
        }
        impl OneThousand for $t {
            const ONE_THOUSAND: Self = 1_000.0;
        }
    )*)
}

impl_constants_float! { f32 f64 }

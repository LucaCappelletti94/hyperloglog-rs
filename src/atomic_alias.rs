use core::fmt::Debug;
/// Module providing Atomic Alias, a trait that allows to define a mapping between a type and its atomic version.
use core::sync::atomic::{AtomicU16, AtomicU32, AtomicU64, AtomicU8, AtomicUsize};

pub trait AtomicAlias {
    type Alias: Debug;

    fn into_atomic(self) -> Self::Alias;
    fn from_atomic(atomic: Self::Alias) -> Self;
}

impl AtomicAlias for u8 {
    type Alias = AtomicU8;

    fn into_atomic(self) -> Self::Alias {
        AtomicU8::from(self)
    }

    fn from_atomic(atomic: Self::Alias) -> Self {
        atomic.into_inner()
    }
}

impl AtomicAlias for u16 {
    type Alias = AtomicU16;

    fn into_atomic(self) -> Self::Alias {
        AtomicU16::from(self)
    }

    fn from_atomic(atomic: Self::Alias) -> Self {
        atomic.into_inner()
    }
}

impl AtomicAlias for u32 {
    type Alias = AtomicU32;

    fn into_atomic(self) -> Self::Alias {
        AtomicU32::from(self)
    }

    fn from_atomic(atomic: Self::Alias) -> Self {
        atomic.into_inner()
    }
}

impl AtomicAlias for u64 {
    type Alias = AtomicU64;

    fn into_atomic(self) -> Self::Alias {
        AtomicU64::from(self)
    }

    fn from_atomic(atomic: Self::Alias) -> Self {
        atomic.into_inner()
    }
}

impl AtomicAlias for usize {
    type Alias = AtomicUsize;

    fn into_atomic(self) -> Self::Alias {
        AtomicUsize::from(self)
    }

    fn from_atomic(atomic: Self::Alias) -> Self {
        atomic.into_inner()
    }
}

/// We implement the same trait as above, but now
/// we would like to make use of the transmute
/// operation, but we cannot do it because the
/// size of the array is not known at compile time.
/// For this reason, we fall back to using pointers.
impl<const N: usize> AtomicAlias for [u8; N] {
    type Alias = [AtomicU8; N];

    fn into_atomic(self) -> Self::Alias {
        let ptr = &self as *const Self as *const Self::Alias;
        unsafe { ptr.read() }
    }

    fn from_atomic(atomic: Self::Alias) -> Self {
        unsafe { *(atomic.as_ptr() as *const [u8; N]) }
    }
}

impl<const N: usize> AtomicAlias for [u16; N] {
    type Alias = [AtomicU16; N];

    fn into_atomic(self) -> Self::Alias {
        let ptr = &self as *const Self as *const Self::Alias;
        unsafe { ptr.read() }
    }

    fn from_atomic(atomic: Self::Alias) -> Self {
        unsafe { *(atomic.as_ptr() as *const [u16; N]) }
    }
}

impl<const N: usize> AtomicAlias for [u32; N] {
    type Alias = [AtomicU32; N];

    fn into_atomic(self) -> Self::Alias {
        let ptr = &self as *const Self as *const Self::Alias;
        unsafe { ptr.read() }
    }

    fn from_atomic(atomic: Self::Alias) -> Self {
        unsafe { *(atomic.as_ptr() as *const [u32; N]) }
    }
}

impl<const N: usize> AtomicAlias for [u64; N] {
    type Alias = [AtomicU64; N];

    fn into_atomic(self) -> Self::Alias {
        let ptr = &self as *const Self as *const Self::Alias;
        unsafe { ptr.read() }
    }

    fn from_atomic(atomic: Self::Alias) -> Self {
        unsafe { *(atomic.as_ptr() as *const [u64; N]) }
    }
}

impl<const N: usize> AtomicAlias for [usize; N] {
    type Alias = [AtomicUsize; N];

    fn into_atomic(self) -> Self::Alias {
        let ptr = &self as *const Self as *const Self::Alias;
        unsafe { ptr.read() }
    }

    fn from_atomic(atomic: Self::Alias) -> Self {
        unsafe { *(atomic.as_ptr() as *const [usize; N]) }
    }
}

//! Layout-frozen `[u64]` view over the counter.
//!
//! Enables `card-est-array`-style external-slab storage (e.g. `HyperBall`'s `SpillStore`, which
//! `memcpy`s the whole backend slab to a memory-mapped temp file after every iteration and
//! scatters modified slabs back into the in-memory array on the next). The downstream slab lives
//! as a borrowed `&mut [u64]`. This module lets a caller view the counter as its backing `u64`
//! slab and, symmetrically, reinterpret a `&[u64]` of the right length as a `&HyperLogLog<...>`.
//!
//! The safety of the view rests on two invariants:
//!
//! * The struct itself is `#[repr(C)]` (see the `repr` attribute on the struct declaration),
//!   so its fields are laid out in declaration order with no field reordering.
//! * The register field `R` implements the [`FixedLayoutRegisters`] marker trait, which
//!   promises the field is a fixed inline `[u64; N]`-shaped array with no heap indirection and
//!   no trailing padding.
//!
//! Together they mean the counter's memory footprint is exactly `word_len() * 8` bytes, all of
//! which are meaningful `u64` words (`harmonic_sum` bits in word 0, register bits in words
//! `1..word_len()`), with the same alignment as `u64`.
//!
//! No wire-format stability: the layout is meaningful only within one build with the same
//! `(P, B, R, H, C)` generic instantiation and the same `hyperloglog-rs` version. Serialize both
//! ends of any persistent boundary through the crate's `serde` derives if that matters.
#![allow(unsafe_code)]

use crate::hyperloglog::HyperLogLog;
use crate::prelude::{Bits, HasherType, Precision, Registers};
use sketching_core::{Packed, Words};

/// Marker trait certifying that the register type `R` has the layout of a fixed inline
/// `[u64; N]` array: its size and alignment are compile-time constants, its memory is directly
/// a `[u64; N]` (no heap indirection, no trailing padding), and any bit pattern is a valid
/// value (since `Packed<Words<N>, B>` imposes no bit-level invariant beyond storing bits).
///
/// Implemented only for the array-backed `Packed<Words<N>, B>` variant, NOT for heap-`Vec`
/// backings. External-slab counters cannot spill.
///
/// # Safety
///
/// Implementers MUST guarantee:
///
/// * `size_of::<Self>() == 8 * N` for the same `N` that indexes the register storage.
/// * `align_of::<Self>() == 8`.
/// * Every possible bit pattern of the underlying `[u64; N]` is a valid instance of `Self`.
///
/// The [`HyperLogLog`] slab view relies on these promises to skip any bit-level revalidation
/// when it borrows or reconstructs a counter over a `&[u64]`.
pub unsafe trait FixedLayoutRegisters<P: Precision, B: Bits>:
    Registers<P, B> + Sized
{
}

// The default array-backed register variant. `Packed<Words<N>, B>` under `#[repr(Rust)]` has the
// same layout as its single non-ZST field `words: Words<N>`, which itself has the same layout as
// its single field `[u64; N]`. That is the layout the marker trait promises. Vec-backed variants
// (`Packed<Vec<u64>, B>`) do NOT implement this trait: their memory footprint is a heap
// indirection, not an inline slab.
//
// SAFETY: the layout equivalence above is what the trait requires; see the doc comment on
// `FixedLayoutRegisters` for the exact promises.
unsafe impl<P: Precision, B: Bits, const N: usize> FixedLayoutRegisters<P, B>
    for Packed<Words<N>, B>
where
    Packed<Words<N>, B>: Registers<P, B>,
{
}

impl<P, B, R, H, C> HyperLogLog<P, B, R, H, C>
where
    P: Precision,
    B: Bits,
    R: FixedLayoutRegisters<P, B>,
    H: HasherType,
{
    /// Number of `u64` words the counter occupies under its `#[repr(C)]` layout.
    ///
    /// Equal to `1 + register_words()` for the default register backing: word 0 is the packed
    /// harmonic-sum and low-cardinality metadata word, words `1..` are the register array.
    #[inline]
    #[must_use]
    pub const fn word_len() -> usize {
        // Compile-time invariants that back the safety of `as_words` and `from_words`. If the
        // struct ever grows tail padding or an alignment jump (say, someone adds a non-word field
        // above the register array), these asserts fire at const evaluation time on every
        // instantiation.
        assert!(
            core::mem::size_of::<Self>().is_multiple_of(core::mem::size_of::<u64>()),
            "the counter's memory footprint must be a whole number of u64 words",
        );
        assert!(
            core::mem::align_of::<Self>() == core::mem::align_of::<u64>(),
            "the counter's alignment must equal u64's alignment",
        );
        core::mem::size_of::<Self>() / core::mem::size_of::<u64>()
    }

    /// Views the counter as its backing `u64` slab. The returned slice has length
    /// [`Self::word_len`] and lives for the borrow of `self`. The bytes are meaningful only
    /// under the exact same `(P, B, R, H, C)` instantiation and the same crate version.
    #[inline]
    #[must_use]
    pub fn as_words(&self) -> &[u64] {
        // SAFETY: `R: FixedLayoutRegisters<P, B>` plus the crate's `#[repr(C)]` on the struct
        // (see `slice_view` `cfg_attr` on the declaration) guarantee the counter is a contiguous
        // `[u64; word_len()]` in memory. `self` is a live reference, so the pointer is well
        // aligned by construction.
        unsafe {
            core::slice::from_raw_parts(
                core::ptr::from_ref::<Self>(self).cast::<u64>(),
                Self::word_len(),
            )
        }
    }

    /// Views the counter as its backing `u64` slab exclusively. As with [`Self::as_words`] but
    /// mutable.
    #[inline]
    #[must_use]
    pub fn as_words_mut(&mut self) -> &mut [u64] {
        // SAFETY: same as `as_words`; the exclusive borrow flows through to the returned slice.
        unsafe {
            core::slice::from_raw_parts_mut(
                core::ptr::from_mut::<Self>(self).cast::<u64>(),
                Self::word_len(),
            )
        }
    }

    /// Reinterprets a slice of length exactly [`Self::word_len`] as a `&Self`. Returns `None`
    /// on length mismatch or if the slice's start is not `u64`-aligned (which is a Rust
    /// requirement for `&[u64]` that only unsafe construction can violate).
    #[inline]
    #[must_use]
    pub fn from_words(words: &[u64]) -> Option<&Self> {
        if words.len() != Self::word_len() {
            return None;
        }
        let ptr = words.as_ptr();
        if !(ptr as usize).is_multiple_of(core::mem::align_of::<Self>()) {
            return None;
        }
        // SAFETY: length equals `word_len()`, so the borrow covers exactly the counter's memory
        // footprint. Alignment is checked one line up. `Self` is `#[repr(C)]` and `R:
        // FixedLayoutRegisters<P, B>` promises every bit pattern is a valid inhabitant, so no
        // bit-level revalidation is needed. The returned reference's lifetime is that of `words`.
        Some(unsafe { &*ptr.cast::<Self>() })
    }

    /// Reinterprets a mutable slice of length exactly [`Self::word_len`] as a `&mut Self`. The
    /// same length and alignment rules as [`Self::from_words`] apply.
    #[inline]
    #[must_use]
    pub fn from_words_mut(words: &mut [u64]) -> Option<&mut Self> {
        if words.len() != Self::word_len() {
            return None;
        }
        let ptr = words.as_mut_ptr();
        if !(ptr as usize).is_multiple_of(core::mem::align_of::<Self>()) {
            return None;
        }
        // SAFETY: same as `from_words`, plus the exclusive borrow of `words` flows through to
        // the returned reference.
        Some(unsafe { &mut *ptr.cast::<Self>() })
    }
}

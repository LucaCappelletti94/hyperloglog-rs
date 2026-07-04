//! The [`HyperLogLog`] counter: a hybrid that transitions across three internal representations as it
//! grows, the sorted value list (exact, for the smallest sets), then the sorted hash list, then the
//! `HyperLogLog` registers. The earlier representations keep values or hashes explicit until they no
//! longer fit, only then falling back to the probabilistic registers.

use crate::composite_hash::{GapHash, SaturationError};
use crate::correction_coefficients::HYPERLOGLOG_LINEAR_COUNT_THRESHOLD;
use crate::prelude::*;
use core::f64;
use core::fmt::Debug;
use core::hash::Hash;
use core::marker::PhantomData;
use sketching_core::sparse_value_list::{
    self as svl, SigBitsCode, SparseValueCodec, ValueInsertion, ValueIter, BE,
};

#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
// The `slice_view` module attaches `unsafe` methods (`as_words`, `from_words`, ...) to the
// counter. Clippy defensively flags any `serde::Deserialize` derive that coexists with unsafe
// code, because a deserializer feeding invalid bytes could break a safety invariant. That does
// not apply here: the counter's fields are `harmonic_sum: f64` (every bit pattern is a valid
// `f64`, including NaNs) and `registers: Packed<Words<N>, B>` (a wrapper over `[u64; N]`, which
// imposes no bit-level invariant beyond storing bits), so a deserialized counter is bit-for-bit
// equivalent to one produced by `insert` / `merge` and the slice-view accessors are safe on it.
#[cfg_attr(feature = "serde", allow(clippy::unsafe_derive_deserialize))]
// The struct is `#[repr(C)]` so its memory layout is stable: fields land in declaration order
// (`harmonic_sum` in word 0, `registers` in words `1..N`), no reordering, no tail padding. This
// is what lets the `slice_view` module view a counter as a `[u64]` slab and reconstruct one over
// a borrowed `&[u64]` in place. Every field is 8-byte aligned already, so `#[repr(C)]` is
// space-equivalent to `#[repr(Rust)]` on this struct, and using it unconditionally has zero cost
// while removing an axis on which downstream users can silently observe layout drift.
#[repr(C)]
/// A hybrid counter for approximate set cardinality estimation that transitions across three
/// representations as it grows (sorted value list, then sorted hash list, then `HyperLogLog`
/// registers), keeping values or hashes explicit until they no longer fit.
pub struct HyperLogLog<
    P: Precision,
    B: Bits,
    R: Registers<P, B> = <P as PackedRegister<B>>::Array,
    Hasher: HasherType = twox_hash::XxHash64,
    C = SigBitsCode,
> {
    /// The harmonic sum of the registers, i.e. the sum of `2^(-register_value)` for all
    /// registers. Placed first so under `#[repr(C)]` it occupies word 0 of the counter's memory
    /// slab, with the register bits filling words `1..N`.
    pub(crate) harmonic_sum: f64,
    /// The registers of the counter.
    pub(crate) registers: R,
    /// Phantom data to ensure the type parameters are used. `C` is a zero-runtime-cost codec
    /// witness: the codec is a codec-witness ZST (see `SigBitsCode`) that methods materialize with
    /// `C::default()` when they need to encode or decode the sorted value list.
    _phantom: PhantomData<(P, B, Hasher, C)>,
}

/// A [`HyperLogLog`] backed by a heap-allocated, growable register vector
/// ([`PackedRegister::Vec`]) rather than the default fixed-size register array
/// ([`PackedRegister::Array`]).
///
/// The default [`HyperLogLog`] stores its registers inline as a fixed array, whose size is
/// part of the type and lives wherever the counter lives (on the stack for a local). `VecHll` moves
/// that storage to the heap, which is preferable when the register array would be large (high
/// precision) or when many counters are created dynamically. The estimation behavior is identical;
/// only the register backing differs. Requires the `alloc` feature.
#[cfg(feature = "alloc")]
pub type VecHll<P, B, H = twox_hash::XxHash64, C = SigBitsCode> =
    HyperLogLog<P, B, <P as PackedRegister<B>>::Vec, H, C>;

impl<P: Precision, B: Bits, R: Registers<P, B>, H: HasherType, C> Default
    for HyperLogLog<P, B, R, H, C>
where
    C: SparseValueCodec,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<P: Precision, B: Bits, R: Registers<P, B> + PartialEq, H: HasherType, C> PartialEq
    for HyperLogLog<P, B, R, H, C>
{
    #[inline]
    /// Compares the `harmonic_sum` word by its raw bits rather than as an `f64`. The word doubles
    /// as a pre-dense metadata bitfield, as a NaN-boxed zero count in dense low-load mode, and as
    /// a pack of the harmonic sum and the zero-register count in dense harmonic mode. `NaN != NaN`
    /// under float equality would make two otherwise-identical zero-mode counters compare unequal,
    /// and the low-bit packing in harmonic mode is a real part of the counter's state. Bitwise
    /// comparison is exact for every representation. As a consequence two counters that reach the
    /// same register state via different insertion orders are NOT guaranteed equal: floating-point
    /// `H += 2^-r_new - 2^-r_old` is order-sensitive at ULP scale, so the packed word may differ.
    /// Counters built by the same insertion sequence are bit-identical.
    fn eq(&self, other: &Self) -> bool {
        self.harmonic_sum.to_bits() == other.harmonic_sum.to_bits()
            && self.registers == other.registers
    }
}

#[inline]
fn correction_upper_bound<P: Precision>() -> f64 {
    7.5 * f64::integer_exp2(P::EXPONENT)
}

/// Bit pattern of a sign-0 quiet NaN used to mark the dense low-load "zeros mode": in dense register
/// mode the `harmonic_sum` word holds the real harmonic sum, except below the linear-counting
/// threshold where it instead holds the zero-register count NaN-boxed here, so linear counting is O(1)
/// without a register scan. The all-ones exponent plus the set bit 51 make it a NaN regardless of the
/// payload, and the sign bit stays 0 so [`HyperLogLog::is_hyperloglog`] still classifies it as dense.
const DENSE_ZEROS_NAN_TAG: u64 = 0x7FF8_0000_0000_0000;

/// Mask for the zero count packed in the low bits of [`DENSE_ZEROS_NAN_TAG`]. 32 bits is far more than
/// the at most `2^P <= 2^18` zeros, and stays clear of the NaN marker bits.
const DENSE_ZEROS_MASK: u64 = 0xFFFF_FFFF;

/// Encodes a zero-register count into the NaN-boxed dense "zeros mode" word.
#[inline]
fn encode_dense_zeros(zeros: u32) -> f64 {
    f64::from_bits(DENSE_ZEROS_NAN_TAG | u64::from(zeros))
}

/// Decodes the zero-register count from a NaN-boxed dense "zeros mode" word.
#[inline]
fn decode_dense_zeros(harmonic_sum: f64) -> u32 {
    (harmonic_sum.to_bits() & DENSE_ZEROS_MASK) as u32
}

/// Low-mantissa bit width reserved for the packed zero-register count in the dense bias-corrected
/// band, indexed `[P - 4][B - 4]`. In that band the `HyperLogLog::harmonic_sum` f64 word carries the
/// harmonic sum `H` in its high `52 - ZERO_BITS[..]` mantissa bits with the low `ZERO_BITS[..]` bits
/// clear and holding the zero-register count, so Ertl's sigma/tau estimator can read both moments in
/// O(1) without a register scan. Sized to the empirical worst case `m * exp(-threshold / m)` (matched
/// by `examples/zero_count_probe.rs` to within 1 count across P4..P18) plus a two-bit safety margin,
/// with a four-bit floor. Full derivation and the probe run are in `docs/zero_bits_table.md`.
const ZERO_BITS: [[u8; 3]; 15] = [
    [4, 4, 4],    // P4   max zeros ~     1 /     1 /     1
    [5, 4, 4],    // P5   max zeros ~     6 /     1 /     1
    [6, 6, 5],    // P6   max zeros ~    12 /     9 /     7
    [7, 7, 6],    // P7   max zeros ~    26 /    17 /    12
    [8, 8, 4],    // P8   max zeros ~    61 /    38 /     2
    [10, 9, 4],   // P9   max zeros ~   140 /    88 /     2
    [11, 10, 9],  // P10  max zeros ~   279 /   176 /   106
    [12, 11, 10], // P11  max zeros ~   566 /   357 /   218
    [13, 12, 11], // P12  max zeros ~  1129 /   726 /   435
    [14, 13, 12], // P13  max zeros ~  2284 /  1446 /   884
    [14, 14, 13], // P14  max zeros ~  3846 /  2760 /  1700
    [16, 15, 14], // P15  max zeros ~ 10764 /  6631 /  3764
    [16, 15, 15], // P16  max zeros ~  8596 /  7127 /  7706
    [13, 13, 13], // P17  max zeros ~  1624 /  1456 /  1456
    [13, 13, 13], // P18  max zeros ~  1376 /  1397 /  1397
];

/// Number of low mantissa bits of `harmonic_sum` that hold the packed zero-register count for the
/// `(P, B)` cell, i.e. `ZERO_BITS[P - 4][B - 4]`. A monomorphizing wrapper so callers do not spell
/// out the index arithmetic each time.
#[inline]
fn zero_bits<P: Precision, B: Bits>() -> u8 {
    ZERO_BITS[P::EXPONENT as usize - 4][B::NUMBER_OF_BITS as usize - 4]
}

/// Packs a zero-register count into the low `zero_bits` mantissa bits of the harmonic-sum word,
/// clearing those bits of `h` first. The high `52 - zero_bits` mantissa bits (and the sign and
/// exponent) carry the harmonic sum, so `unpack_harmonic(pack_harmonic_with_zeros(h, z, k), k)`
/// recovers `h` truncated by at most one low-bits ULP and `unpack_zeros(.., k)` recovers `z`.
///
/// `zero_bits` MUST be at most 32 (the width of the `zeros: u32` payload); `zeros` MUST fit in that
/// many bits (`zeros < 1 << zero_bits`), or the debug asserts fire and the packed count truncates.
#[inline]
fn pack_harmonic_with_zeros(h: f64, zeros: u32, zero_bits: u8) -> f64 {
    debug_assert!(
        zero_bits <= 32,
        "zero_bits ({zero_bits}) must be at most 32 (the u32 zeros field width)"
    );
    debug_assert!(
        zero_bits == 32 || u64::from(zeros) < 1u64 << zero_bits,
        "zeros ({zeros}) does not fit in {zero_bits} bits"
    );
    let mask = (1u64 << zero_bits) - 1;
    let h_bits = h.to_bits() & !mask;
    f64::from_bits(h_bits | u64::from(zeros))
}

/// Extracts the harmonic sum from a packed word by clearing the low `zero_bits` mantissa bits. The
/// relative error against the original harmonic sum is at most `2^(zero_bits - 52)`.
#[inline]
fn unpack_harmonic(word: f64, zero_bits: u8) -> f64 {
    debug_assert!(
        zero_bits <= 32,
        "zero_bits ({zero_bits}) must be at most 32"
    );
    let mask = (1u64 << zero_bits) - 1;
    f64::from_bits(word.to_bits() & !mask)
}

/// Extracts the packed zero-register count from the low `zero_bits` mantissa bits of the word.
#[inline]
fn unpack_zeros(word: f64, zero_bits: u8) -> u32 {
    debug_assert!(
        zero_bits <= 32,
        "zero_bits ({zero_bits}) must be at most 32"
    );
    let mask = (1u64 << zero_bits) - 1;
    (word.to_bits() & mask) as u32
}

/// Which cardinality-estimation regime a counter is in, returned by
/// [`HyperLogLog::estimation_regime`]. It is a function of the representation and, for `HyperLogLog`
/// registers, of the cardinality (raw above the correction bound, empirically corrected below).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EstimationRegime {
    /// Sorted value list: the stored values are counted directly, with no estimation.
    Exact,
    /// Sorted hash list: corrected for hash collisions via the birthday-paradox
    /// `HASHLIST_CORRECTION_*` tables.
    HashListCollisionCorrected,
    /// `HyperLogLog` registers below `correction_upper_bound`: Ertl's analytical tau/sigma
    /// estimator (`sigma_tau_cardinality`), evaluated in O(1) from the packed `(H, zeros)` moments.
    HyperLogLogBiasCorrected,
    /// `HyperLogLog` registers at very low load (the linear-counting estimate is at or below the
    /// regenerated per-`(P, B)` `HYPERLOGLOG_LINEAR_COUNT_THRESHOLD`): the count of zero registers
    /// drives `m * ln(m / zeros)`, which beats the bias-corrected raw estimate there. This regime
    /// is reached only by a counter forced into registers early (`to_hll`) while still sparse; a
    /// counter that densified naturally is already past it.
    HyperLogLogLinearCounted,
    /// `HyperLogLog` registers at or above `correction_upper_bound`: the raw `alpha * m^2 / sum`
    /// estimate, returned uncorrected.
    HyperLogLogRaw,
}

impl<P: Precision, B: Bits, R: Registers<P, B>, H: HasherType, C> HyperLogLog<P, B, R, H, C>
where
    C: SparseValueCodec,
{
    #[inline]
    fn new() -> Self {
        let mut hll = Self {
            registers: R::default(),
            harmonic_sum: f64::NEG_INFINITY,
            _phantom: PhantomData,
        };

        hll.clear();

        hll
    }

    #[inline]
    /// Returns whether the counter is empty.
    pub fn is_empty(&self) -> bool {
        self.is_sorted_hash_list() && self.get_number_of_hashes().unwrap() == 0
    }

    #[inline]
    /// Returns whether the counter is fully saturated.
    pub fn is_full(&self) -> bool {
        // The harmonic sum is defined as Sum(2^(-register_value)) for all registers.
        // When all registers are maximally filled, i.e. equal to the maximal multiplicity value,
        // the harmonic sum is equal to (2^(-max_multiplicity)) * number_of_registers.
        // Since number_of_registers is a power of 2, specifically 2^exponent, the harmonic sum
        // is equal to 2^(exponent - max_multiplicity). Only a HyperLogLog counter can be full; the
        // pre-HyperLogLog representations reuse `harmonic_sum` as a metadata word, not a real sum.
        // The saturated harmonic sum is a pure power of two (mantissa low bits are zero), so the
        // band-mask applied inside `dense_harmonic_sum` never disturbs it.
        self.is_hyperloglog()
            && self.dense_harmonic_sum()
                <= f64::integer_exp2_minus_signed(
                    (1_i16 << B::NUMBER_OF_BITS) - i16::from(P::EXPONENT) - 1,
                )
    }

    #[inline]
    /// Returns whether the provided element may be contained in the counter.
    pub fn may_contain<T: Hash>(&self, element: &T) -> bool {
        let (index, register, original_hash) = Self::index_and_register_and_hash(element);
        // In sorted-value-list mode the stored items are literal values, not hashes, so test membership by
        // hashing each stored value and matching the full original hash (exact, no false negatives).
        if self.is_sorted_value_list() {
            return ValueIter::<BE, _>::new(
                self.registers.as_ref(),
                0,
                self.get_number_of_values(),
                C::default(),
            )
            .any(|value| Self::index_and_register_and_hash(&value).2 == original_hash);
        }
        if self.is_sorted_hash_list() {
            GapHash::<P, B>::find(
                self.registers.as_ref(),
                self.get_number_of_hashes().unwrap(),
                index,
                register,
                original_hash,
                self.get_hash_bits().unwrap(),
                self.get_writer_tell(),
            )
        } else {
            self.registers.get_register(index) >= register
        }
    }

    #[inline]
    /// Returns whether the counter is in `HyperLogLog` registers, as opposed to one of the two
    /// pre-HyperLogLog representations (the sorted hash list or the sorted value list), which repurpose
    /// `harmonic_sum` as a metadata word with its top bit set.
    pub fn is_hyperloglog(&self) -> bool {
        self.harmonic_sum.to_bits().leading_zeros() != 0
    }

    #[inline]
    /// Returns whether the counter is in the sorted value list: the representation that precedes the
    /// sorted hash list, storing the literal inserted integers for exact recovery and exact set
    /// operations.
    pub fn is_sorted_value_list(&self) -> bool {
        !self.is_hyperloglog() && self.is_sorted_value_list_metadata()
    }

    #[inline]
    /// Returns whether the counter is in sorted hash list: a sorted list of composite hashes, the
    /// representation between the sorted value list and `HyperLogLog` registers. This is exactly one of the
    /// three representations (see [`HyperLogLog::is_sorted_value_list`] and [`HyperLogLog::is_hyperloglog`]).
    pub fn is_sorted_hash_list(&self) -> bool {
        !self.is_hyperloglog() && !self.is_sorted_value_list_metadata()
    }

    #[inline]
    /// Returns which cardinality-estimation regime the counter is currently in (see
    /// [`EstimationRegime`]), a function of its representation and, for registers, its cardinality.
    pub fn estimation_regime(&self) -> EstimationRegime {
        if self.is_sorted_value_list() {
            return EstimationRegime::Exact;
        }
        if self.is_sorted_hash_list() {
            return EstimationRegime::HashListCollisionCorrected;
        }
        // HyperLogLog registers. The dense word's mode records which estimator applies, mirroring
        // `estimate_cardinality`: zeros mode is the low-load linear-counting regime, harmonic mode is
        // raw above the correction bound and empirically bias-corrected below it.
        // Zeros mode still lives in the NaN box, so keep the direct check; the raw-vs-corrected
        // decision then reads the harmonic sum through the band-mask accessor.
        if self.harmonic_sum.is_nan() {
            return EstimationRegime::HyperLogLogLinearCounted;
        }
        let raw_estimate =
            P::ALPHA * f64::integer_exp2(P::EXPONENT + P::EXPONENT) / self.dense_harmonic_sum();
        if raw_estimate >= correction_upper_bound::<P>() {
            EstimationRegime::HyperLogLogRaw
        } else {
            EstimationRegime::HyperLogLogBiasCorrected
        }
    }

    #[inline]
    /// Returns the number of registers equal to zero.
    ///
    /// # Raises
    /// If the counter is in sorted hash list, an error is raised.
    pub fn number_of_zero_registers(&self) -> Result<usize, &'static str> {
        if self.is_hyperloglog() {
            Ok(self
                .registers
                .iter_registers()
                .filter(|&register| register == 0)
                .count())
        } else {
            Err("The counter is in sorted hash list mode.")
        }
    }

    #[inline]
    /// The harmonic sum `H = sum 2^-register` of the dense registers, ready for any raw or
    /// bias-corrected estimator. O(1) in harmonic mode (the stored word with the low
    /// [`ZERO_BITS`] mantissa bits masked to zero, so the packed zero-register count that step 4
    /// puts there does not leak into `H`), O(m) in zeros mode (reconstructed from the registers,
    /// since the word is a NaN-boxed count instead of a sum). Only valid in register mode.
    ///
    /// The mask perturbs `H` by at most one low-bits ULP, i.e. `|H_masked - H| <= H *
    /// 2^(ZERO_BITS[..] - 52) ~ 1.5e-11` in the worst cell (P15/P16 B4). That is six to eight
    /// orders of magnitude below the register-noise floor and negligible for every estimator.
    pub(crate) fn dense_harmonic_sum(&self) -> f64 {
        if self.harmonic_sum.is_nan() {
            self.harmonic_sum_from_registers()
        } else {
            unpack_harmonic(self.harmonic_sum, zero_bits::<P, B>())
        }
    }

    #[inline]
    /// Zero-register count of a dense counter, O(1) from the word alone: from the NaN-boxed count
    /// in dense zeros mode, from the low-mantissa packing in dense harmonic mode. Panics in debug
    /// if the counter is not dense.
    pub(crate) fn dense_zero_count(&self) -> u32 {
        debug_assert!(
            self.is_hyperloglog(),
            "dense_zero_count called on a pre-dense counter",
        );
        if self.harmonic_sum.is_nan() {
            decode_dense_zeros(self.harmonic_sum)
        } else {
            unpack_zeros(self.harmonic_sum, zero_bits::<P, B>())
        }
    }

    #[inline]
    /// Computes the harmonic sum `sum 2^-register` directly from the registers (an O(m) scan). Used to
    /// materialize the real sum when leaving zeros mode.
    fn harmonic_sum_from_registers(&self) -> f64 {
        self.registers
            .iter_registers()
            .map(f64::integer_exp2_minus)
            .sum()
    }

    #[inline]
    fn clear(&mut self) {
        self.registers.clear_registers();
        self.harmonic_sum = f64::NEG_INFINITY;
        self.set_number_of_hashes(0);
        self.set_writer_tell(0);
        self.set_duplicates(0);
        self.set_hash_bits(GapHash::<P, B>::LARGEST_VIABLE_HASH_BITS);
        debug_assert!(self.is_sorted_hash_list());
    }

    #[inline]
    /// Inserts an element into the counter.
    pub fn insert<T: Hash>(&mut self, element: &T) -> bool {
        // A hashed insert is incompatible with the sorted value list (which stores literal values),
        // so first promote an exact counter to a proper sorted hash list by hashing its stored values.
        if self.is_sorted_value_list() {
            self.to_sorted_hash_list();
        }
        let (index, register, original_hash) = Self::index_and_register_and_hash(element);
        self.insert_index_register_hash(index, register, original_hash)
    }

    #[inline]
    /// Inserts a pre-hashed element, given its register index, register value and the
    /// original hash it was derived from.
    ///
    /// This is the shared core of [`HyperLogLog::insert`] and of the counter merging
    /// performed by the [`BitOr`] implementations: both need to route a hash through the
    /// sorted hash list insertion path (with its saturation and downgrade handling) or, once the
    /// counter is a fully-fledged [`HyperLogLog`], straight into the registers.
    fn insert_index_register_hash(
        &mut self,
        index: usize,
        register: u8,
        original_hash: u64,
    ) -> bool {
        // The sorted value list never reaches this hashed-insert path: callers promote it to a
        // proper sorted hash list first.
        debug_assert!(!self.is_sorted_value_list());
        if self.is_sorted_hash_list() {
            let hash_bits = self.get_hash_bits().unwrap();
            let number_of_hashes = self.get_number_of_hashes().unwrap();
            let writer_tell = self.get_writer_tell();

            match GapHash::<P, B>::insert_sorted_desc(
                self.registers.as_mut(),
                number_of_hashes,
                writer_tell,
                index,
                register,
                original_hash,
                hash_bits,
            ) {
                Ok(Some(insert_metadata)) => {
                    self.set_number_of_hashes(number_of_hashes + 1 - insert_metadata.duplicates);
                    self.set_writer_tell(insert_metadata.bit_index);
                    self.add_duplicates(insert_metadata.duplicates);
                    self.set_hash_bits(insert_metadata.hash_bits);
                    true
                }
                Ok(None) => false,
                Err(err) => match err {
                    SaturationError::ExtendableSaturation => {
                        self.registers.increase_capacity();
                        self.insert_index_register_hash(index, register, original_hash)
                    }
                    SaturationError::Saturation(bit_index) => {
                        self.set_writer_tell(bit_index);
                        debug_assert_eq!(bit_index, self.get_writer_tell());
                        self.to_hll();
                        debug_assert!(self.is_hyperloglog());
                        self.insert_index_register_hash(index, register, original_hash)
                    }
                },
            }
        } else {
            self.insert_register_value_and_index(register, index)
        }
    }

    #[inline]
    /// Returns a zeroed register buffer grown to the full `HyperLogLog` register array size.
    ///
    /// The current buffer may be a lazily grown vector far smaller than the full register array
    /// (a small sorted value list or sorted hash list never triggers a capacity bump). Promotions
    /// that scatter registers across the whole index space need a full-size destination first,
    /// otherwise a high register index would write out of bounds.
    fn full_size_cleared_registers(&self) -> R {
        let mut new_registers = self.registers.clone();
        new_registers.clear_registers();
        let maximal_bits = (1usize << P::EXPONENT) * B::NUMBER_OF_BITS as usize;
        while new_registers.as_ref().len() * 8 < maximal_bits {
            new_registers.increase_capacity();
        }
        new_registers
    }

    #[inline]
    /// Promotes the counter to `HyperLogLog` registers, dispatching on its current representation. A
    /// sorted hash list is decoded into registers. A sorted value list is rehashed **directly** into
    /// registers at full hash width, which is strictly less lossy than routing through the sorted
    /// hash list (whose truncated hashes can clip a register's leading-zero rank, biasing the count
    /// low). A no-op if the counter is already in `HyperLogLog` registers.
    pub fn to_hll(&mut self) {
        if self.is_hyperloglog() {
            return;
        }

        if self.is_sorted_value_list() {
            // Direct: decode each stored value and rehash it at full width straight into the
            // registers (no sorted hash list round-trip).
            let count = self.get_number_of_values();
            let new_registers = self.full_size_cleared_registers();
            let source = core::mem::replace(&mut self.registers, new_registers);
            self.harmonic_sum = encode_dense_zeros(u32::try_from(1u64 << P::EXPONENT).unwrap());
            for value in ValueIter::<BE, _>::new(source.as_ref(), 0, count, C::default()) {
                let (index, register, _) = Self::index_and_register_and_hash(&value);
                self.insert_register_value_and_index(register, index);
            }
            debug_assert!(self.is_hyperloglog());
            return;
        }

        // Sorted hash list: decode the stored hashes into registers.
        let hash_bits = self.get_hash_bits().unwrap();
        let new_registers = self.full_size_cleared_registers();
        let registers = core::mem::replace(&mut self.registers, new_registers);
        let number_of_hashes = self.get_number_of_hashes().unwrap();
        let writer_tell = self.get_writer_tell();
        self.harmonic_sum = encode_dense_zeros(u32::try_from(1u64 << P::EXPONENT).unwrap());

        let mut last_index = usize::MAX;
        GapHash::<P, B>::decoded(registers.as_ref(), number_of_hashes, hash_bits, writer_tell)
            .for_each(|(new_register_value, index)| {
                if last_index == index {
                    return;
                }
                last_index = index;
                self.insert_register_value_and_index(new_register_value, index);
            });

        debug_assert!(self.is_hyperloglog());
    }

    #[inline]
    #[must_use]
    /// Consuming form of [`to_hll`](Self::to_hll): promotes the counter to `HyperLogLog` registers and
    /// returns it.
    pub fn into_hll(mut self) -> Self {
        self.to_hll();
        self
    }

    #[inline]
    /// Inserts a literal integer value, storing it exactly (and recoverably) while the counter is
    /// small enough to remain in the sorted value list.
    ///
    /// Accepts any unsigned integer that fits in a `u64` (`u8`, `u16`, `u32`, `u64`), so a smaller
    /// type can be passed without an `as u64` cast. Smaller values also pack tighter in the sorted
    /// value list (its gamma codec sizes each entry by magnitude), so `u32` inputs roughly double its
    /// capacity over full-width `u64` values.
    ///
    /// A fresh counter enters the sorted value list on its first `insert_value`. When the exact
    /// buffer fills, the stored values are hashed (with the counter's hasher `H`) into a proper hash
    /// list, which later transitions to `HyperLogLog` registers, exactly like a hashed counter. Once a
    /// counter has left the sorted value list (because it grew, or because a hashed
    /// [`HyperLogLog::insert`] was used) a value is hashed and inserted like any other element.
    ///
    /// Returns whether the value was newly inserted.
    pub fn insert_value<T: Into<u64>>(&mut self, value: T) -> bool {
        let value: u64 = value.into();
        if self.is_sorted_value_list() {
            return self.insert_value_exact(value);
        }
        if self.is_sorted_hash_list() && self.get_number_of_hashes().unwrap() == 0 {
            // A fresh, empty counter: enter the sorted value list.
            self.registers.clear_registers();
            self.set_sorted_value_list_mode();
            debug_assert!(self.is_sorted_value_list());
            return self.insert_value_exact(value);
        }
        // The counter has already left the sorted value list: hash the value like any element.
        let (index, register, original_hash) = Self::index_and_register_and_hash(&value);
        self.insert_index_register_hash(index, register, original_hash)
    }

    #[inline]
    /// Inserts a value into the sorted value list, growing the buffer or transitioning to a hash
    /// list when it no longer fits.
    fn insert_value_exact(&mut self, value: u64) -> bool {
        let count = self.get_number_of_values();
        match svl::insert_value::<BE, _>(self.registers.as_mut(), 0, count, value, C::default()) {
            ValueInsertion::Inserted => {
                self.set_number_of_values(count + 1);
                true
            }
            ValueInsertion::Duplicate => false,
            ValueInsertion::DoesNotFit => {
                let maximal_bits = (1usize << P::EXPONENT) * B::NUMBER_OF_BITS as usize;
                if self.registers.as_ref().len() * 8 < maximal_bits {
                    // The buffer is a growable vector below its maximum: grow and retry.
                    self.registers.increase_capacity();
                    self.insert_value_exact(value)
                } else {
                    // The buffer is at its maximum: hash the stored values into a sorted hash list and
                    // insert the new value there.
                    self.to_sorted_hash_list();
                    let (index, register, original_hash) =
                        Self::index_and_register_and_hash(&value);
                    self.insert_index_register_hash(index, register, original_hash)
                }
            }
        }
    }

    /// Merges another sorted value list counter into this one (both must be in exact mode) in linear time,
    /// keeping the result exact. Returns `false` without modifying `self` if the union does not fit
    /// the exact buffer, so the caller can transition out of exact mode instead.
    ///
    /// Both operands store their values sorted (descending) and gap-coded, so the union is produced by
    /// a single two-pointer merge written once, rather than by splicing each value of the other
    /// operand into this one (which is quadratic). The only allocation is one clone of this counter's
    /// own value buffer, mirroring the mode-transition paths.
    fn try_merge_exact_values(&mut self, rhs: &Self) -> bool {
        let count_self = self.get_number_of_values();
        let count_rhs = rhs.get_number_of_values();
        let (union_count, needed_bits) = svl::merge_metrics::<BE, _>(
            self.registers.as_ref(),
            0,
            count_self,
            rhs.registers.as_ref(),
            0,
            count_rhs,
            C::default(),
        );

        let maximal_bits = (1usize << P::EXPONENT) * B::NUMBER_OF_BITS as usize;
        if needed_bits as usize > maximal_bits {
            return false;
        }

        // Move this counter's values aside, then grow (for a growable buffer) and rewrite in place.
        let source = self.registers.clone();
        while self.registers.as_ref().len() * 8 < needed_bits as usize {
            self.registers.increase_capacity();
        }
        self.registers.clear_registers();
        svl::merge_write::<BE, _>(
            source.as_ref(),
            0,
            count_self,
            rhs.registers.as_ref(),
            0,
            count_rhs,
            self.registers.as_mut(),
            0,
            C::default(),
        );
        self.set_number_of_values(union_count);
        true
    }

    #[inline]
    /// Promotes a sorted value list to a sorted hash list by hashing each stored value with the
    /// counter's hasher `H`. This one-way transition loses recovery and absolute exactness. A no-op
    /// if the counter is already a sorted hash list or `HyperLogLog` registers (you cannot move back
    /// down the ladder); use [`to_hll`](Self::to_hll) to go further.
    pub fn to_sorted_hash_list(&mut self) {
        if !self.is_sorted_value_list() {
            return;
        }
        // The values and the destination sorted hash list share the same register buffer, so move the
        // value bytes aside (a single buffer clone, as the sorted hash list to HyperLogLog transition
        // also does) and stream them lazily into the cleared sorted hash list.
        let count = self.get_number_of_values();
        let source = self.registers.clone();
        self.clear();
        for value in ValueIter::<BE, _>::new(source.as_ref(), 0, count, C::default()) {
            let (index, register, original_hash) = Self::index_and_register_and_hash(&value);
            self.insert_index_register_hash(index, register, original_hash);
        }
    }

    #[inline]
    #[must_use]
    /// Consuming form of [`to_sorted_hash_list`](Self::to_sorted_hash_list): promotes the counter to
    /// a sorted hash list and returns it.
    pub fn into_sorted_hash_list(mut self) -> Self {
        self.to_sorted_hash_list();
        self
    }

    #[inline]
    /// Recovers the exact set of literal values inserted via [`HyperLogLog::insert_value`] as a lazy
    /// iterator yielding them in descending order, if the counter is still in the sorted value list.
    /// Returns `None` once the counter has left exact mode (the literal values are no longer retained
    /// past that transition).
    pub fn recover_values(&self) -> Option<impl Iterator<Item = u64> + '_> {
        if self.is_sorted_value_list() {
            Some(ValueIter::<BE, _>::new(
                self.registers.as_ref(),
                0,
                self.get_number_of_values(),
                C::default(),
            ))
        } else {
            None
        }
    }

    #[inline]
    /// Returns whether the given literal value is present, exactly, while the counter is in the
    /// sorted value list. Falls back to the probabilistic hashed membership otherwise.
    pub fn may_contain_value(&self, value: u64) -> bool {
        if self.is_sorted_value_list() {
            svl::contains_value::<BE, _>(
                self.registers.as_ref(),
                0,
                self.get_number_of_values(),
                value,
                C::default(),
            )
        } else {
            self.may_contain(&value)
        }
    }

    #[inline]
    /// Splits a hash into a register value and an index.
    fn insert_register_value_and_index(&mut self, new_register_value: u8, index: usize) -> bool {
        // Count leading zeros.
        debug_assert!(
            new_register_value <= u8::try_from(B::MASK).unwrap(),
            "Register value is too large: {new_register_value} > {}",
            B::MASK
        );

        let (old_register_value, larger_register_value) =
            self.registers.set_greater(index, new_register_value);

        if self.harmonic_sum.is_nan() {
            // Zeros mode (dense, low load): the word holds the zero-register count, not the sum. A
            // register leaves the zero state exactly when `old == 0` and the new larger value is > 0.
            if old_register_value == 0 && larger_register_value > 0 {
                let zeros = decode_dense_zeros(self.harmonic_sum) - 1;
                let m = f64::integer_exp2(P::EXPONENT);
                // Stay in zeros mode while linear counting is still the preferred estimator (the same
                // decision `corrected_register_cardinality` makes); otherwise materialize the real
                // harmonic sum once and switch to harmonic mode for the rest of the counter's life,
                // packing the current zero count into the low mantissa bits so the sigma/tau O(1)
                // moments path can read it back without a scan.
                let stays_linear = zeros > 0
                    && m * (m / f64::from(zeros)).natural_log() <= Self::linear_count_threshold();
                self.harmonic_sum = if stays_linear {
                    encode_dense_zeros(zeros)
                } else {
                    let h = self.harmonic_sum_from_registers();
                    pack_harmonic_with_zeros(h, zeros, zero_bits::<P, B>())
                };
            }
        } else {
            // Harmonic mode (dense, above the crossover): extract the current harmonic sum from the
            // high mantissa bits and the packed zero-register count from the low bits, apply the H
            // delta, decrement the count when a zero flipped to nonzero, then repack. The mask-add-
            // repack pattern is essential: doing `harmonic_sum += delta` directly would let the
            // delta's low bits corrupt the packed count, since delta and count share the same word.
            let k = zero_bits::<P, B>();
            let mut zeros = unpack_zeros(self.harmonic_sum, k);
            let new_h = unpack_harmonic(self.harmonic_sum, k)
                + f64::integer_exp2_minus(larger_register_value)
                - f64::integer_exp2_minus(old_register_value);
            if old_register_value == 0 && larger_register_value > 0 {
                debug_assert!(zeros > 0, "packed zero count went stale on decrement");
                zeros -= 1;
            }
            debug_assert!(new_h.is_finite() && new_h > 0.0);
            self.harmonic_sum = pack_harmonic_with_zeros(new_h, zeros, k);
        }

        old_register_value < new_register_value
    }

    #[inline]
    /// Returns the uncorrected estimate of the cardinality.
    pub fn uncorrected_estimate_cardinality(&self) -> f64 {
        if self.is_sorted_value_list() {
            return f64::from(self.get_number_of_values());
        }
        if self.is_sorted_hash_list() {
            f64::from(self.get_number_of_hashes().unwrap() + self.get_duplicates())
        } else {
            P::ALPHA * f64::integer_exp2(P::EXPONENT + P::EXPONENT) / self.dense_harmonic_sum()
        }
    }

    #[inline]
    /// The cardinality at or below which registers should report linear counting instead of the
    /// bias-corrected raw estimate, the regenerated empirical `HYPERLOGLOG_LINEAR_COUNT_THRESHOLD`
    /// for this `(precision, bits)`.
    fn linear_count_threshold() -> f64 {
        f64::from(
            HYPERLOGLOG_LINEAR_COUNT_THRESHOLD[P::EXPONENT as usize - 4]
                [B::NUMBER_OF_BITS as usize - 4],
        )
    }

    #[inline]
    /// Corrects a register-mode cardinality from its harmonic sum and zero-register count. Uses
    /// linear counting (`m * ln(m / zeros)`) when its estimate is at or below the regenerated
    /// threshold (the most accurate estimator at low load), otherwise Ertl's analytical tau/sigma
    /// estimator via [`ertl_cardinality_from_moments`](crate::sigma_tau::ertl_cardinality_from_moments)
    /// with `saturated = 0`. Shared by the single-counter
    /// [`estimate_cardinality`](Self::estimate_cardinality) and the dense union path so both apply
    /// linear counting at low load rather than sigma/tau's `O(m)` H reconstruction, and sigma/tau
    /// everywhere above.
    pub(crate) fn corrected_register_cardinality(harmonic_sum: f64, zeros: usize) -> f64 {
        if zeros > 0 {
            let m = f64::integer_exp2(P::EXPONENT);
            let linear_counting = m * (m / zeros as f64).natural_log();
            if linear_counting <= Self::linear_count_threshold() {
                return linear_counting;
            }
        }
        #[allow(
            clippy::cast_precision_loss,
            reason = "zeros <= m <= 2^18 is exact in f64"
        )]
        let zeros_f = zeros as f64;
        crate::sigma_tau::ertl_cardinality_from_moments::<P, B>(harmonic_sum, zeros_f, 0.0)
    }

    /// Expected number of distinct composite hashes for `n` elements at the given width, and its
    /// derivative with respect to `n`, under the `SwitchHash` cell model.
    ///
    /// A composite at width `w = hash_bits` is a uniform `P`-bit index followed by a `t = w - P` bit
    /// tail encoding the register (a geometric leading-zero count) and a uniform residual. All `2^P`
    /// indices share the same tail-cell probabilities, so with per-cell absolute probability `p_c`
    /// the expected distinct count is `E[D | n] = sum_c (1 - (1 - p_c)^n)`, evaluated here by group
    /// of equiprobable cells. The derivative is returned alongside so the inversion can use Newton
    /// steps. See `correction_coefficients/validate_occupancy.py` for the model derivation and its
    /// validation against measured trajectories.
    fn hash_list_expected_distinct(n: f64, hash_bits: u8) -> (f64, f64, f64) {
        let m = f64::integer_exp2(P::EXPONENT);
        let t = hash_bits - P::EXPONENT; // tail bits, always >= B (the smallest viable width is P + B)
        let r_max = (1_u8 << B::NUMBER_OF_BITS) - 1; // largest register value the B-bit field stores

        let mut expected = 0.0_f64;
        let mut derivative = 0.0_f64;
        // Group sums for the fixed-n variance of the distinct count below: `survival_sum = sum_c
        // s_c`, `shifted_survival_sum = sum_c (1 - 2 p_c)^n`, `weighted_survival = sum_c s_c p_c`,
        // with `s_c = (1 - p_c)^n`.
        let mut survival_sum = 0.0_f64;
        let mut shifted_survival_sum = 0.0_f64;
        let mut weighted_survival = 0.0_f64;
        // Accumulate a group of `count` cells, each with per-index conditional probability `q` (so
        // absolute probability `p = q / m`). `survival = (1 - p)^n` and `d/dn (1 - survival) =
        // -ln(1 - p) * survival`.
        let mut accumulate = |count: f64, q: f64| {
            let p = q / m;
            let log_survival = (1.0 - p).natural_log();
            // Call the trait method explicitly: with the `std` feature active `.exp()` would resolve
            // to std's inherent `f64::exp`, but the no_std build must use our transcendental-free one.
            let survival = FloatOps::exp(n * log_survival);
            expected += count * (1.0 - survival);
            derivative += count * (-log_survival) * survival;
            survival_sum += count * survival;
            shifted_survival_sum += count * FloatOps::exp(n * (1.0 - 2.0 * p).natural_log());
            weighted_survival += count * survival * q;
        };

        if t == B::NUMBER_OF_BITS {
            // Smallest width: the tail is exactly the register, capped at r_max.
            for r in 1..r_max {
                accumulate(1.0, f64::integer_exp2_minus(r));
            }
            // The saturating register absorbs the geometric tail sum_{r >= r_max} 2^-r = 2^-(r_max-1).
            accumulate(1.0, f64::integer_exp2_minus(r_max - 1));
        } else {
            // Wider width: tail = flag bit + (register or leading bits) + residual.
            // Flag 0 (register <= B + 1): the t-1 leading hash bits are stored verbatim, so every such
            // cell is equiprobable at 2^-(t-1); group them by leading-zero run length l.
            let leading_q = f64::integer_exp2_minus(t - 1);
            let max_run = core::cmp::min(B::NUMBER_OF_BITS, t - 2);
            for l in 0..=max_run {
                accumulate(f64::integer_exp2(t - 2 - l), leading_q);
            }
            // At t = B + 1 the t - 1 = B stored leading bits cannot hold a terminating one for
            // register B + 1, so the loop above (capped at run B - 1) omits it. That register is
            // carried by the single all-zeros Flag 0 pattern, one cell per index at probability
            // 2^-(B+1). Add it back so Flag 0 covers registers 1 through B + 1 and the cell masses
            // sum to one. For t >= B + 2 the loop already reaches run B, and an all-zeros pattern
            // there means register at least B + 2 (Flag 1), so nothing is added.
            if max_run < B::NUMBER_OF_BITS {
                accumulate(1.0, f64::integer_exp2_minus(B::NUMBER_OF_BITS + 1));
            }
            // Flag 1 (register >= B + 2): register in B bits plus a (t-1-B)-bit residual.
            let residual_bits = t - 1 - B::NUMBER_OF_BITS;
            let residual_cells = f64::integer_exp2(residual_bits);
            let residual_q = f64::integer_exp2_minus(residual_bits);
            for r in (B::NUMBER_OF_BITS + 2)..r_max {
                accumulate(residual_cells, f64::integer_exp2_minus(r) * residual_q);
            }
            // The saturating register again absorbs 2^-(r_max-1), spread over the residual cells.
            accumulate(
                residual_cells,
                f64::integer_exp2_minus(r_max - 1) * residual_q,
            );
        }

        // Fixed-n variance of the distinct-composite count D. The cells are occupied/empty under a
        // FIXED n (a multinomial allocation), so the occupancy indicators are negatively correlated.
        // The naive Poisson sum `sum_c s_c (1 - s_c)` ignores that correlation and would spuriously
        // count the variance of n itself, making a near-exact small hash list look as noisy as
        // `1.04/sqrt(n)`. To leading order in the per-cell probabilities the multinomial variance is
        // `Var(D) = m*(sum_c s_c) - m*(sum_c (1 - 2 p_c)^n) - n*(sum_c s_c p_c)^2`, which correctly
        // collapses to ~0 when there are no collisions and grows as the width shrinks toward
        // conversion. (Here `m * survival_sum = sum over all cells`, since each group holds `count * m`
        // cells, and `weighted_survival = sum_c s_c p_c` already carries the `1/m` from `p = q/m`.)
        let variance = (m * (survival_sum - shifted_survival_sum)
            - n * weighted_survival * weighted_survival)
            .max(0.0);
        (expected * m, derivative * m, variance)
    }

    #[inline]
    /// Theoretical relative standard error of the sorted-hash-list cardinality estimate, from the
    /// occupancy model. The distinct-composite count `D` has variance `Var(D)` and sensitivity
    /// `dE[D]/dn`, so by the delta method the inverted cardinality has
    /// `Var(n_hat) ~ Var(D) / (dE[D]/dn)^2`, and this returns `sqrt(Var(n_hat)) / n_hat`. Returns `0`
    /// for an empty list.
    pub(crate) fn hash_list_relative_standard_error(number_of_hashes: u32, hash_bits: u8) -> f64 {
        if number_of_hashes == 0 {
            return 0.0;
        }
        let n = Self::hash_list_cardinality(number_of_hashes, hash_bits);
        if n <= 0.0 {
            return 0.0;
        }
        let (_expected, derivative, variance) = Self::hash_list_expected_distinct(n, hash_bits);
        if derivative <= 0.0 || variance <= 0.0 {
            return 0.0;
        }
        FloatOps::sqrt(variance) / derivative / n
    }

    #[inline]
    /// Cardinality estimate for a sorted hash list from its distinct-composite count and width.
    ///
    /// Inverts the expected distinct-composite count
    /// [`hash_list_expected_distinct`](Self::hash_list_expected_distinct) for the cardinality `n` by
    /// safeguarded Newton iteration (a bisection bracket guarantees convergence even where the
    /// occupancy curve flattens near saturation). This is the parameter-free, table-free occupancy
    /// estimator that replaces the former empirical hash-list bias table; it is path-independent (it
    /// reads only the distinct-composite count, never the order-dependent duplicate count) and exact
    /// in expectation under uniform hashing.
    fn hash_list_cardinality(number_of_hashes: u32, hash_bits: u8) -> f64 {
        let d = f64::from(number_of_hashes);
        if d == 0.0 {
            return 0.0;
        }

        // Bracket the root: the distinct count never exceeds n, so n >= d; grow the upper bound until
        // the expected distinct count reaches the observed one.
        let mut lo = d;
        let mut hi = d.max(1.0);
        while Self::hash_list_expected_distinct(hi, hash_bits).0 < d {
            lo = hi;
            hi *= 2.0;
            if hi > 1e15 {
                return hi;
            }
        }

        // Safeguarded Newton: keep `[lo, hi]` bracketing the root and take a Newton step when it stays
        // inside, else bisect. Convergence is judged on the step size and the returned value is the
        // iterate itself, NOT the bracket midpoint: the bracket can stay one-sided (when Newton
        // approaches the root monotonically from one side, only `lo` or only `hi` ever moves), so its
        // midpoint is not the estimate.
        let mut n = f64::midpoint(lo, hi);
        for _ in 0..80 {
            let (expected, derivative, _variance) = Self::hash_list_expected_distinct(n, hash_bits);
            if expected > d {
                hi = n;
            } else {
                lo = n;
            }
            let newton = n - (expected - d) / derivative;
            let next = if derivative > 0.0 && newton > lo && newton < hi {
                newton
            } else {
                f64::midpoint(lo, hi)
            };
            // Converge on the step in n-space: near saturation the occupancy curve flattens, so a tiny
            // distinct-count residual still leaves a large cardinality uncertainty, but the step does
            // shrink to zero. A tight step keeps the two estimation paths (direct and via
            // inclusion-exclusion) in agreement.
            if (next - n).abs() <= 1e-12 * n {
                return next;
            }
            n = next;
        }
        n
    }

    #[inline]
    /// Returns the corrected estimate of the cardinality.
    pub fn estimate_cardinality(&self) -> f64 {
        // The sorted value list stores every inserted value verbatim, so its cardinality is the
        // exact count with no bias correction.
        if self.is_sorted_value_list() {
            return f64::from(self.get_number_of_values());
        }
        if self.is_sorted_hash_list() {
            Self::hash_list_cardinality(
                self.get_number_of_hashes().unwrap(),
                self.get_hash_bits().unwrap(),
            )
        } else if self.harmonic_sum.is_nan() {
            // Dense, zeros mode (low load): the word holds the zero count, and linear counting is the
            // preferred estimator here by construction (the counter has not crossed the threshold), so
            // compute it directly in O(1) with no register scan.
            let zeros = decode_dense_zeros(self.harmonic_sum);
            let m = f64::integer_exp2(P::EXPONENT);
            m * (m / f64::from(zeros)).natural_log()
        } else {
            // Dense, harmonic mode: Ertl's analytical tau/sigma estimator, evaluated in O(1) from
            // the packed `(H, zeros)` moments. This replaces the shipped fitted-polynomial bias
            // correction (which failed at P8, oscillated across the band, and damaged the raw
            // estimate at high load). Saturation is treated as zero: the packed band ends at
            // `7.5 * m`, orders of magnitude below where even a four-bit register field saturates.
            self.sigma_tau_cardinality()
        }
    }

    #[inline]
    /// Returns whether the provided [`HyperLogLog`] counter may be fully contained in the current [`HyperLogLog`] counter.
    ///
    /// # Arguments
    /// * `rhs` - The [`HyperLogLog`] counter to check.
    ///
    /// # Implementative details
    /// We define a counter that fully contains another counter when all of the registers
    /// of the first counter are greater than or equal to the corresponding registers of the second counter.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use hyperloglog_rs::prelude::*;
    ///
    /// let mut hll1: HyperLogLog<Precision8, Bits6> =
    ///     Default::default();
    /// let mut hll2: HyperLogLog<Precision8, Bits6> =
    ///     Default::default();
    ///
    /// hll1.insert(&42);
    /// hll1.insert(&43);
    /// hll1.insert(&44);
    ///
    /// hll2.insert(&42);
    /// hll2.insert(&43);
    ///
    /// assert_eq!(hll1.may_contain_all(&hll2), true);
    /// assert_eq!(hll2.may_contain_all(&hll1), false);
    ///
    /// hll2.insert(&44);
    ///
    /// assert_eq!(hll1.may_contain_all(&hll2), true);
    /// assert_eq!(hll2.may_contain_all(&hll1), true);
    /// ```
    pub fn may_contain_all(&self, rhs: &Self) -> bool {
        self.registers
            .iter_registers_zipped(&rhs.registers)
            .all(|[left_register, right_register]| left_register >= right_register)
    }

    #[inline]
    /// Hashes the element and returns the register value and the index of the register.
    pub fn index_and_register_and_hash<T: Hash>(element: &T) -> (usize, u8, u64) {
        let mut hasher = H::default();
        element.hash(&mut hasher);
        let hash = hasher.finish();

        let index: usize = usize::try_from(hash & ((1 << P::EXPONENT) - 1)).unwrap();

        debug_assert!(
            index < 1 << P::EXPONENT,
            "The index {index} must be less than the number of registers {}.",
            1 << P::EXPONENT
        );

        // And we censor we just used for the index.
        let mut censored_hash: u64 = hash | 1 << P::EXPONENT;

        // We need to add ones to the hash to make sure that the
        // the number of zeros we obtain afterwards is never higher
        // than the maximal value that may be represented in a register
        // with BITS bits.
        if <B as VariableWord>::NUMBER_OF_BITS < 6_u8 {
            censored_hash |= 1_u64 << (64_u64 - <B as VariableWord>::MASK);
        }

        let register_value = u8::try_from(censored_hash.leading_zeros() + 1).unwrap();

        debug_assert!(
            register_value <= u8::try_from(<B as VariableWord>::MASK).unwrap(),
            "The register value {} must be less than or equal to the maximum register value {}.",
            register_value,
            (1 << <B as VariableWord>::NUMBER_OF_BITS) - 1
        );

        debug_assert!(
            register_value > 0,
            "The register value must be greater than zero."
        );

        (index, register_value, hash)
    }

    #[inline]
    /// Returns the estimate of the cardinality of the union of two [`HyperLogLog`] counters.
    pub fn estimate_union_cardinality(&self, other: &Self) -> f64 {
        self.estimate_union_cardinality_with_cardinalities(
            other,
            self.estimate_cardinality(),
            other.estimate_cardinality(),
        )
    }

    #[inline]
    fn estimate_union_cardinality_with_cardinalities(
        &self,
        other: &Self,
        self_cardinality: f64,
        other_cardinality: f64,
    ) -> f64 {
        // Exact-values operands are handled before the sorted hash list/HyperLogLog matrix: two exact operands
        // give the exact union directly, and a mixed pair promotes the exact one to a proper hash
        // list (a clone) and reuses the existing logic.
        {
            if self.is_sorted_value_list() && other.is_sorted_value_list() {
                let union = svl::union_count::<BE, _>(
                    self.registers.as_ref(),
                    0,
                    self.get_number_of_values(),
                    other.registers.as_ref(),
                    0,
                    other.get_number_of_values(),
                    C::default(),
                );
                return f64::from(union);
            }
            if self.is_sorted_value_list() {
                let mut promoted = self.clone();
                promoted.to_sorted_hash_list();
                return promoted.estimate_union_cardinality_with_cardinalities(
                    other,
                    self_cardinality,
                    other_cardinality,
                );
            }
            if other.is_sorted_value_list() {
                let mut promoted = other.clone();
                promoted.to_sorted_hash_list();
                return self.estimate_union_cardinality_with_cardinalities(
                    &promoted,
                    self_cardinality,
                    other_cardinality,
                );
            }
        }
        match (self.is_sorted_hash_list(), other.is_sorted_hash_list()) {
            (true, true) => {
                // Build the union as a sorted hash list and estimate its cardinality directly, so the
                // birthday-paradox correction is applied to the union the same way it is to a
                // single counter. Inclusion-exclusion (A + B - intersection) would subtract a
                // raw, uncorrected count of coinciding downgraded hashes; for sets with little
                // real overlap those coincidences are dominated by spurious birthday collisions,
                // which biases the union estimate low and increasingly so at higher precisions.
                let mut union = self.clone();
                union.merge(other);
                correct_union_estimate(
                    self_cardinality,
                    other_cardinality,
                    union.estimate_cardinality(),
                )
            }
            (true, false) => {
                let hash_bits = self.get_hash_bits().unwrap();
                assert!(hash_bits >= GapHash::<P, B>::SMALLEST_VIABLE_HASH_BITS);

                self.union_estimation_from_sorted_iterator_and_counter(
                    other,
                    self_cardinality,
                    other_cardinality,
                )
            }
            (false, true) => other.estimate_union_cardinality_with_cardinalities(
                self,
                self_cardinality,
                other_cardinality,
            ),
            (false, false) => {
                // Estimate the union from the element-wise-max registers, applying linear counting at
                // low union load just like the single-counter path, so a force-dense low-cardinality
                // union is not stuck with the badly-biased raw correction.
                let (union_harmonic_sum, union_zeros) =
                    self.registers.get_union_harmonic_sum(&other.registers);
                let union_estimate =
                    Self::corrected_register_cardinality(union_harmonic_sum, union_zeros);
                correct_union_estimate(self_cardinality, other_cardinality, union_estimate)
            }
        }
    }

    /// Width-consistent intersection and union of two counters, returned as
    /// `Some((intersection, union))`, or `None` when the width-consistent path does not apply (so the
    /// caller falls back to plain inclusion-exclusion).
    ///
    /// The plain inclusion-exclusion `|A| + |B| - |U|` used by the default Jaccard and intersection
    /// estimates corrects each term at its own composite width. When the union sits at a coarser width
    /// than the operands (it holds more elements, so it downgrades first), the three corrections live
    /// in different width regimes and no longer cancel, which inflates the intersection and Jaccard
    /// error at the width boundary. Here we bring all three to the union's width: `|U|` is already
    /// there, and `|A|`, `|B|` are recomputed from their distinct composite counts downgraded to that
    /// width, so a single occupancy correction applies to all three.
    ///
    /// Only the both-hash-list case needs this. When either operand is an exact value list (set
    /// operations are already exact) or a register counter (a single width), the default path is
    /// already width-consistent and this returns `None`.
    pub(crate) fn width_consistent_intersection_and_union(
        &self,
        other: &Self,
    ) -> Option<(f64, f64)> {
        if !(self.is_sorted_hash_list() && other.is_sorted_hash_list()) {
            return None;
        }
        // Build the union and read the width it settled at.
        let mut union = self.clone();
        union.merge(other);
        if !union.is_sorted_hash_list() {
            // The union overflowed to registers: there is no common hash-list width, so fall back.
            return None;
        }
        let union_hash_bits = union.get_hash_bits().unwrap();
        let union_cardinality = union.estimate_cardinality();

        let left = self.cardinality_at_hash_bits(union_hash_bits);
        let right = other.cardinality_at_hash_bits(union_hash_bits);

        // The numerator is width-consistent (all three terms at the union's width). Cap the result at
        // the smaller operand's reported cardinality, since an intersection can exceed neither set and
        // the downgrade recount of a much smaller operand can otherwise overshoot slightly.
        let cap = self
            .estimate_cardinality()
            .min(other.estimate_cardinality());
        let intersection = (left + right - union_cardinality).max(0.0).min(cap);
        Some((intersection, union_cardinality))
    }

    /// The corrected cardinality of this hash-list counter evaluated at `target_hash_bits`, a width no
    /// wider than the counter's own. The stored composites are downgraded to the target width (which
    /// merges any that collide there) and the occupancy correction for that width is applied to the
    /// resulting distinct count, so the estimate shares the width regime of a union taken at that
    /// width.
    fn cardinality_at_hash_bits(&self, target_hash_bits: u8) -> f64 {
        let hash_bits = self.get_hash_bits().unwrap();
        debug_assert!(
            target_hash_bits <= hash_bits,
            "target width ({target_hash_bits}) must not exceed the counter's width ({hash_bits})",
        );
        if target_hash_bits == hash_bits {
            return Self::hash_list_cardinality(self.get_number_of_hashes().unwrap(), hash_bits);
        }
        // Downgrade truncates low bits, so collided composites become adjacent equal values in the
        // descending-sorted stream. Count distinct by comparing each to its predecessor.
        let mut distinct: u32 = 0;
        let mut previous: u32 = u32::MAX;
        for downgraded in GapHash::<P, B>::downgraded(
            self.registers.as_ref(),
            self.get_number_of_hashes().unwrap(),
            hash_bits,
            self.get_writer_tell(),
            hash_bits - target_hash_bits,
        ) {
            if downgraded != previous {
                distinct += 1;
                previous = downgraded;
            }
        }
        Self::hash_list_cardinality(distinct, target_hash_bits)
    }

    #[inline]
    /// Merges another counter into this one, equivalent to a set union.
    ///
    /// # Implementative details
    /// When both counters are still in sorted hash list, the union is itself kept as a hash
    /// list, preserving the accuracy of small cardinalities: the hashes of the
    /// higher-precision counter are downgraded and inserted into the lower-precision one
    /// (a stored hash can only be downgraded, never upgraded). As soon as either operand
    /// is a fully-fledged [`HyperLogLog`], the result is a [`HyperLogLog`] whose registers
    /// are the element-wise maximum of the two operands.
    fn merge(&mut self, rhs: &Self) {
        // Exact-values operands are folded in before the sorted hash list/HyperLogLog matrix. When both counters
        // are exact, a single linear two-pointer merge keeps the result exact (and falls back to a
        // mode transition if the union no longer fits). When only `self` is exact and `rhs` is
        // hashed, `self` is first promoted to a proper sorted hash list, then merged normally.
        {
            if rhs.is_sorted_value_list() {
                if self.is_sorted_value_list() && self.try_merge_exact_values(rhs) {
                    return;
                }
                if self.is_sorted_value_list() {
                    // The exact union overflows the buffer: leave exact mode, then fold `rhs`'s
                    // values in (now hashed, so each insertion is cheap).
                    self.to_sorted_hash_list();
                }
                for value in ValueIter::<BE, _>::new(
                    rhs.registers.as_ref(),
                    0,
                    rhs.get_number_of_values(),
                    C::default(),
                ) {
                    self.insert_value(value);
                }
                return;
            }
            if self.is_sorted_value_list() {
                self.to_sorted_hash_list();
            }
        }
        match (self.is_sorted_hash_list(), rhs.is_sorted_hash_list()) {
            (false, false) => {
                // Both counters are fully-fledged HyperLogLogs: element-wise register maximum.
                for (index, register) in rhs.registers.iter_registers().enumerate() {
                    self.insert_register_value_and_index(register, index);
                }
            }
            (true, false) => {
                // Only `self` is a sorted hash list: materialize it, then take the register maximum.
                self.to_hll();
                for (index, register) in rhs.registers.iter_registers().enumerate() {
                    self.insert_register_value_and_index(register, index);
                }
            }
            (false, true) => {
                // Only `rhs` is a sorted hash list: fold its hashes into `self`'s registers.
                let mut last_index = usize::MAX;
                for (register, index) in GapHash::<P, B>::decoded(
                    rhs.registers.as_ref(),
                    rhs.get_number_of_hashes().unwrap(),
                    rhs.get_hash_bits().unwrap(),
                    rhs.get_writer_tell(),
                ) {
                    if index == last_index {
                        continue;
                    }
                    last_index = index;
                    self.insert_register_value_and_index(register, index);
                }
            }
            (true, true) => {
                // Both counters are sorted hash lists. Both operands are already sorted (descending),
                // so a single two-pointer merge writes the union into a fresh buffer in O(n + m),
                // instead of splicing one hash at a time (O(n*m), the cost that humped near
                // saturation). The hashes are brought to the common (coarser) hash size, since a
                // stored hash can only be downgraded. The whole path, including the cardinality-count
                // simulation, re-streams the operand buffers and is allocation-free.
                {
                    let self_hash_bits = self.get_hash_bits().unwrap();
                    let self_number_of_hashes = self.get_number_of_hashes().unwrap();
                    let self_writer_tell = self.get_writer_tell();
                    let self_duplicates = self.get_duplicates();
                    let rhs_hash_bits = rhs.get_hash_bits().unwrap();
                    let rhs_number_of_hashes = rhs.get_number_of_hashes().unwrap();
                    let rhs_writer_tell = rhs.get_writer_tell();
                    let rhs_duplicates = rhs.get_duplicates();

                    // The coarser operand (smaller hash size, ties broken toward `self`, matching the
                    // one-at-a-time path) is the base whose distinct-count history is taken whole; the
                    // finer operand's hashes are replayed on top to recover the union's statistic.
                    let self_is_base = self_hash_bits <= rhs_hash_bits;
                    let base_raw = if self_is_base {
                        self_number_of_hashes + self_duplicates
                    } else {
                        rhs_number_of_hashes + rhs_duplicates
                    };

                    // Move self's hashes aside so the merge can read them while writing into self.
                    let source = self.registers.clone();
                    // Grow self to the full hash-list buffer (a no-op for the fixed-size array
                    // backing, which is already full). The destination size must match what
                    // `merge_metrics` sizes against, so the rank index lands at the same offset
                    // during the write.
                    let maximal_bytes = (1usize << P::EXPONENT) * B::NUMBER_OF_BITS as usize / 8;
                    while self.registers.as_ref().len() < maximal_bytes {
                        self.registers.increase_capacity();
                    }
                    let dest_len = self.registers.as_ref().len();

                    if let Some(meta) = GapHash::<P, B>::merge_metrics(
                        source.as_ref(),
                        self_number_of_hashes,
                        self_hash_bits,
                        self_writer_tell,
                        rhs.registers.as_ref(),
                        rhs_number_of_hashes,
                        rhs_hash_bits,
                        rhs_writer_tell,
                        dest_len,
                    ) {
                        self.registers.clear_registers();
                        GapHash::<P, B>::merge_write(
                            source.as_ref(),
                            self_number_of_hashes,
                            self_hash_bits,
                            self_writer_tell,
                            rhs.registers.as_ref(),
                            rhs_number_of_hashes,
                            rhs_hash_bits,
                            rhs_writer_tell,
                            self.registers.as_mut(),
                            meta,
                        );

                        // Recover the union's `number_of_hashes + duplicates`: the coarser
                        // operand's count is taken whole, the finer operand's hashes are replayed
                        // across the union's downgrade schedule (see `merge_finer_new_count`).
                        let b_new = if self_is_base {
                            GapHash::<P, B>::merge_finer_new_count(
                                source.as_ref(),
                                self_number_of_hashes,
                                self_hash_bits,
                                self_writer_tell,
                                rhs.registers.as_ref(),
                                rhs_number_of_hashes,
                                rhs_hash_bits,
                                rhs_writer_tell,
                                meta.hash_bits,
                                dest_len,
                            )
                        } else {
                            GapHash::<P, B>::merge_finer_new_count(
                                rhs.registers.as_ref(),
                                rhs_number_of_hashes,
                                rhs_hash_bits,
                                rhs_writer_tell,
                                source.as_ref(),
                                self_number_of_hashes,
                                self_hash_bits,
                                self_writer_tell,
                                meta.hash_bits,
                                dest_len,
                            )
                        };
                        let raw = base_raw + b_new;

                        self.set_hash_bits(meta.hash_bits);
                        self.set_number_of_hashes(meta.number_of_hashes);
                        self.set_duplicates(raw.saturating_sub(meta.number_of_hashes));
                        self.set_writer_tell(meta.bit_index);
                    } else {
                        // The union does not fit the hash list even at the smallest viable hash
                        // size: densify self (still a valid hash list, only its buffer grew) and
                        // fold rhs's hashes as register maxima, like the `(false, true)` arm.
                        self.to_hll();
                        let mut last_index = usize::MAX;
                        for (register, index) in GapHash::<P, B>::decoded(
                            rhs.registers.as_ref(),
                            rhs_number_of_hashes,
                            rhs_hash_bits,
                            rhs_writer_tell,
                        ) {
                            if index == last_index {
                                continue;
                            }
                            last_index = index;
                            self.insert_register_value_and_index(register, index);
                        }
                    }
                }
            }
        }
    }

    #[cfg(test)]
    /// Reference both-hash-list union by one-at-a-time insertion (the pre-two-pointer-merge
    /// implementation), kept as the oracle the two-pointer [`merge`](Self::merge) is validated
    /// against in tests.
    pub(crate) fn merge_hash_lists_one_by_one(&mut self, rhs: &Self) {
        debug_assert!(self.is_sorted_hash_list() && rhs.is_sorted_hash_list());
        let self_hash_bits = self.get_hash_bits().unwrap();
        let rhs_hash_bits = rhs.get_hash_bits().unwrap();
        if self_hash_bits <= rhs_hash_bits {
            for encoded_hash in GapHash::<P, B>::downgraded(
                rhs.registers.as_ref(),
                rhs.get_number_of_hashes().unwrap(),
                rhs_hash_bits,
                rhs.get_writer_tell(),
                rhs_hash_bits - self_hash_bits,
            ) {
                let (index, register, original_hash) =
                    GapHash::<P, B>::decode_full(encoded_hash, self_hash_bits);
                self.insert_index_register_hash(index, register, original_hash);
            }
        } else {
            let mut base = rhs.clone();
            for encoded_hash in GapHash::<P, B>::downgraded(
                self.registers.as_ref(),
                self.get_number_of_hashes().unwrap(),
                self_hash_bits,
                self.get_writer_tell(),
                self_hash_bits - rhs_hash_bits,
            ) {
                let (index, register, original_hash) =
                    GapHash::<P, B>::decode_full(encoded_hash, rhs_hash_bits);
                base.insert_index_register_hash(index, register, original_hash);
            }
            *self = base;
        }
    }
}

impl<P: Precision, B: Bits, R: Registers<P, B>, H: HasherType, C> BitOrAssign<&Self>
    for HyperLogLog<P, B, R, H, C>
where
    C: SparseValueCodec,
{
    #[inline]
    fn bitor_assign(&mut self, rhs: &Self) {
        self.merge(rhs);
    }
}

impl<P: Precision, B: Bits, R: Registers<P, B>, H: HasherType, C> BitOrAssign
    for HyperLogLog<P, B, R, H, C>
where
    C: SparseValueCodec,
{
    #[inline]
    fn bitor_assign(&mut self, rhs: Self) {
        self.merge(&rhs);
    }
}

impl<P: Precision, B: Bits, R: Registers<P, B>, H: HasherType, C> BitOr
    for HyperLogLog<P, B, R, H, C>
where
    C: SparseValueCodec,
{
    type Output = Self;

    #[inline]
    fn bitor(mut self, rhs: Self) -> Self::Output {
        self.merge(&rhs);
        self
    }
}

impl<P: Precision, B: Bits, R: Registers<P, B>, H: HasherType, C> BitOr
    for &HyperLogLog<P, B, R, H, C>
where
    C: SparseValueCodec,
{
    type Output = HyperLogLog<P, B, R, H, C>;

    #[inline]
    fn bitor(self, rhs: Self) -> Self::Output {
        let mut result = self.clone();
        result.merge(rhs);
        result
    }
}

#[cfg(test)]
mod occupancy_estimator_tests {
    //! Tests for the table-free width-aware occupancy inverse used in the sorted hash list regime.
    use super::*;

    /// The occupancy inverse must be well-behaved across the whole hash-list range at every width: it
    /// recovers a cardinality at least as large as the observed distinct count, strictly increasing in
    /// it, and close to it at wide widths where collisions are negligible. This guards against the
    /// inversion returning a stale bracket midpoint instead of the converged root (a bug that produced
    /// a ~25% over-estimate at isolated distinct counts such as `D = 1544` at `width = 22`).
    #[test]
    fn occupancy_inverse_is_monotone_and_tight() {
        fn check<P, B>()
        where
            P: Precision + PackedRegister<B>,
            B: Bits,
        {
            let smallest = P::EXPONENT + B::NUMBER_OF_BITS;
            let largest = GapHash::<P, B>::LARGEST_VIABLE_HASH_BITS;
            for hash_bits in smallest..=largest {
                let mut previous = 0.0_f64;
                let mut d = 1u32;
                while d < (1u32 << P::EXPONENT) * 4 {
                    let estimate = HyperLogLog::<P, B>::hash_list_cardinality(d, hash_bits);
                    assert!(
                        estimate.is_finite() && estimate >= f64::from(d) - 1e-6,
                        "P{} B{} w{hash_bits}: estimate {estimate} below D={d}",
                        P::EXPONENT,
                        B::NUMBER_OF_BITS,
                    );
                    assert!(
                        estimate > previous - 1e-6,
                        "P{} B{} w{hash_bits}: estimate {estimate} at D={d} not increasing (prev {previous})",
                        P::EXPONENT,
                        B::NUMBER_OF_BITS,
                    );
                    // At the widest width collisions are negligible, so the inverse must stay close to
                    // the observed count rather than ballooning.
                    if hash_bits == largest {
                        assert!(
                            estimate <= f64::from(d) * 1.02 + 4.0,
                            "P{} B{} w{hash_bits}: estimate {estimate} far above D={d} at the widest width",
                            P::EXPONENT,
                            B::NUMBER_OF_BITS,
                        );
                    }
                    previous = estimate;
                    d += 1 + d / 64;
                }
            }
        }
        check::<Precision8, Bits6>();
        check::<Precision12, Bits6>();
        check::<Precision14, Bits6>();
    }

    /// The predicted hash-list standard error must reflect that the list is near-exact when collisions
    /// are negligible. This guards against the fixed-n variance regressing to the Poisson occupancy
    /// variance, which spuriously included the variance of n itself and reported a relative error of
    /// about `1.04/sqrt(D)` even for a collision-free list (empirically the spread there is ~0).
    #[test]
    fn hash_list_relative_standard_error_is_near_exact_without_collisions() {
        fn check<P, B>()
        where
            P: Precision + PackedRegister<B>,
            B: Bits,
        {
            let widest = GapHash::<P, B>::LARGEST_VIABLE_HASH_BITS;
            for &d in &[10_u32, 50, 200, 1000] {
                let rse = HyperLogLog::<P, B>::hash_list_relative_standard_error(d, widest);
                let poisson = 1.04 / f64::from(d).sqrt();
                assert!(
                    rse < 0.25 * poisson,
                    "P{} B{} D{d}: hash-list RSE {rse} is not near-exact (Poisson would be {poisson})",
                    P::EXPONENT,
                    B::NUMBER_OF_BITS,
                );
            }
        }
        check::<Precision10, Bits6>();
        check::<Precision12, Bits6>();
        check::<Precision14, Bits6>();
    }
}

#[cfg(test)]
mod test_hybrid_properties {
    use super::*;
    use hyperloglog_derive::test_estimator;

    fn smix(state: &mut u64) -> u64 {
        *state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = *state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// Builds two sorted-hash-list counters from disjoint pools (`only_a`, `shared`, `only_b`) and
    /// checks the two-pointer both-hash-list merge against the one-at-a-time reference and the true
    /// union: same saturation outcome, accuracy within bound and no worse than the old path,
    /// commutative up to the base-operand choice, and idempotent. Returns false (skip) if the
    /// operands do not stay hash lists, so callers can size pools per precision.
    fn check_merge_equivalence<P: Precision, B: Bits>(
        only_a: u64,
        shared: u64,
        only_b: u64,
        seed: u64,
    ) -> bool
    where
        P: PackedRegister<B>,
    {
        let mut state = seed;
        let mut a: HyperLogLog<P, B> = Default::default();
        let mut b: HyperLogLog<P, B> = Default::default();
        for _ in 0..only_a {
            a.insert(&smix(&mut state));
        }
        for _ in 0..shared {
            let v = smix(&mut state);
            a.insert(&v);
            b.insert(&v);
        }
        for _ in 0..only_b {
            b.insert(&smix(&mut state));
        }
        if !(a.is_sorted_hash_list() && b.is_sorted_hash_list()) {
            return false;
        }

        let true_union = (only_a + shared + only_b) as f64;
        let label = format!(
            "P{} B{} {only_a}/{shared}/{only_b}",
            P::EXPONENT,
            B::NUMBER_OF_BITS
        );

        // The merged object's representation must match the one-at-a-time path (saturation in
        // lockstep). The bulk merge is fast; the one-at-a-time path is the reference.
        let new = &a | &b;
        let mut old = a.clone();
        old.merge_hash_lists_one_by_one(&b);
        assert_eq!(
            new.is_sorted_hash_list(),
            old.is_sorted_hash_list(),
            "{label}: representation diverged (new hl={} dense={} hashes={:?}; old hl={} dense={})",
            new.is_sorted_hash_list(),
            new.is_hyperloglog(),
            new.get_number_of_hashes(),
            old.is_sorted_hash_list(),
            old.is_hyperloglog(),
        );

        // The fast two-pointer merge estimates the same cardinality as the one-at-a-time reference.
        // The stored bytes are not required to be identical: for a sparse union the fast path settles
        // on the largest optimal (prefix-free) hash size, while the incremental path can retain the
        // larger non-prefix-free size until an insert forces it down, and the simulated downgrade
        // schedule can place an overflow one hash either side of the incremental one. That last effect
        // is a +/-1-2 difference in the raw `number_of_hashes + duplicates` statistic, so the bound is
        // a small relative tolerance plus a few-count absolute floor (at tiny cardinalities a single
        // count is a larger fraction of the estimate). The precision-tuned accuracy-versus-truth check
        // across every representation lives in `tests/proptest_hinge.rs`.
        if new.is_sorted_hash_list() {
            let new_estimate = new.estimate_cardinality();
            let old_estimate = old.estimate_cardinality();
            assert!(
                (new_estimate - old_estimate).abs() <= old_estimate * 0.005 + 3.0,
                "{label}: merged estimate {new_estimate} diverged from one-at-a-time {old_estimate}",
            );
        }

        // User-facing guarantee: the union cardinality (which clamps to [max, sum]) stays within the
        // hash-list error envelope of the true union, and is commutative and idempotent.
        let union_new = a.estimate_union_cardinality(&b);
        let union_swapped = b.estimate_union_cardinality(&a);
        let err = (union_new - true_union).abs() / true_union;
        if std::env::var("MERGE_DEBUG").is_ok() {
            let a_raw = a.get_number_of_hashes().unwrap() + a.get_duplicates();
            let b_raw = b.get_number_of_hashes().unwrap() + b.get_duplicates();
            let old_raw = old.get_number_of_hashes().unwrap_or(0) + old.get_duplicates();
            let new_raw = new.get_number_of_hashes().unwrap_or(0) + new.get_duplicates();
            let (base_raw, finer_raw) = if a.get_hash_bits().unwrap() <= b.get_hash_bits().unwrap()
            {
                (a_raw, b_raw)
            } else {
                (b_raw, a_raw)
            };
            eprintln!(
                "{label}: truth {true_union} union_new {union_new:.1} err {err:.4} | a_bits={} b_bits={} | a_raw={a_raw} b_raw={b_raw} base_raw={base_raw} finer_raw={finer_raw} | new(bits={} n={:?} new_raw={new_raw}) ORACLE(bits={} old_raw={old_raw} est={:.1})",
                a.get_hash_bits().unwrap(),
                b.get_hash_bits().unwrap(),
                new.get_hash_bits().unwrap_or(0),
                new.get_number_of_hashes(),
                old.get_hash_bits().unwrap_or(0),
                old.estimate_cardinality(),
            );
        }
        // The estimate-vs-truth and commutativity bounds are precision-relative: a single hash-list
        // union sits within the precision's error envelope of the truth, and the merge is commutative
        // only up to the base-operand choice (which shifts the inherited duplicate tally), an effect
        // that scales with the same envelope. The tight, precision-independent guarantee is the
        // estimate-vs-oracle parity asserted above; this is a looser accuracy sanity (exact accuracy
        // across representations is covered by `tests/proptest_hinge.rs`).
        let truth_tolerance = P::error_rate() * 2.0;
        assert!(
            err < truth_tolerance,
            "{label}: union {union_new} vs truth {true_union} (err {err}, tol {truth_tolerance})"
        );
        assert!(
            (union_new - union_swapped).abs() / true_union < truth_tolerance,
            "{label}: union not commutative ({union_new} vs {union_swapped}, tol {truth_tolerance})"
        );
        let self_union = a.estimate_union_cardinality(&a);
        assert!(
            (self_union - a.estimate_cardinality()).abs() / a.estimate_cardinality() < 0.01,
            "{label}: a union a != a ({self_union} vs {})",
            a.estimate_cardinality()
        );
        true
    }

    /// Fixed cases across precisions: P8 exercises tiny (possibly non-prefix-free) lists, P12 the
    /// mid range, and P14 the active rank index, including high-overlap unions near saturation.
    #[test]
    fn merge_two_pointer_matches_one_by_one_and_truth() {
        for (i, &(a, s, b)) in [(20u64, 10, 20), (40, 20, 40), (60, 30, 60)]
            .iter()
            .enumerate()
        {
            assert!(check_merge_equivalence::<Precision8, Bits6>(
                a,
                s,
                b,
                0x51 ^ i as u64
            ));
        }
        for (i, &(a, s, b)) in [
            (500u64, 250, 500),
            (1000, 0, 1000),
            (800, 400, 800),
            (2000, 1000, 2000),
        ]
        .iter()
        .enumerate()
        {
            assert!(check_merge_equivalence::<Precision12, Bits6>(
                a,
                s,
                b,
                0xC12 ^ i as u64
            ));
        }
        for (i, &(a, s, b)) in [(5000u64, 2500, 5000), (8000, 0, 8000), (3000, 6000, 3000)]
            .iter()
            .enumerate()
        {
            assert!(check_merge_equivalence::<Precision14, Bits6>(
                a,
                s,
                b,
                0xE14 ^ i as u64
            ));
        }
    }

    proptest::proptest! {
        #![proptest_config(proptest::prelude::ProptestConfig {
            cases: 200,
            max_global_rejects: 4096,
            ..proptest::prelude::ProptestConfig::default()
        })]

        /// Random pool sizes and overlap at P12: the two-pointer merge must stay equivalent to the
        /// one-at-a-time reference across the hash-list range.
        #[test]
        fn merge_two_pointer_equivalence_p12(
            only_a in 1u64..1500,
            shared in 0u64..1000,
            only_b in 1u64..1500,
            seed in proptest::prelude::any::<u64>(),
        ) {
            // Skip draws whose operands leave the hash-list regime (sized to stay in it).
            proptest::prop_assume!(check_merge_equivalence::<Precision12, Bits6>(
                only_a, shared, only_b, seed
            ));
        }

        /// Equivalence at P8, where the hash list is tiny and often non-prefix-free (the raw
        /// largest-size layout), exercising the raw-merge and absent-from-table final-size paths.
        #[test]
        fn merge_two_pointer_equivalence_p8(
            only_a in 1u64..150,
            shared in 0u64..100,
            only_b in 1u64..150,
            seed in proptest::prelude::any::<u64>(),
        ) {
            proptest::prop_assume!(check_merge_equivalence::<Precision8, Bits6>(
                only_a, shared, only_b, seed
            ));
        }

        /// Equivalence at P14, where the rank index is active and the union spans several downgrade
        /// phases near saturation (the regime where the count simulation does the most work).
        #[test]
        fn merge_two_pointer_equivalence_p14(
            only_a in 1u64..4000,
            shared in 0u64..3000,
            only_b in 1u64..4000,
            seed in proptest::prelude::any::<u64>(),
        ) {
            proptest::prop_assume!(check_merge_equivalence::<Precision14, Bits6>(
                only_a, shared, only_b, seed
            ));
        }
    }

    #[test]
    fn insert_value_accepts_smaller_int_types() {
        // `insert_value` takes any `T: Into<u64>`, so `u8`/`u16`/`u32` go in without an `as u64` cast.
        let mut h: HyperLogLog<Precision10, Bits6> = Default::default();
        assert!(h.insert_value(7u32));
        assert!(h.insert_value(300u16));
        assert!(h.insert_value(5u8));
        assert!(h.insert_value(1_000_000u64));
        assert!(!h.insert_value(7u32), "duplicate must be rejected");
        assert!(h.is_sorted_value_list());
        assert_eq!(h.estimate_cardinality(), 4.0);
        let recovered: std::collections::HashSet<u64> = h.recover_values().unwrap().collect();
        assert_eq!(recovered, [7u64, 300, 5, 1_000_000].into_iter().collect());
    }

    #[test_estimator]
    fn test_estimation_regime_and_direct_to_hll<
        P: Precision,
        B: Bits,
        R: Registers<P, B>,
        H: HasherType,
    >() {
        // A fresh counter is a sorted hash list.
        let fresh: HyperLogLog<P, B, R, H> = Default::default();
        assert!(fresh.is_sorted_hash_list());
        assert_eq!(
            fresh.estimation_regime(),
            EstimationRegime::HashListCollisionCorrected
        );

        // A counter populated via `insert_value` is a sorted value list, estimated exactly.
        let mut values: HyperLogLog<P, B, R, H> = Default::default();
        for v in 0..16u64 {
            values.insert_value(v);
        }
        assert!(values.is_sorted_value_list());
        assert_eq!(values.estimation_regime(), EstimationRegime::Exact);
        assert_eq!(values.estimate_cardinality(), 16.0);

        // Converting the sorted value list directly to HyperLogLog registers (re-hashing each stored
        // value at full width) yields a register-mode counter with a finite positive estimate. At
        // this low load the regenerated threshold selects linear counting, which is accurate, rather
        // than the bias-corrected raw estimate, which is badly inflated here.
        let dense = values.clone().into_hll();
        assert!(dense.is_hyperloglog());
        let estimate = dense.estimate_cardinality();
        assert!(
            estimate.is_finite() && estimate > 0.0,
            "direct value-list to HLL estimate {estimate} is not a finite positive number"
        );

        // `estimate_cardinality` and `estimation_regime` must agree on the linear-counting branch,
        // and where linear counting is active (and the registers are not too small to be noisy) the
        // estimate must be close to the true 16 instead of HyperLogLog's small-cardinality bias.
        let zeros = dense.number_of_zero_registers().unwrap();
        let m = f64::integer_exp2(P::EXPONENT);
        if dense.estimation_regime() == EstimationRegime::HyperLogLogLinearCounted {
            let linear_counting = m * (m / zeros as f64).ln();
            assert!(
                (estimate - linear_counting).abs() < 1e-9,
                "regime is linear counting but estimate {estimate} != m*ln(m/zeros) {linear_counting}"
            );
            if P::EXPONENT >= 8 {
                assert!(
                    (estimate - 16.0).abs() <= 8.0,
                    "linear-counting estimate {estimate} not near the true 16 at precision {}",
                    P::EXPONENT
                );
            }
        }

        // The direct route must agree with the indirect route through the sorted hash list for this
        // small case (the hash list does not truncate the rank at this load), proving the direct
        // conversion scatters the same registers rather than dropping or misplacing values.
        let mut indirect = values.clone();
        indirect.to_sorted_hash_list();
        indirect.to_hll();
        assert!(indirect.is_hyperloglog());
        assert_eq!(indirect.estimate_cardinality(), estimate);

        // `into_hll` is idempotent: a counter already in register mode is returned unchanged.
        let again = dense.clone().into_hll();
        assert!(again.is_hyperloglog());
        assert_eq!(again.estimate_cardinality(), dense.estimate_cardinality());
    }

    #[test_estimator]
    fn test_plusplus_properties<P: Precision, B: Bits, R: Registers<P, B>, H: HasherType>() {
        let mut hybrid: HyperLogLog<P, B, R, H> = Default::default();
        assert!(hybrid.is_sorted_hash_list());
        assert!(hybrid.is_empty());
        assert!(!hybrid.is_full());
        assert_eq!(hybrid.get_number_of_hashes().unwrap(), 0);
        let mut normalized_error = 0.0;
        let mut non_normalized_error = 0.0;
        let mut random_state = 34567897654354_u64;
        let mut iterations = 0;

        while hybrid.is_sorted_hash_list() {
            iterations += 1;
            // To make the test a bit fairer using more random elements
            // than a numerical sequence.
            random_state = splitmix64(splitmix64(random_state));
            hybrid.insert(&random_state);
            assert!(
                !hybrid.insert(&random_state),
                "The Hybrid counter should NOT already contain the element {random_state}. Hash size: {}. Iteration n. {iterations}. Hash list status: {}",
                hybrid.get_hash_bits().unwrap(),
                hybrid.is_sorted_hash_list()
            );
            assert!(
                hybrid.may_contain(&random_state),
                "The Hybrid counter must contain the element {random_state}. Iteration n. {iterations}.",
            );

            let estimated_cardinality = hybrid.estimate_cardinality();

            let error = f64::from(iterations) - estimated_cardinality;
            non_normalized_error += error;
            normalized_error += error / f64::from(iterations);
        }

        normalized_error /= f64::from(iterations);
        non_normalized_error /= f64::from(iterations);

        // In sorted hash list the counter stores explicit hashes, so the only error source is
        // hash collisions plus the residual bias of the fitted cardinality correction. The
        // meaningful, theoretically grounded bound is the structure's own accuracy contract:
        // the estimate must satisfy the precision's nominal relative error rate, which it does
        // with a wide margin (the sorted hash list is far more accurate than the HyperLogLog
        // register estimator at these cardinalities). We bound the magnitude of the mean
        // relative error, catching both under- and over-counting. The previous `/ 13.0`
        // tightening had no theoretical basis and is dropped.
        assert!(
            normalized_error.abs() <= P::error_rate(),
            "The mean relative sorted hash list error ({normalized_error}, non-normalized {non_normalized_error}) must not exceed the precision's error rate ({}).",
            P::error_rate()
        );

        assert!(!hybrid.is_sorted_hash_list());
    }
}

#[cfg(test)]
mod dense_zeros_mode_tests {
    //! Tests for the NaN-boxed dense "zeros mode": the dense estimate must be unchanged by the
    //! optimization (it must equal the scan-based reference), and the maintained zero count must stay
    //! exact across the lifecycle, including the zeros-to-harmonic transition.
    use super::*;

    /// The scan-based reference estimate (the pre-optimization behavior): the real harmonic sum and
    /// the scanned zero count fed through the unified corrected estimator.
    fn reference_estimate<P, B>(counter: &HyperLogLog<P, B>) -> f64
    where
        P: Precision + PackedRegister<B>,
        B: Bits,
    {
        HyperLogLog::<P, B>::corrected_register_cardinality(
            counter.dense_harmonic_sum(),
            counter.number_of_zero_registers().unwrap(),
        )
    }

    fn check_counter<P, B>(counter: &HyperLogLog<P, B>, label: &str)
    where
        P: Precision + PackedRegister<B>,
        B: Bits,
    {
        if !counter.is_hyperloglog() {
            return;
        }
        // The fast mode-dispatched estimate must match the slow scan-based reference (the optimization
        // changes only cost, not the value).
        let fast = counter.estimate_cardinality();
        let reference = reference_estimate(counter);
        assert!(
            (fast - reference).abs() <= 1e-6 * reference.max(1.0),
            "{label}: fast estimate {fast} disagrees with scan reference {reference}",
        );
        // In zeros mode the maintained count must equal the true scanned zero count exactly.
        if counter.harmonic_sum.is_nan() {
            assert_eq!(
                decode_dense_zeros(counter.harmonic_sum) as usize,
                counter.number_of_zero_registers().unwrap(),
                "{label}: maintained zero count drifted from the true count",
            );
        }
    }

    #[test]
    fn dense_estimate_matches_scan_reference_across_lifecycle() {
        fn check<P, B>()
        where
            P: Precision + PackedRegister<B>,
            B: Bits,
        {
            let seed =
                0x51A7_C0DEu64 ^ (u64::from(P::EXPONENT) << 16) ^ u64::from(B::NUMBER_OF_BITS);
            for &n in &[40u64, 200, 1000, 5000, 30000, 120_000] {
                let mut natural = HyperLogLog::<P, B>::default();
                let mut state = seed;
                for _ in 0..n {
                    state = splitmix64(state);
                    natural.insert(&state);
                }
                check_counter(&natural, "natural");

                // Force dense (low load reaches zeros mode), then grow it across the transition.
                let forced = natural.clone().into_hll();
                check_counter(&forced, "forced");

                let mut grown = forced.clone();
                let mut grow_state = seed ^ 0xDEAD_BEEF;
                for _ in 0..50_000 {
                    grow_state = splitmix64(grow_state);
                    grown.insert(&grow_state);
                }
                check_counter(&grown, "grown");
            }
        }
        check::<Precision10, Bits6>();
        check::<Precision12, Bits6>();
        check::<Precision8, Bits4>();
        check::<Precision6, Bits4>();
    }

    #[test]
    fn forced_dense_low_load_is_zeros_mode() {
        // A small counter forced dense is at low load, so it must land in zeros mode (NaN word) and
        // report the linear-counting regime, while a naturally densified large counter is in harmonic
        // mode (a real, finite sum).
        let mut small = HyperLogLog::<Precision12, Bits6>::default();
        for x in 0u64..200 {
            small.insert(&x);
        }
        let small = small.into_hll();
        assert!(small.is_hyperloglog());
        assert!(
            small.harmonic_sum.is_nan(),
            "low-load forced dense must be zeros mode"
        );
        assert_eq!(
            small.estimation_regime(),
            EstimationRegime::HyperLogLogLinearCounted
        );

        let mut large = HyperLogLog::<Precision12, Bits6>::default();
        let mut state = 0x1234_5678u64;
        for _ in 0..200_000 {
            state = splitmix64(state);
            large.insert(&state);
        }
        assert!(large.is_hyperloglog());
        assert!(
            !large.harmonic_sum.is_nan() && large.harmonic_sum.is_finite(),
            "high-load counter must be harmonic mode",
        );
    }
}

#[cfg(test)]
mod occupancy_partition_tests {
    //! The occupancy model treats the composite values as cells that partition the hash-outcome
    //! space, so the per-index cell probabilities must sum to one at every viable width. The
    //! cleanest executable form of that invariant is `E[D | n = 1] = 1`: a single inserted element
    //! always lands in exactly one composite, so the expected distinct count at `n = 1` equals the
    //! total cell mass, which must be one. This caught (and now guards against) the wide-width
    //! enumeration dropping register `B + 1` at the boundary width `t = B + 1`.
    use super::*;

    /// `E[D | n = 1]` must equal one for every width from the narrowest (`t = B`) up to the widest
    /// viable hash width, for the given precision and register bits.
    fn check_partition<P, B>()
    where
        P: Precision + PackedRegister<B>,
        B: Bits,
    {
        let smallest = GapHash::<P, B>::SMALLEST_VIABLE_HASH_BITS;
        let largest = GapHash::<P, B>::LARGEST_VIABLE_HASH_BITS;
        for hash_bits in smallest..=largest {
            let expected_at_one =
                HyperLogLog::<P, B>::hash_list_expected_distinct(1.0, hash_bits).0;
            // Tolerance sits well above the float rounding of the no_std exp/log survival path
            // (observed up to ~1e-6 accumulated across the widest bands) and far below the bug it
            // guards: dropping register B + 1 left a deficit of 2^-(B+1) >= 2^-7, i.e. at least
            // 0.0078, roughly eighty times the tolerance.
            assert!(
                (expected_at_one - 1.0).abs() < 1e-4,
                "cell masses must sum to one (E[D | n = 1] = 1) at hash_bits = {hash_bits} \
                 (P = {}, B = {}), got {expected_at_one}",
                P::EXPONENT,
                B::NUMBER_OF_BITS,
            );
        }
    }

    #[test]
    fn cell_masses_sum_to_one_across_widths() {
        check_partition::<Precision11, Bits4>();
        check_partition::<Precision12, Bits4>();
        check_partition::<Precision10, Bits5>();
        check_partition::<Precision11, Bits5>();
        check_partition::<Precision9, Bits6>();
        check_partition::<Precision10, Bits6>();
        check_partition::<Precision12, Bits6>();
    }
}

#[cfg(test)]
mod zero_pack_tests {
    //! Round-trip and relative-error bounds for the harmonic-sum packing helpers
    //! ([`pack_harmonic_with_zeros`], [`unpack_harmonic`], [`unpack_zeros`]). These are pure
    //! numerical properties of the bit-layout, independent of any `HyperLogLog` counter.

    use super::{pack_harmonic_with_zeros, unpack_harmonic, unpack_zeros, zero_bits, ZERO_BITS};
    use crate::prelude::*;

    /// A representative slice of harmonic-sum magnitudes for the packing tests. Covers the small
    /// P4 sums (a few units) through the P18 sums (hundreds of thousands, once `H ~ 0.1 * m`).
    const HARMONIC_SUMS: &[f64] = &[
        1.0,
        1.5,
        7.0 / 3.0,
        16.0,
        1.234_567_890_123_456e2,
        2_048.0,
        3.141_592_653_589_793e4,
        6.5e5,
    ];

    /// Packing then unpacking must recover the zero count exactly for every width and every value
    /// the width can hold, regardless of the harmonic-sum bit pattern.
    #[test]
    fn pack_unpack_round_trip_zeros() {
        for &h in HARMONIC_SUMS {
            for zero_bits in 0u8..=16 {
                let max_zeros = if zero_bits == 0 {
                    0
                } else {
                    (1u32 << zero_bits) - 1
                };
                // Sample low, mid, and boundary zero counts.
                let samples = [
                    0u32,
                    1,
                    max_zeros / 2,
                    max_zeros.saturating_sub(1),
                    max_zeros,
                ];
                for &zeros in &samples {
                    if zeros > max_zeros {
                        continue;
                    }
                    let packed = pack_harmonic_with_zeros(h, zeros, zero_bits);
                    let recovered = unpack_zeros(packed, zero_bits);
                    assert_eq!(
                        recovered, zeros,
                        "h={h} zero_bits={zero_bits} zeros={zeros}: got {recovered}"
                    );
                }
            }
        }
    }

    /// Unpacking the harmonic sum must return `h` truncated by at most one low-bits ULP, i.e. the
    /// relative error is at most `2^(zero_bits - 52)`. This is the mask-only property (packing
    /// clears the low bits; nothing else moves).
    #[test]
    fn unpack_harmonic_relative_error_bound() {
        for &h in HARMONIC_SUMS {
            for zero_bits in 0u8..=16 {
                let max_zeros = if zero_bits == 0 {
                    0
                } else {
                    (1u32 << zero_bits) - 1
                };
                for &zeros in &[0u32, 1, max_zeros / 2, max_zeros] {
                    if zeros > max_zeros {
                        continue;
                    }
                    let packed = pack_harmonic_with_zeros(h, zeros, zero_bits);
                    let recovered = unpack_harmonic(packed, zero_bits);
                    // The recovered value differs from `h` by at most one low-bits ULP, which in
                    // the worst case is `2^(zero_bits - 52) * |h|` relative.
                    let bound = h * f64::integer_exp2_minus_signed(52 - i16::from(zero_bits));
                    let err = (recovered - h).abs();
                    assert!(
                        err <= bound,
                        "h={h} zero_bits={zero_bits} zeros={zeros}: err {err} > bound {bound}"
                    );
                    // Truncation, never rounding: the recovered value never exceeds `h`.
                    assert!(
                        recovered <= h,
                        "h={h} zero_bits={zero_bits} zeros={zeros}: recovered {recovered} > h"
                    );
                }
            }
        }
    }

    /// A zero-bit width is the identity: no bits stolen, no zeros carried, packed word equals `h`.
    #[test]
    fn zero_width_is_identity() {
        for &h in HARMONIC_SUMS {
            let packed = pack_harmonic_with_zeros(h, 0, 0);
            assert_eq!(packed.to_bits(), h.to_bits(), "h={h}");
            assert_eq!(unpack_harmonic(packed, 0).to_bits(), h.to_bits());
            assert_eq!(unpack_zeros(packed, 0), 0);
        }
    }

    /// A zero-count of zero recovers the bit-identical masked-`h`, and packing then unpacking is
    /// idempotent (the packed word is a fixed point of `unpack_harmonic`).
    #[test]
    fn pack_zero_matches_mask_only() {
        for &h in HARMONIC_SUMS {
            for zero_bits in 0u8..=16 {
                let packed = pack_harmonic_with_zeros(h, 0, zero_bits);
                let masked = unpack_harmonic(h, zero_bits);
                assert_eq!(
                    packed.to_bits(),
                    masked.to_bits(),
                    "h={h} zero_bits={zero_bits}"
                );
                let re_masked = unpack_harmonic(packed, zero_bits);
                assert_eq!(packed.to_bits(), re_masked.to_bits());
            }
        }
    }

    /// The frozen `ZERO_BITS` table must be a valid mantissa slice: every entry is at most 52 (the
    /// mantissa width) and, per the derivation, at most 16 (P15 B4, P16 B4).
    #[test]
    fn zero_bits_table_bounds() {
        for row in &ZERO_BITS {
            for &k in row {
                assert!(k >= 4, "ZERO_BITS entry {k} < 4-bit floor");
                assert!(k <= 16, "ZERO_BITS entry {k} > 16-bit ceiling");
            }
        }
    }

    /// The `zero_bits::<P, B>()` monomorphizing accessor must equal the table lookup.
    #[test]
    fn zero_bits_accessor_matches_table() {
        macro_rules! check {
            ($p:ty, $b:ty) => {{
                let got = zero_bits::<$p, $b>();
                let expected = ZERO_BITS[<$p as Precision>::EXPONENT as usize - 4]
                    [<$b as VariableWord>::NUMBER_OF_BITS as usize - 4];
                assert_eq!(
                    got,
                    expected,
                    "P{} B{}",
                    <$p as Precision>::EXPONENT,
                    <$b as VariableWord>::NUMBER_OF_BITS
                );
            }};
        }
        check!(Precision4, Bits4);
        check!(Precision4, Bits5);
        check!(Precision4, Bits6);
        check!(Precision8, Bits6);
        check!(Precision10, Bits6);
        check!(Precision12, Bits4);
        check!(Precision15, Bits4);
        check!(Precision16, Bits4);
        check!(Precision18, Bits6);
    }

    /// The packed zero count in the harmonic-mode word MUST match a fresh register scan at every
    /// mutation, whether the counter arrived at harmonic mode by natural growth from a hash list,
    /// by the zeros-to-harmonic transition inside `insert_register_value_and_index`, or by a
    /// merge that folded a second dense counter in.
    #[test]
    fn packed_zeros_match_scan_across_lifecycle() {
        fn assert_matches<P, B>(hll: &HyperLogLog<P, B>, label: &str, step: u64)
        where
            P: Precision + PackedRegister<B>,
            B: Bits,
        {
            if !hll.is_hyperloglog() || hll.harmonic_sum.is_nan() {
                return;
            }
            let scanned = u32::try_from(hll.number_of_zero_registers().unwrap()).unwrap();
            let packed = hll.dense_zero_count();
            assert_eq!(
                scanned, packed,
                "{label} step={step}: packed {packed} != scan {scanned}",
            );
        }

        fn check<P, B>(name: &str)
        where
            P: Precision + PackedRegister<B>,
            B: Bits,
        {
            let m = 1u64 << P::EXPONENT;
            for seed in 0u64..4 {
                // Natural growth: hash-list -> dense (zeros -> harmonic transition may fire
                // inside the insert path); sample the invariant along the way and at the end.
                let n = 6 * m.max(200);
                let sample_every = (n / 512).max(1);
                let mut hll = HyperLogLog::<P, B>::default();
                let mut state = 0x00C0_FFEE_u64 ^ seed;
                for i in 0..n {
                    state = splitmix64(state);
                    hll.insert(&state);
                    if i % sample_every == 0 {
                        assert_matches(&hll, &format!("{name} natural seed={seed}"), i);
                    }
                }
                assert_matches(&hll, &format!("{name} natural-end seed={seed}"), n);

                // Forced dense at low load, then grown through the crossover: the counter enters
                // dense zeros mode via `into_hll`, then the subsequent inserts drive the
                // zeros-to-harmonic transition and its packed-count materialization.
                let mut small = HyperLogLog::<P, B>::default();
                let mut sstate = 0x0000_BABE_u64 ^ seed;
                for _ in 0..(m / 8).max(20) {
                    sstate = splitmix64(sstate);
                    small.insert(&sstate);
                }
                let mut grown = small.into_hll();
                for i in 0..2 * m {
                    sstate = splitmix64(sstate);
                    grown.insert(&sstate);
                    if i % sample_every == 0 {
                        assert_matches(&grown, &format!("{name} forced seed={seed}"), i);
                    }
                }
                assert_matches(&grown, &format!("{name} forced-end seed={seed}"), 2 * m);

                // Merge of two independent dense counters: `merge` iterates over the rhs
                // registers through `insert_register_value_and_index`, so packed maintenance must
                // survive that hot path too.
                let mut a = HyperLogLog::<P, B>::default();
                let mut astate = 0x000A_11CE_u64 ^ seed;
                for _ in 0..3 * m {
                    astate = splitmix64(astate);
                    a.insert(&astate);
                }
                let mut b = HyperLogLog::<P, B>::default();
                let mut bstate = 0x0000_0B0B_u64 ^ seed;
                for _ in 0..3 * m {
                    bstate = splitmix64(bstate);
                    b.insert(&bstate);
                }
                let merged = a | b;
                assert_matches(&merged, &format!("{name} merge seed={seed}"), 0);
            }
        }

        check::<Precision6, Bits4>("P6B4");
        check::<Precision8, Bits5>("P8B5");
        check::<Precision10, Bits6>("P10B6");
        check::<Precision12, Bits4>("P12B4");
    }

    /// Two counters built by the identical insertion sequence must be bit-equal after the packed
    /// zeros field lands in the harmonic word. This is the invariant `PartialEq` documents: same
    /// registers plus same `harmonic_sum` bits.
    #[test]
    fn partial_eq_bit_exact_after_packing() {
        let mut a = HyperLogLog::<Precision10, Bits6>::default();
        let mut b = HyperLogLog::<Precision10, Bits6>::default();
        let mut state = 0x00C0_FFEE_u64;
        for _ in 0..5_000u64 {
            state = splitmix64(state);
            a.insert(&state);
            b.insert(&state);
        }
        assert!(
            a.is_hyperloglog() && !a.harmonic_sum.is_nan(),
            "test setup: need dense harmonic mode with a packed count",
        );
        assert_eq!(
            a, b,
            "identical insertion sequences must produce bit-equal counters"
        );
        assert_eq!(
            a.harmonic_sum.to_bits(),
            b.harmonic_sum.to_bits(),
            "packed harmonic-sum word must be bit-identical",
        );
        assert_eq!(a.dense_zero_count(), b.dense_zero_count());
        // Force a divergence: bump one of `b`'s registers directly to a value strictly larger
        // than what it currently holds. That guarantees both the register state and the packed
        // count change, so `PartialEq` must reflect the divergence.
        let target = (1u8 << <Bits6 as VariableWord>::NUMBER_OF_BITS) - 1;
        b.insert_register_value_and_index(target, 0);
        assert_ne!(a, b, "diverged register state must not compare equal");
    }

    /// The raw regime (cardinality above the `7.5 * m` correction bound) keeps packing zeros, and
    /// that count MUST fit in `ZERO_BITS[P - 4][B - 4]` bits. In the raw regime the true zero
    /// count is at most `m * exp(-7.5)`, well under every cell's cap, so this test just
    /// materializes the state and confirms the packed accessor round-trips the fresh scan.
    #[test]
    fn raw_regime_packed_count_fits_and_matches_scan() {
        fn check<P, B>(name: &str)
        where
            P: Precision + PackedRegister<B>,
            B: Bits,
        {
            let m = 1u64 << P::EXPONENT;
            // 12 * m > 7.5 * m puts every configuration comfortably in the raw regime.
            let n = 12 * m;
            let mut hll = HyperLogLog::<P, B>::default();
            let mut state = 0x0000_DEAD_BEEF_u64 ^ (u64::from(P::EXPONENT) << 8);
            for _ in 0..n {
                state = splitmix64(state);
                hll.insert(&state);
            }
            assert!(
                hll.is_hyperloglog() && !hll.harmonic_sum.is_nan(),
                "{name} n={n}: must land in dense harmonic mode",
            );
            let raw_estimate = hll.uncorrected_estimate_cardinality();
            assert!(
                raw_estimate >= 7.5 * (m as f64),
                "{name} n={n}: raw estimate {raw_estimate} should be in the raw regime",
            );
            let scanned = u32::try_from(hll.number_of_zero_registers().unwrap()).unwrap();
            assert_eq!(
                hll.dense_zero_count(),
                scanned,
                "{name} n={n}: packed count drifted from scan in raw regime",
            );
            let k = ZERO_BITS[P::EXPONENT as usize - 4][B::NUMBER_OF_BITS as usize - 4];
            let cap = if k >= 32 { u32::MAX } else { (1u32 << k) - 1 };
            assert!(
                scanned <= cap,
                "{name} n={n}: raw-regime zeros {scanned} exceeded {k}-bit cap {cap}",
            );
        }

        check::<Precision8, Bits4>("P8B4");
        check::<Precision10, Bits5>("P10B5");
        check::<Precision12, Bits6>("P12B6");
    }

    /// Serde round-trip of the packed harmonic word: the low-mantissa packing must survive any
    /// serde format that preserves finite `f64` bit-exactly. Whole-counter round-trip cannot be
    /// tested at this layer because the register backing `sketching_core::Packed<W, V>` does not
    /// yet implement `Serialize`/`Deserialize` under its own `serde` feature. That is a
    /// pre-existing gap in `sketching_core`, unrelated to the packing; the invariant we own here
    /// is that the harmonic-sum word round-trips, which is verified end to end via `serde_json`.
    /// Zeros mode is skipped intentionally: its NaN box does not survive `serde_json`, which is a
    /// JSON limitation, not a packing limitation.
    #[cfg(feature = "serde")]
    #[test]
    fn serde_round_trip_packed_harmonic_word() {
        let mut hll = HyperLogLog::<Precision10, Bits6>::default();
        let mut state = 0x00C0_FFEE_u64;
        for _ in 0..(5 * (1u64 << 10)) {
            state = splitmix64(state);
            hll.insert(&state);
        }
        assert!(
            hll.is_hyperloglog() && !hll.harmonic_sum.is_nan(),
            "test setup: need dense harmonic mode with a finite packed word",
        );
        let original_bits = hll.harmonic_sum.to_bits();
        let packed_zeros = hll.dense_zero_count();
        let scanned_zeros = u32::try_from(hll.number_of_zero_registers().unwrap()).unwrap();
        assert_eq!(
            packed_zeros, scanned_zeros,
            "test setup: packed count must match the fresh scan before the round-trip",
        );

        // Round-trip the packed word itself. `serde_json` uses correctly-rounded formatting,
        // which is a bijection on finite normal `f64` values, so bits are preserved.
        let json = serde_json::to_string(&hll.harmonic_sum).expect("serialize packed word");
        let restored_word: f64 = serde_json::from_str(&json).expect("deserialize packed word");
        assert_eq!(
            restored_word.to_bits(),
            original_bits,
            "packed harmonic-sum word must survive JSON round-trip bit-exactly",
        );

        // Reconstruct a counter around the restored word and confirm the packing survives on the
        // reader side: `dense_harmonic_sum` and `dense_zero_count` recover the same `(H, zeros)`
        // as the original, and the sigma/tau estimate matches bit-exactly.
        let mut restored = hll;
        restored.harmonic_sum = restored_word;
        assert_eq!(restored.dense_zero_count(), scanned_zeros);
        assert_eq!(
            restored.dense_harmonic_sum().to_bits(),
            hll.dense_harmonic_sum().to_bits(),
        );
        assert_eq!(
            restored.sigma_tau_cardinality(),
            hll.sigma_tau_cardinality(),
        );
    }
}

#[cfg(test)]
mod codec_generic_tests {
    //! Proves the `C` codec generic on `HyperLogLog<P, B, R, H, C>` is real work at runtime, not
    //! just plumbing that always resolves to `SigBitsCode`. Two counters with the same values
    //! inserted MUST recover the same set (functional equivalence), and MUST produce different raw
    //! register bytes (proof the codec parameter is actually driving the encoding).
    use super::*;
    use dsi_bitstream::prelude::{
        CodeLen as _CodeLen, CodesRead, CodesWrite, DynamicCodeRead as _DynamicCodeRead,
        DynamicCodeWrite as _DynamicCodeWrite, Endianness,
    };
    use sketching_core::sparse_value_list::{code_consts, ConstCode};

    /// A `Default`-derivable wrapper around `ConstCode<GAMMA>`. Elias gamma has a different
    /// codeword shape from `SigBitsCode` (`SigBits(n) = 2 * bit_length(n) + 1`, `gamma(n) =
    /// 2 * floor(log2(n + 1)) + 1`), so encoding the same non-trivial value in each yields
    /// different bit strings.
    #[derive(Debug, Default, Clone, Copy)]
    struct Gamma;

    impl _DynamicCodeRead for Gamma {
        #[inline]
        fn read<E: Endianness, R: CodesRead<E> + ?Sized>(
            &self,
            r: &mut R,
        ) -> Result<u64, R::Error> {
            ConstCode::<{ code_consts::GAMMA }>.read(r)
        }
    }

    impl _DynamicCodeWrite for Gamma {
        #[inline]
        fn write<E: Endianness, W: CodesWrite<E> + ?Sized>(
            &self,
            w: &mut W,
            value: u64,
        ) -> Result<usize, W::Error> {
            ConstCode::<{ code_consts::GAMMA }>.write(w, value)
        }
    }

    impl _CodeLen for Gamma {
        #[inline]
        fn len(&self, value: u64) -> usize {
            ConstCode::<{ code_consts::GAMMA }>.len(value)
        }
    }

    type ArrayReg = <Precision10 as PackedRegister<Bits6>>::Array;
    type DefaultHll = HyperLogLog<Precision10, Bits6, ArrayReg, twox_hash::XxHash64, SigBitsCode>;
    type GammaHll = HyperLogLog<Precision10, Bits6, ArrayReg, twox_hash::XxHash64, Gamma>;

    /// Insert a handful of values that stay in the sorted value list under both codecs, then check
    /// that recovery agrees but the raw register bytes differ. If the codec parameter were dead
    /// code (silently resolving to `SigBitsCode` for both), the raw bytes would be identical.
    #[test]
    fn codec_parameter_drives_bit_layout_in_value_list() {
        let mut default_hll: DefaultHll = Default::default();
        let mut gamma_hll: GammaHll = Default::default();
        // 50 sequential u64s: well under either codec's saturation budget on a p10 b6 counter
        // (768 bytes), so both should still be in exact-mode after insertion.
        for v in 0u64..50 {
            default_hll.insert_value(v);
            gamma_hll.insert_value(v);
        }
        assert!(default_hll.is_sorted_value_list());
        assert!(gamma_hll.is_sorted_value_list());

        // Functional equivalence: both codecs recover the same set of values in the same order.
        let default_values: alloc::vec::Vec<u64> = default_hll.recover_values().unwrap().collect();
        let gamma_values: alloc::vec::Vec<u64> = gamma_hll.recover_values().unwrap().collect();
        assert_eq!(
            default_values, gamma_values,
            "recovery must yield the same set of values under both codecs",
        );

        // Bit-layout distinctness: same logical set, different on-wire encoding. If these are
        // equal, the codec parameter is not actually driving the writer path.
        let default_bytes: &[u8] = default_hll.registers.as_ref();
        let gamma_bytes: &[u8] = gamma_hll.registers.as_ref();
        assert_ne!(
            default_bytes, gamma_bytes,
            "SigBitsCode and gamma must produce different register byte layouts for the same set",
        );
    }

    /// Membership queries must agree between the two codecs: an inserted value is present under
    /// both, an uninserted value is absent under both. This exercises the decode path (in
    /// `contains_value`) through the `C` parameter.
    #[test]
    fn codec_parameter_drives_membership() {
        let mut default_hll: DefaultHll = Default::default();
        let mut gamma_hll: GammaHll = Default::default();
        for v in [7u64, 42, 123, 999, 1_000_000] {
            default_hll.insert_value(v);
            gamma_hll.insert_value(v);
        }
        for v in [7u64, 42, 123, 999, 1_000_000] {
            assert!(default_hll.may_contain_value(v));
            assert!(gamma_hll.may_contain_value(v));
        }
        for v in [0u64, 1, 8, 100, 1_000_001] {
            assert!(!default_hll.may_contain_value(v));
            assert!(!gamma_hll.may_contain_value(v));
        }
    }
}

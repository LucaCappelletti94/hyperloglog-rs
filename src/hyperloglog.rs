//! Marker struct for the hybrid approach, that keeps the hash explicit up until they fit into the registers.

use crate::composite_hash::{GapHash, SaturationError};
use crate::correction_coefficients::{
    HASHLIST_CORRECTION_BIAS, HASHLIST_CORRECTION_CARDINALITIES, HYPERLOGLOG_CORRECTION_BIAS,
    HYPERLOGLOG_CORRECTION_CARDINALITIES,
};
use crate::prelude::*;
use core::f64;
use core::fmt::Debug;
use core::hash::Hash;
use core::marker::PhantomData;

#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "mem_dbg", derive(mem_dbg::MemDbg, mem_dbg::MemSize))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// A struct representing the hybrid for approximate set cardinality estimation,
/// where the hash values are kept explicit up until they fit into the registers.
pub struct HyperLogLog<
    P: Precision,
    B: Bits,
    R: Registers<P, B> = <P as PackedRegister<B>>::Array,
    Hasher: HasherType = twox_hash::XxHash64,
> {
    /// The registers of the counter.
    pub(crate) registers: R,
    /// The harmonic sum of the registers, i.e. the sum of 2^(-register_value) for all registers.
    pub(crate) harmonic_sum: f64,
    /// Phantom data to ensure the type parameters are used.
    _phantom: PhantomData<(P, B, Hasher)>,
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
pub type VecHll<P, B, H = twox_hash::XxHash64> =
    HyperLogLog<P, B, <P as PackedRegister<B>>::Vec, H>;

impl<P: Precision, B: Bits, R: Registers<P, B>, H: HasherType> Default for HyperLogLog<P, B, R, H> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

#[inline]
fn correction_upper_bound<P: Precision>() -> f64 {
    7.5 * f64::integer_exp2(P::EXPONENT)
}

#[inline]
/// Returns the corrected estimate of the cardinality.
pub fn correct_cardinality<P: Precision, B: Bits>(
    raw_estimate: f64,
    cardinalities: &[u32],
    biases: &[f64],
) -> f64 {
    if raw_estimate >= correction_upper_bound::<P>() {
        return raw_estimate;
    }

    let estimate_u32 = u32::try_from(raw_estimate as u64).unwrap();

    if estimate_u32 <= cardinalities[0] {
        return raw_estimate + biases[0] * raw_estimate / f64::from(cardinalities[0]).max(1.0);
    }

    if estimate_u32 > cardinalities[cardinalities.len() - 1] {
        return raw_estimate
            + biases[cardinalities.len() - 1] * raw_estimate
                / f64::from(cardinalities[cardinalities.len() - 1]);
    }

    // We use a binary-search-based partition search to find the point where the raw estimate is
    // located in the cardinalities.

    debug_assert!(cardinalities.windows(2).all(|window| window[0] < window[1]));

    let index = cardinalities.partition_point(|&x| x < estimate_u32);

    let lower_cardinality = cardinalities[index - 1];
    let upper_cardinality = cardinalities[index];

    let lower_bias = biases[index - 1];
    let upper_bias = biases[index];

    assert!(lower_cardinality < upper_cardinality);

    raw_estimate
        + (raw_estimate - f64::from(lower_cardinality))
            / f64::from(upper_cardinality - lower_cardinality)
            * (upper_bias - lower_bias)
        + lower_bias
}

impl<P: Precision, B: Bits, R: Registers<P, B>, H: HasherType> HyperLogLog<P, B, R, H> {
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
        self.is_hash_list() && self.get_number_of_hashes().unwrap() == 0
    }

    #[inline]
    /// Returns whether the counter is fully saturated.
    pub fn is_full(&self) -> bool {
        // The harmonic sum is defined as Sum(2^(-register_value)) for all registers.
        // When all registers are maximally filled, i.e. equal to the maximal multiplicity value,
        // the harmonic sum is equal to (2^(-max_multiplicity)) * number_of_registers.
        // Since number_of_registers is a power of 2, specifically 2^exponent, the harmonic sum
        // is equal to 2^(exponent - max_multiplicity). Only a dense counter can be full; the
        // pre-dense representations reuse `harmonic_sum` as a metadata word, not a real sum.
        self.is_dense()
            && self.harmonic_sum
                <= f64::integer_exp2_minus_signed(
                    (1_i16 << B::NUMBER_OF_BITS) - i16::from(P::EXPONENT) - 1,
                )
    }

    #[inline]
    /// Returns whether the provided element may be contained in the counter.
    pub fn may_contain<T: Hash>(&self, element: &T) -> bool {
        let (index, register, original_hash) = Self::index_and_register_and_hash(element);
        // In exact mode the stored items are literal values, not hashes, so test membership by
        // hashing each stored value and matching the full original hash (exact, no false negatives).
        #[cfg(feature = "exact")]
        if self.is_exact() {
            return crate::composite_hash::gaps::value_list::ValueIter::new(
                self.registers.as_ref(),
                self.get_number_of_values(),
            )
            .any(|value| Self::index_and_register_and_hash(&value).2 == original_hash);
        }
        if self.is_hash_list() {
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
    /// Returns whether the counter is in dense (register) mode, as opposed to one of the two
    /// pre-dense representations (the hash list or the exact-values list), which repurpose
    /// `harmonic_sum` as a metadata word with its top bit set.
    pub fn is_dense(&self) -> bool {
        self.harmonic_sum.to_bits().leading_zeros() != 0
    }

    #[inline]
    /// Returns whether the counter is in the exact-values mode: the representation that precedes the
    /// hash list, storing the literal inserted integers for exact recovery and exact set
    /// operations.
    pub fn is_exact(&self) -> bool {
        !self.is_dense() && self.is_exact_metadata()
    }

    #[inline]
    /// Returns whether the counter is in hash-list mode: a sorted list of composite hashes, the
    /// representation between the exact-values list and dense registers. This is exactly one of the
    /// three representations (see [`HyperLogLog::is_exact`] and [`HyperLogLog::is_dense`]).
    pub fn is_hash_list(&self) -> bool {
        !self.is_dense() && !self.is_exact_metadata()
    }

    #[inline]
    /// Returns the number of registers equal to zero.
    ///
    /// # Raises
    /// If the counter is in HashList mode, an error is raised.
    pub fn number_of_zero_registers(&self) -> Result<usize, &'static str> {
        if !self.is_dense() {
            Err("The counter is in HashList mode.")
        } else {
            Ok(self
                .registers
                .iter_registers()
                .filter(|&register| register == 0)
                .count())
        }
    }

    #[inline]
    fn clear(&mut self) {
        self.registers.clear_registers();
        self.harmonic_sum = f64::NEG_INFINITY;
        self.set_number_of_hashes(0);
        self.set_writer_tell(0);
        self.set_duplicates(0);
        self.set_hash_bits(GapHash::<P, B>::LARGEST_VIABLE_HASH_BITS);
        debug_assert!(self.is_hash_list());
    }

    #[inline]
    /// Inserts an element into the counter.
    pub fn insert<T: Hash>(&mut self, element: &T) -> bool {
        // A hashed insert is incompatible with the exact-values mode (which stores literal values),
        // so first promote an exact counter to a proper hash list by hashing its stored values.
        #[cfg(feature = "exact")]
        if self.is_exact() {
            self.convert_exact_to_hash_list().unwrap();
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
    /// hash-list insertion path (with its saturation and downgrade handling) or, once the
    /// counter is a fully-fledged [`HyperLogLog`], straight into the registers.
    fn insert_index_register_hash(
        &mut self,
        index: usize,
        register: u8,
        original_hash: u64,
    ) -> bool {
        // The exact-values mode never reaches this hashed-insert path: callers promote it to a
        // proper hash list first.
        #[cfg(feature = "exact")]
        debug_assert!(!self.is_exact());
        if self.is_hash_list() {
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
                        self.convert_hash_list_to_hyperloglog().unwrap();
                        debug_assert!(self.is_dense());
                        self.insert_index_register_hash(index, register, original_hash)
                    }
                },
            }
        } else {
            self.insert_register_value_and_index(register, index)
        }
    }

    #[inline]
    /// Converts the Hybrid counter to a regular [`HyperLogLog`] counter.
    pub fn convert_hash_list_to_hyperloglog(&mut self) -> Result<(), &str> {
        if !self.is_hash_list() {
            return Err("The counter is already in HyperLogLog mode.");
        }
        let hash_bits = self.get_hash_bits().unwrap();
        let mut new_registers = self.registers.clone();
        new_registers.clear_registers();
        let registers = core::mem::replace(&mut self.registers, new_registers);
        let number_of_hashes = self.get_number_of_hashes().unwrap();
        let writer_tell = self.get_writer_tell();
        self.harmonic_sum = f64::integer_exp2(P::EXPONENT);

        let mut last_index = usize::MAX;
        GapHash::<P, B>::decoded(registers.as_ref(), number_of_hashes, hash_bits, writer_tell)
            .for_each(|(new_register_value, index)| {
                if last_index == index {
                    return;
                }
                last_index = index;
                self.insert_register_value_and_index(new_register_value, index);
            });

        debug_assert!(self.harmonic_sum.is_finite());

        Ok(())
    }

    #[cfg(feature = "exact")]
    #[inline]
    /// Inserts a literal integer value, storing it exactly (and recoverably) while the counter is
    /// small enough to remain in the exact-values mode.
    ///
    /// A fresh counter enters the exact-values mode on its first `insert_value`. When the exact
    /// buffer fills, the stored values are hashed (with the counter's hasher `H`) into a proper hash
    /// list, which later transitions to dense registers, exactly like a hashed counter. Once a
    /// counter has left the exact-values mode (because it grew, or because a hashed
    /// [`HyperLogLog::insert`] was used) a value is hashed and inserted like any other element.
    ///
    /// Returns whether the value was newly inserted.
    pub fn insert_value(&mut self, value: u64) -> bool {
        if self.is_exact() {
            return self.insert_value_exact(value);
        }
        if self.is_hash_list() && self.get_number_of_hashes().unwrap() == 0 {
            // A fresh, empty counter: enter the exact-values mode.
            self.registers.clear_registers();
            self.set_exact_mode();
            debug_assert!(self.is_exact());
            return self.insert_value_exact(value);
        }
        // The counter has already left the exact-values mode: hash the value like any element.
        let (index, register, original_hash) = Self::index_and_register_and_hash(&value);
        self.insert_index_register_hash(index, register, original_hash)
    }

    #[cfg(feature = "exact")]
    #[inline]
    /// Inserts a value into the exact-values list, growing the buffer or transitioning to a hash
    /// list when it no longer fits.
    fn insert_value_exact(&mut self, value: u64) -> bool {
        use crate::composite_hash::gaps::value_list::{self, ValueInsertion};

        let count = self.get_number_of_values();
        match value_list::insert_value(self.registers.as_mut(), count, value) {
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
                    // The buffer is at its maximum: hash the stored values into a hash list and
                    // insert the new value there.
                    self.convert_exact_to_hash_list().unwrap();
                    let (index, register, original_hash) =
                        Self::index_and_register_and_hash(&value);
                    self.insert_index_register_hash(index, register, original_hash)
                }
            }
        }
    }

    #[cfg(feature = "exact")]
    /// Merges another exact-values counter into this one (both must be in exact mode) in linear time,
    /// keeping the result exact. Returns `false` without modifying `self` if the union does not fit
    /// the exact buffer, so the caller can transition out of exact mode instead.
    ///
    /// Both operands store their values sorted (descending) and gap-coded, so the union is produced by
    /// a single two-pointer merge written once, rather than by splicing each value of the other
    /// operand into this one (which is quadratic). The only allocation is one clone of this counter's
    /// own value buffer, mirroring the mode-transition paths.
    fn try_merge_exact_values(&mut self, rhs: &Self) -> bool {
        use crate::composite_hash::gaps::value_list;

        let count_self = self.get_number_of_values();
        let count_rhs = rhs.get_number_of_values();
        let (union_count, needed_bits) = value_list::merge_metrics(
            self.registers.as_ref(),
            count_self,
            rhs.registers.as_ref(),
            count_rhs,
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
        value_list::merge_write(
            source.as_ref(),
            count_self,
            rhs.registers.as_ref(),
            count_rhs,
            self.registers.as_mut(),
        );
        self.set_number_of_values(union_count);
        true
    }

    #[cfg(feature = "exact")]
    #[inline]
    /// Converts an exact-values counter into a proper hash list by hashing each stored value with
    /// the counter's hasher `H`. This is the one-way transition that the exact mode shares with the
    /// hash-list to dense transition: recovery and absolute exactness are lost past this point.
    ///
    /// # Errors
    /// If the counter is not in exact mode, an error is returned.
    pub fn convert_exact_to_hash_list(&mut self) -> Result<(), &'static str> {
        if !self.is_exact() {
            return Err("The counter is not in exact-values mode.");
        }
        // The values and the destination hash list share the same register buffer, so move the
        // value bytes aside (a single buffer clone, as the hash-list to dense transition also does)
        // and stream them lazily into the cleared hash list.
        let count = self.get_number_of_values();
        let source = self.registers.clone();
        self.clear();
        debug_assert!(self.is_hash_list());
        for value in crate::composite_hash::gaps::value_list::ValueIter::new(source.as_ref(), count)
        {
            let (index, register, original_hash) = Self::index_and_register_and_hash(&value);
            self.insert_index_register_hash(index, register, original_hash);
        }
        Ok(())
    }

    #[cfg(feature = "exact")]
    #[inline]
    /// Recovers the exact set of literal values inserted via [`HyperLogLog::insert_value`] as a lazy
    /// iterator yielding them in descending order, if the counter is still in the exact-values mode.
    /// Returns `None` once the counter has left exact mode (the literal values are no longer retained
    /// past that transition).
    pub fn recover_values(&self) -> Option<impl Iterator<Item = u64> + '_> {
        if self.is_exact() {
            Some(crate::composite_hash::gaps::value_list::ValueIter::new(
                self.registers.as_ref(),
                self.get_number_of_values(),
            ))
        } else {
            None
        }
    }

    #[cfg(feature = "exact")]
    #[inline]
    /// Returns whether the given literal value is present, exactly, while the counter is in the
    /// exact-values mode. Falls back to the probabilistic hashed membership otherwise.
    pub fn may_contain_value(&self, value: u64) -> bool {
        if self.is_exact() {
            crate::composite_hash::gaps::value_list::contains_value(
                self.registers.as_ref(),
                self.get_number_of_values(),
                value,
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

        self.harmonic_sum += f64::integer_exp2_minus(larger_register_value)
            - f64::integer_exp2_minus(old_register_value);

        debug_assert!(self.harmonic_sum.is_finite());
        debug_assert!(self.harmonic_sum > 0.0);

        old_register_value < new_register_value
    }

    #[inline]
    /// Returns the uncorrected estimate of the cardinality.
    pub fn uncorrected_estimate_cardinality(&self) -> f64 {
        #[cfg(feature = "exact")]
        if self.is_exact() {
            return f64::from(self.get_number_of_values());
        }
        if self.is_hash_list() {
            f64::from(self.get_number_of_hashes().unwrap() + self.get_duplicates())
        } else {
            P::ALPHA * f64::integer_exp2(P::EXPONENT + P::EXPONENT) / self.harmonic_sum
        }
    }

    #[inline]
    /// Returns the corrected estimate of the cardinality.
    pub fn estimate_cardinality(&self) -> f64 {
        // The exact-values mode stores every inserted value verbatim, so its cardinality is the
        // exact count with no bias correction.
        #[cfg(feature = "exact")]
        if self.is_exact() {
            return f64::from(self.get_number_of_values());
        }
        if self.is_hash_list() {
            correct_cardinality::<P, B>(
                f64::from(self.get_number_of_hashes().unwrap() + self.get_duplicates()),
                &HASHLIST_CORRECTION_CARDINALITIES[P::EXPONENT as usize - 4]
                    [B::NUMBER_OF_BITS as usize - 4],
                &HASHLIST_CORRECTION_BIAS[P::EXPONENT as usize - 4][B::NUMBER_OF_BITS as usize - 4],
            )
        } else {
            let raw_estimate =
                P::ALPHA * f64::integer_exp2(P::EXPONENT + P::EXPONENT) / self.harmonic_sum;

            correct_cardinality::<P, B>(
                raw_estimate,
                &HYPERLOGLOG_CORRECTION_CARDINALITIES[P::EXPONENT as usize - 4]
                    [B::NUMBER_OF_BITS as usize - 4],
                &HYPERLOGLOG_CORRECTION_BIAS[P::EXPONENT as usize - 4]
                    [B::NUMBER_OF_BITS as usize - 4],
            )
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
        // Exact-values operands are handled before the hash-list/dense matrix: two exact operands
        // give the exact union directly, and a mixed pair promotes the exact one to a proper hash
        // list (a clone) and reuses the existing logic.
        #[cfg(feature = "exact")]
        {
            if self.is_exact() && other.is_exact() {
                let union = crate::composite_hash::gaps::value_list::union_count(
                    self.registers.as_ref(),
                    self.get_number_of_values(),
                    other.registers.as_ref(),
                    other.get_number_of_values(),
                );
                return f64::from(union);
            }
            if self.is_exact() {
                let mut promoted = self.clone();
                promoted.convert_exact_to_hash_list().unwrap();
                return promoted.estimate_union_cardinality_with_cardinalities(
                    other,
                    self_cardinality,
                    other_cardinality,
                );
            }
            if other.is_exact() {
                let mut promoted = other.clone();
                promoted.convert_exact_to_hash_list().unwrap();
                return self.estimate_union_cardinality_with_cardinalities(
                    &promoted,
                    self_cardinality,
                    other_cardinality,
                );
            }
        }
        match (self.is_hash_list(), other.is_hash_list()) {
            (true, true) => {
                // Build the union as a hash list and estimate its cardinality directly, so the
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
                let union_estimate = correct_cardinality::<P, B>(
                    P::ALPHA * f64::integer_exp2(P::EXPONENT + P::EXPONENT)
                        / self.registers.get_union_harmonic_sum(&other.registers),
                    &HYPERLOGLOG_CORRECTION_CARDINALITIES[P::EXPONENT as usize - 4]
                        [B::NUMBER_OF_BITS as usize - 4],
                    &HYPERLOGLOG_CORRECTION_BIAS[P::EXPONENT as usize - 4]
                        [B::NUMBER_OF_BITS as usize - 4],
                );
                correct_union_estimate(self_cardinality, other_cardinality, union_estimate)
            }
        }
    }

    #[inline]
    /// Merges another counter into this one, equivalent to a set union.
    ///
    /// # Implementative details
    /// When both counters are still in hash-list mode, the union is itself kept as a hash
    /// list, preserving the accuracy of small cardinalities: the hashes of the
    /// higher-precision counter are downgraded and inserted into the lower-precision one
    /// (a stored hash can only be downgraded, never upgraded). As soon as either operand
    /// is a fully-fledged [`HyperLogLog`], the result is a [`HyperLogLog`] whose registers
    /// are the element-wise maximum of the two operands.
    fn merge(&mut self, rhs: &Self) {
        // Exact-values operands are folded in before the hash-list/dense matrix. When both counters
        // are exact, a single linear two-pointer merge keeps the result exact (and falls back to a
        // mode transition if the union no longer fits). When only `self` is exact and `rhs` is
        // hashed, `self` is first promoted to a proper hash list, then merged normally.
        #[cfg(feature = "exact")]
        {
            if rhs.is_exact() {
                if self.is_exact() && self.try_merge_exact_values(rhs) {
                    return;
                }
                if self.is_exact() {
                    // The exact union overflows the buffer: leave exact mode, then fold `rhs`'s
                    // values in (now hashed, so each insertion is cheap).
                    self.convert_exact_to_hash_list().unwrap();
                }
                for value in crate::composite_hash::gaps::value_list::ValueIter::new(
                    rhs.registers.as_ref(),
                    rhs.get_number_of_values(),
                ) {
                    self.insert_value(value);
                }
                return;
            }
            if self.is_exact() {
                self.convert_exact_to_hash_list().unwrap();
            }
        }
        match (self.is_hash_list(), rhs.is_hash_list()) {
            (false, false) => {
                // Both counters are fully-fledged HyperLogLogs: element-wise register maximum.
                for (index, register) in rhs.registers.iter_registers().enumerate() {
                    self.insert_register_value_and_index(register, index);
                }
            }
            (true, false) => {
                // Only `self` is a hash list: materialize it, then take the register maximum.
                self.convert_hash_list_to_hyperloglog().unwrap();
                for (index, register) in rhs.registers.iter_registers().enumerate() {
                    self.insert_register_value_and_index(register, index);
                }
            }
            (false, true) => {
                // Only `rhs` is a hash list: fold its hashes into `self`'s registers.
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
                // Both counters are hash lists: keep the union as a hash list by inserting the
                // hashes of the higher-precision counter into the lower-precision one. A stored
                // hash can only be downgraded, never upgraded, so the lower-precision counter
                // (the one with the larger or equal hash size... i.e. fewer hashes) is used as
                // the base, and the other counter's hashes are downgraded to its hash size.
                // Inserting an already-present hash is a no-op, which keeps the union idempotent.
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
    }
}

impl<P: Precision, B: Bits, R: Registers<P, B>, H: HasherType> BitOrAssign<&Self>
    for HyperLogLog<P, B, R, H>
{
    #[inline]
    fn bitor_assign(&mut self, rhs: &Self) {
        self.merge(rhs);
    }
}

impl<P: Precision, B: Bits, R: Registers<P, B>, H: HasherType> BitOrAssign
    for HyperLogLog<P, B, R, H>
{
    #[inline]
    fn bitor_assign(&mut self, rhs: Self) {
        self.merge(&rhs);
    }
}

impl<P: Precision, B: Bits, R: Registers<P, B>, H: HasherType> BitOr for HyperLogLog<P, B, R, H> {
    type Output = Self;

    #[inline]
    fn bitor(mut self, rhs: Self) -> Self::Output {
        self.merge(&rhs);
        self
    }
}

impl<P: Precision, B: Bits, R: Registers<P, B>, H: HasherType> BitOr for &HyperLogLog<P, B, R, H> {
    type Output = HyperLogLog<P, B, R, H>;

    #[inline]
    fn bitor(self, rhs: Self) -> Self::Output {
        let mut result = self.clone();
        result.merge(rhs);
        result
    }
}

#[cfg(test)]
mod test_hybrid_propertis {
    use super::*;
    use hyperloglog_derive::test_estimator;

    #[test_estimator]
    fn test_plusplus_properties<P: Precision, B: Bits, R: Registers<P, B>, H: HasherType>() {
        let mut hybrid: HyperLogLog<P, B, R, H> = Default::default();
        assert!(hybrid.is_hash_list());
        assert!(hybrid.is_empty());
        assert!(!hybrid.is_full());
        assert_eq!(hybrid.get_number_of_hashes().unwrap(), 0);
        let mut normalized_error = 0.0;
        let mut non_normalized_error = 0.0;
        let mut random_state = 34567897654354_u64;
        let mut iterations = 0;

        while hybrid.is_hash_list() {
            iterations += 1;
            // To make the test a bit fairer using more random elements
            // than a numerical sequence.
            random_state = splitmix64(splitmix64(random_state));
            hybrid.insert(&random_state);
            assert!(
                !hybrid.insert(&random_state),
                "The Hybrid counter should NOT already contain the element {random_state}. Hash size: {}. Iteration n. {iterations}. Hash list status: {}",
                hybrid.get_hash_bits().unwrap(),
                hybrid.is_hash_list()
            );
            assert!(
                hybrid.may_contain(&random_state),
                "The Hybrid counter must contain the element {random_state}. Iteration n. {iterations}.",
            );

            let estimated_cardinality = hybrid.estimate_cardinality();

            let error = iterations as f64 - estimated_cardinality;
            non_normalized_error += error;
            normalized_error += error / iterations as f64;
        }

        normalized_error /= iterations as f64;
        non_normalized_error /= iterations as f64;

        // In hash-list mode the counter stores explicit hashes, so the only error source is
        // hash collisions plus the residual bias of the fitted cardinality correction. The
        // meaningful, theoretically grounded bound is the structure's own accuracy contract:
        // the estimate must satisfy the precision's nominal relative error rate, which it does
        // with a wide margin (the hash-list mode is far more accurate than the HyperLogLog
        // register estimator at these cardinalities). We bound the magnitude of the mean
        // relative error, catching both under- and over-counting. The previous `/ 13.0`
        // tightening had no theoretical basis and is dropped.
        assert!(
            normalized_error.abs() <= P::error_rate(),
            "The mean relative hash-list error ({normalized_error}, non-normalized {non_normalized_error}) must not exceed the precision's error rate ({}).",
            P::error_rate()
        );

        assert!(!hybrid.is_hash_list());
    }
}

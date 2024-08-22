//! Marker struct for the hybrid approach, that keeps the hash explicit up until they fit into the registers.

use crate::prelude::*;
use core::cmp::Ordering;
use core::fmt::Debug;
use core::hash::Hash;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "mem_dbg", derive(mem_dbg::MemDbg, mem_dbg::MemSize))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// A struct representing the hybrid for approximate set cardinality estimation,
/// where the hash values are kept explicit up until they fit into the registers.
pub struct Hybrid<H> {
    /// The inner counter.
    inner: H,
}

impl<H: Hybridazable> Default for Hybrid<H>
where
    H: Default,
{
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<H: SetProperties + Hybridazable> SetProperties for Hybrid<H> {
    #[inline]
    fn is_empty(&self) -> bool {
        if self.is_hybrid() {
            self.inner.get_number_of_zero_registers().is_zero()
        } else {
            self.inner.is_empty()
        }
    }

    #[inline]
    fn is_full(&self) -> bool {
        if self.is_hybrid() {
            self.will_dehybridize_upon_insert()
        } else {
            self.inner.is_full()
        }
    }
}

impl<T: Hash, H: ApproximatedSet<T> + Hybridazable> ApproximatedSet<T> for Hybrid<H> {
    #[inline]
    fn may_contain(&self, element: &T) -> bool {
        if self.is_hybrid() {
            let hash_bytes = self.hash_bytes();
            assert!(hash_bytes >= Self::smallest_viable_hash());
            match hash_bytes {
                1 => self.find_sorted::<T, u8>(element),
                2 => self.find_sorted::<T, u16>(element),
                3 => self.find_sorted::<T, u24>(element),
                4 => self.find_sorted::<T, u32>(element),
                _ => {
                    unreachable!();
                }
            }
        } else {
            self.inner.may_contain(element)
        }
    }
}

impl<H: MutableSet + Hybridazable> MutableSet for Hybrid<H> {
    #[inline]
    fn clear(&mut self) {
        self.inner.registers_mut().clear_registers();
        *self.inner.number_of_zero_registers_mut() =
            <H::Precision as Precision>::NumberOfRegisters::ZERO;
        *self.inner.harmonic_sum_mut() = f64::NEG_INFINITY;
        encode_harmonic_flag(self.inner.harmonic_sum_mut(), 8);
    }
}

impl<T: Hash, H: ExtendableApproximatedSet<T> + Hybridazable> ExtendableApproximatedSet<T>
    for Hybrid<H>
{
    #[inline]
    fn insert(&mut self, element: &T) -> bool {
        if self.is_hybrid() {
            if self.will_dehybridize_upon_insert() {
                self.dehybridize();
                debug_assert!(!self.is_hybrid());
                self.insert(element)
            } else if self.will_downgrade_upon_insert() {
                self.downgrade();
                debug_assert!(self.is_hybrid());
                debug_assert!(!self.will_downgrade_upon_insert());
                self.insert(element)
            } else {
                let hash_bytes = self.hash_bytes();
                assert!(hash_bytes >= Self::smallest_viable_hash());
                match hash_bytes {
                    1 => self.insert_value::<u8, T>(element),
                    2 => self.insert_value::<u16, T>(element),
                    3 => self.insert_value::<u24, T>(element),
                    4 => self.insert_value::<u32, T>(element),
                    _ => {
                        unreachable!();
                    }
                }
            }
        } else {
            self.inner.insert(element)
        }
    }
}

#[inline]
/// Returns the number of unique values from two sorted iterators.
///
/// # Implementative details
/// The sets we are considering are the union of the two sorted iterators
/// of Hybrid counters' hashes. The largest possible number of unique values
/// in each iterator is the number of words in a 2**18 counter, with the bit
/// size set to 8 (used primarely to benefit from the SIMD instructions).
/// As such 8 * 2**18 = 2**21, divided by the number of bits in a u64, we get
/// 2**21 / 64 = 2**15 unique values. The number of unique values in the union
/// of the two sets is at most the sum of the number of unique values in each set,
/// so at most 2**16 unique values. We can thus use a u32 to represent the number
/// of unique values.
///
/// # Panics
/// Panics if the number of unique values is greater than 2**32.
pub fn unique_count_from_sorted_iterators<
    T: Ord,
    I: ExactSizeIterator<Item = T>,
    J: ExactSizeIterator<Item = T>,
>(
    mut left: I,
    mut right: J,
) -> u32 {
    let mut intersection = u32::ZERO;
    let left_length = u32::try_from(left.len()).unwrap();
    let right_length = u32::try_from(right.len()).unwrap();
    let mut maybe_left_value = left.next();
    let mut maybe_right_value = right.next();
    while let (Some(left_value), Some(right_value)) =
        (maybe_left_value.as_ref(), maybe_right_value.as_ref())
    {
        let cmp = left_value.cmp(right_value);

        intersection += u32::from(cmp == Ordering::Equal);

        if cmp == Ordering::Equal || cmp == Ordering::Less {
            maybe_left_value = left.next();
        }
        if cmp == Ordering::Equal || cmp == Ordering::Greater {
            maybe_right_value = right.next();
        }
    }

    left_length + right_length - intersection
}

#[inline]
/// Returns the union estimation from a sorted iterator and a counter.
///
/// # Implementative details
/// The provided iterator is expected to be sorted in ascending order,
/// in such a way that hash values that point to the same index are contiguos,
/// and ordered by value of the register as well.
pub fn union_estimation_from_sorted_iterator_and_counter<
    I: ExactSizeIterator<Item = (u8, usize)> + DoubleEndedIterator,
    H: HyperLogLog + Correction,
>(
    iterator: I,
    counter: &H,
    left_cardinality: f64,
    right_cardinality: f64,
) -> f64 {
    let mut number_of_zeros = counter.get_number_of_zero_registers();
    let mut harmonic_sum = counter.harmonic_sum();
    // We set the previous index to the NUMBER OF REGISTERS, which is a value higher
    // than the maximal possible index, so that the first value is always considered
    // as a new value.
    let mut previous_index = usize::MAX;

    for (left_register_value, index) in iterator.rev() {
        debug_assert!(
            index <= previous_index,
            "The index must be less than or equal to the previous index, but got {index} and {previous_index}",
        );

        // If the index is the same as the previous index, we skip the value
        // as the register value is necessarily less or equal to the previous one.
        if index == previous_index {
            continue;
        }

        // We update the previous index.
        previous_index = index;
        // Otherwise, we update the number of zeros and the harmonic sum.
        let right_register_value = counter.get_register(index);

        if left_register_value <= right_register_value {
            continue;
        }

        // If the right register value is a zero, we are surely now removing
        // it because the left register value cannot be a zero.
        number_of_zeros -= u32::from(right_register_value == 0);
        harmonic_sum += f64::integer_exp2_minus(left_register_value)
            - f64::integer_exp2_minus(right_register_value);
    }

    correct_union_estimate(
        left_cardinality,
        right_cardinality,
        H::correction(harmonic_sum, number_of_zeros),
    )
}

/// Trait for a struct that can be used in the hybrid approach.
pub trait Hybridazable:
    HyperLogLog<Registers = <Self as Hybridazable>::Hashes> + Correction
{
    /// Registers type.
    type Hashes: Registers<Self::Precision, Self::Bits>;

    /// Returns the capacity of the counter.
    fn registers_mut(&mut self) -> &mut Self::Hashes;

    /// Returns a mutable reference to the number of zeros.
    fn number_of_zero_registers_mut(
        &mut self,
    ) -> &mut <Self::Precision as Precision>::NumberOfRegisters;

    /// Returns a mutable reference to the harmonic sum.
    fn harmonic_sum_mut(&mut self) -> &mut f64;
}

#[cfg(feature = "std")]
impl<H: Named> Named for Hybrid<H> {
    #[inline]
    fn name(&self) -> String {
        format!("H-{}", self.inner.name())
    }
}

#[allow(unsafe_code)]
#[expect(
    clippy::transmute_ptr_to_ptr,
    reason = "We are transmuting a mutable reference to a mutable reference, which is safe."
)]
fn encode_harmonic_flag(harmonic_sum_as_flat: &mut f64, target_hash: u8) {
    let harmonic_sum_as_u64: &mut u64 = unsafe { core::mem::transmute(harmonic_sum_as_flat) };
    // We clear the bits that are used to store the number of entries.
    *harmonic_sum_as_u64 &= !((1 << (8 + 1)) - 1);

    // Then, we set the flag associated with this specific Composite Hash.
    *harmonic_sum_as_u64 |= 1 << target_hash;
}

fn decode_harmonic_flag(harmonic_sum_as_flat: f64) -> u8 {
    // We use a trailing zeros operation to determine the number of entries.
    u8::try_from(harmonic_sum_as_flat.to_bits().trailing_zeros()).unwrap()
}

#[cfg(test)]
mod test_encode_decode_harmonic_flag {
    use super::*;

    #[test]
    fn test_encode_decode_harmonic_flag() {
        // The harmonic flag is initialized to minus infinity.
        let mut harmonic_sum = f64::NEG_INFINITY;
        encode_harmonic_flag(&mut harmonic_sum, 1);
        assert_eq!(decode_harmonic_flag(harmonic_sum), 1);
        encode_harmonic_flag(&mut harmonic_sum, 2);
        assert_eq!(decode_harmonic_flag(harmonic_sum), 2);
        encode_harmonic_flag(&mut harmonic_sum, 3);
        assert_eq!(decode_harmonic_flag(harmonic_sum), 3);
        encode_harmonic_flag(&mut harmonic_sum, 4);
        assert_eq!(decode_harmonic_flag(harmonic_sum), 4);

        // We check that the harmonic sum has still a leading number of zeros
        // equal to zero, as we have initialized it to minus infinity and we
        // should not have touched those bits.
        assert_eq!(harmonic_sum.to_bits().leading_zeros(), 0);
    }
}

#[inline]
/// Shifts the provided slice to the right in little endian and returns the number of duplicates removed.
fn shift_right_little_endian(slice: &mut [u8], slice_size: usize, shift_size: usize, len: usize) {
    debug_assert!(
        len * slice_size <= slice.len(),
        "The slice len ({}) must be greater or equal to the product of the slice size ({slice_size}) and the number of elements ({len})",
        slice.len(),
    );
    debug_assert!(shift_size < slice_size);
    debug_assert!(slice_size > 1);
    debug_assert!(shift_size > 0);

    for i in 0..len {
        slice.copy_within(
            (i * slice_size + shift_size)..i * slice_size + slice_size,
            i * (slice_size - shift_size),
        );
    }
}

#[cfg(test)]
mod test_shift_right_little_endian {
    use super::*;

    #[test]
    fn test_shift_right_little_endian() {
        let mut slice = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        shift_right_little_endian(&mut slice, 3, 1, 3);
        assert_eq!(slice, [2, 3, 5, 6, 8, 9, 7, 8, 9, 10]);
        shift_right_little_endian(&mut slice, 2, 1, 3);
        assert_eq!(slice, [3, 6, 9, 6, 8, 9, 7, 8, 9, 10]);
    }

    #[test]
    /// When shifting the slice by one, it may happen that two
    /// slices become identical. In such cases, we need to remove
    /// the duplicates.
    fn test_shift_right_little_endian_with_duplicates() {
        let mut slice = [1, 2, 3, 9, 2, 3, 4, 100, 200, 7, 100, 200];
        shift_right_little_endian(&mut slice, 3, 1, 3);
        assert_eq!(slice, [2, 3, 2, 3, 100, 200, 4, 100, 200, 7, 100, 200]);
        shift_right_little_endian(&mut slice, 2, 1, 3);
        assert_eq!(slice, [3, 3, 200, 3, 100, 200, 4, 100, 200, 7, 100, 200]);
    }

    #[test]
    fn test_shift_singleton_move() {
        let mut slice = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        shift_right_little_endian(&mut slice, 10, 1, 1);
        assert_eq!(slice, [2, 3, 4, 5, 6, 7, 8, 9, 10, 10]);
        shift_right_little_endian(&mut slice, 9, 1, 1);
        assert_eq!(slice, [3, 4, 5, 6, 7, 8, 9, 10, 10, 10]);
    }
}

impl<H: Hybridazable> Hybrid<H> {
    #[inline]
    fn new() -> Self {
        let mut inner = H::default();

        // We set the number of zeros to zero.
        *inner.number_of_zero_registers_mut() =
            <H::Precision as Precision>::NumberOfRegisters::ZERO;

        // We set the harmonic sum to negative infinity.
        // 0xFFF0000000000000
        *inner.harmonic_sum_mut() = f64::NEG_INFINITY;

        // And then we apply to it the mask of the highest Composite Hash,
        // i.e. u32. Since the total number of Composite Hash we have is 4,
        // for this one we use the number 8, and we apply the mask to it.
        encode_harmonic_flag(inner.harmonic_sum_mut(), 4);

        Self { inner }
    }

    #[inline]
    /// Returns whether the counter is in hybrid mode.
    fn is_hybrid(&self) -> bool {
        self.inner.harmonic_sum().to_bits().leading_zeros() == 0
    }

    #[inline]
    /// Returns the maximum number of hashes that can be stored in the counter.
    fn maximal_number_of_hashes() -> usize {
        H::Registers::bitsize() / usize::from(Self::smallest_viable_hash() * 8)
    }

    const fn smallest_viable_hash() -> u8 {
        if H::Precision::EXPONENT == 4 && H::Bits::NUMBER_OF_BITS == 4 {
            return 1;
        }

        if H::Precision::EXPONENT < 9
            || H::Precision::EXPONENT == 9 && H::Bits::NUMBER_OF_BITS < 6
            || H::Precision::EXPONENT == 10 && H::Bits::NUMBER_OF_BITS < 5
        {
            return 2;
        }

        if H::Precision::EXPONENT < 15
            || H::Precision::EXPONENT == 15 && H::Bits::NUMBER_OF_BITS < 6
            || H::Precision::EXPONENT == 16 && H::Bits::NUMBER_OF_BITS < 5
        {
            return 3;
        }

        4
    }

    #[inline]
    fn hash_bytes(&self) -> u8 {
        debug_assert!(self.is_hybrid());
        let hash_bytes = decode_harmonic_flag(self.inner.harmonic_sum());
        debug_assert!(
            hash_bytes >= Self::smallest_viable_hash(),
            "The number of bytes used for the hash ({hash_bytes}) must be at least equal to the smallest viable hash ({})",
            Self::smallest_viable_hash()
        );
        hash_bytes
    }

    #[inline]
    fn find_sorted<T: Hash, W: VariableWord>(&self, element: &T) -> bool
    where
        H::Registers: VariableWords<W>,
        W::Word: TryFrom<u64>,
        <W::Word as TryFrom<u64>>::Error: Debug,
    {
        let encoded = self.to_encoded_hash::<T>(element);
        debug_assert!(encoded <= W::MASK);
        let number_of_hashes = self.inner.get_number_of_zero_registers().to_usize();
        let word_encoded = <W::Word as TryFrom<u64>>::try_from(encoded).unwrap();
        <H::Hashes as VariableWords<W>>::find_sorted_with_len(
            self.inner.registers(),
            word_encoded,
            number_of_hashes,
        )
    }

    #[inline]
    /// Returns the next largest hash that can be used.
    ///
    /// # Implementation details
    /// The next largest hash is the first hash that allows the underlying
    /// registers vector to store the current number of hash (in the size of
    /// the target hash) and the next hash.
    fn downgrade_maximal_hash_bytes(&self) -> u8 {
        let number_of_hash = self.inner.get_number_of_zero_registers().to_usize();
        let current_hash = decode_harmonic_flag(self.inner.harmonic_sum());
        let smallest_viable_hash = Self::smallest_viable_hash();
        debug_assert!(
            current_hash > smallest_viable_hash,
            "The current hash ({current_hash}) must be at least equal to the smallest viable hash ({smallest_viable_hash})",
        );

        for i in (smallest_viable_hash..current_hash).rev() {
            if (number_of_hash + 1) * usize::from(i * 8) <= H::Registers::bitsize() {
                return i;
            }
        }

        unreachable!()
    }

    #[inline]
    /// Returns the current hash capacity.
    fn current_hash_capacity(&self) -> usize {
        H::Registers::bitsize() / usize::from(self.hash_bytes() * 8)
    }

    #[inline]
    /// Returns whether the hasher will have to be dehybridized at the next insert.
    fn will_dehybridize_upon_insert(&self) -> bool {
        debug_assert!(self.is_hybrid());
        self.inner.get_number_of_zero_registers().to_usize() == Self::maximal_number_of_hashes()
    }

    #[inline]
    /// Returns whether the hasher will have to downgrade the hash at the next insert.
    fn will_downgrade_upon_insert(&self) -> bool {
        debug_assert!(self.is_hybrid());
        self.inner.get_number_of_zero_registers().to_usize() == self.current_hash_capacity()
    }

    #[inline]
    /// Converts the Hybrid counter to a regular [`HyperLogLog`] counter.
    fn dehybridize_with_word<W: VariableWord>(&mut self)
    where
        H::Hashes: VariableWords<W>,
    {
        debug_assert!(self.is_hybrid());
        debug_assert!(self.will_dehybridize_upon_insert());
        debug_assert_eq!(
            self.current_hash_capacity(),
            Self::maximal_number_of_hashes()
        );
        debug_assert_eq!(self.hash_bytes(), Self::smallest_viable_hash());

        let mut new_counter: H = H::default();

        <H::Hashes as VariableWords<W>>::iter_variable_words(
            self.inner.registers(),
            Self::maximal_number_of_hashes(),
        )
        .for_each(|hash: W::Word| {
            let (register, index) = self.decode(hash.into());
            new_counter.insert_register_value_and_index(register, index);
        });

        *self = Self { inner: new_counter };
    }

    #[inline]
    /// Converts the Hybrid counter to a regular [`HyperLogLog`] counter.
    fn dehybridize(&mut self) {
        let hash_bytes = self.hash_bytes();
        assert!(hash_bytes >= Self::smallest_viable_hash());
        match hash_bytes {
            1 => self.dehybridize_with_word::<u8>(),
            2 => self.dehybridize_with_word::<u16>(),
            3 => self.dehybridize_with_word::<u24>(),
            4 => self.dehybridize_with_word::<u32>(),
            _ => {
                unreachable!();
            }
        }
    }

    #[inline]
    #[must_use]
    /// Encode the hash from the provided register value, index and the original unsplitted hash.
    ///
    /// # Arguments
    /// * `register` - The register value to be encoded.
    /// * `hash` - The original hash to be encoded.
    ///
    /// # Implementation
    /// The hash we receive is expected to be in the following form:
    ///
    /// ```text
    /// | bits used for the leading zeros count | potentially unused bits | bits used for the index |
    /// ```
    ///
    /// We need to ensure that the higher bits are the bits of the index, as we will
    /// sort the hashes and the index needs to be the primary sorting key. Next, we
    /// want to sort by the number of leading zeros, followed by any eventual unused bits.
    /// The resulting hash therefore, will be in the following form:
    ///
    /// ```text
    /// | bits used for the index | number of leading zeros | potentially unused bits |
    /// ```
    fn encode(&self, index: usize, register: u8, hash: u64) -> u64 {
        debug_assert!(register > 0);
        debug_assert!(index < 1 << H::Precision::EXPONENT);
        let hash_bits = self.hash_bytes() * 8;

        let offset = hash_bits - H::Bits::NUMBER_OF_BITS - H::Precision::EXPONENT;
        let mask = (1 << offset) - 1;

        // We remove the portion used for the index and apply the padding mask,
        // which ensures that now only the bits used for the padding (if any) are kept.
        let mut hash = (hash >> H::Precision::EXPONENT) & mask;

        debug_assert!(
            hash.leading_zeros() >= u32::from(64 - hash_bits + H::Precision::EXPONENT + H::Bits::NUMBER_OF_BITS),
            concat!(
                "Since the hash starts from a u64, and we are constructing a W::Word, ",
                "it must have at least 64 - W::NUMBER_OF_BITS + H::Precision::EXPONENT + H::Bits::NUMBER_OF_BITS leading zeros."
            )
        );

        // Next, we place the index in the rightmost bits of the hash.
        hash |= (index as u64) << (hash_bits - H::Precision::EXPONENT);

        // Next, we place the register in the rightmost bits of the hash, minus the bits used for the index.
        hash |= u64::from(register) << offset;

        // The resulting hash, since it starts from a u64, and we are constructing a W::Word,
        // must have at least 64 - hash_bits leading zeros.
        debug_assert!(
            hash.leading_zeros() >= u32::from(64 - hash_bits),
            "The hash we have constructed must have at least 64 - {hash_bits} leading zeros."
        );

        hash
    }

    #[must_use]
    #[inline]
    /// Decode the hash into the register value and index.
    fn decode(&self, hash: u64) -> (u8, usize) {
        let hash_bits = self.hash_bytes() * 8;
        // We extract the index from the rightmost bits of the hash.
        let index = usize::try_from(hash >> (hash_bits - H::Precision::EXPONENT)).unwrap();
        // Next, we extract the register from the rightmost bits of the hash, minus the bits used for the index.
        let register = u8::try_from(
            (hash >> (hash_bits - H::Bits::NUMBER_OF_BITS - H::Precision::EXPONENT))
                & H::Bits::MASK,
        )
        .unwrap();

        (register, index)
    }

    fn to_encoded_hash<T: Hash>(&self, value: &T) -> u64 {
        let (index, register, hash) = H::hash_and_register_and_index(value);
        let encoded = self.encode(index, register, hash);
        debug_assert_eq!(self.decode(encoded), (register, index));
        encoded
    }

    #[inline]
    fn insert_value<W: VariableWord, T: Hash>(&mut self, value: &T) -> bool
    where
        H::Hashes: VariableWords<W>,
        W::Word: TryFrom<u64>,
        <W::Word as TryFrom<u64>>::Error: Debug,
    {
        let encoded = self.to_encoded_hash::<T>(value);
        debug_assert!(encoded <= W::MASK);
        let encoded_word = <W::Word as TryFrom<u64>>::try_from(encoded).unwrap();
        let number_of_hashes = self.inner.get_number_of_zero_registers().to_usize();
        let inserted = self
            .inner
            .registers_mut()
            .sorted_insert_with_len(encoded_word, number_of_hashes);

        *self.inner.number_of_zero_registers_mut() +=
            <H::Precision as Precision>::NumberOfRegisters::from(inserted);
        inserted
    }

    fn unique_count_from_iterators<L: VariableWord, R: Ord + VariableWord>(
        &self,
        other: &Self,
    ) -> f64
    where
        H::Hashes: VariableWords<L> + VariableWords<R>,
        L::Word: TryInto<R::Word>,
        <L::Word as TryInto<R::Word>>::Error: Debug,
    {
        debug_assert!(self.is_hybrid());
        debug_assert!(other.is_hybrid());
        debug_assert!(self.hash_bytes() >= other.hash_bytes());
        let self_number_of_hashes = self.inner.get_number_of_zero_registers().to_usize();
        let other_number_of_hashes = other.inner.get_number_of_zero_registers().to_usize();
        let shift = (self.hash_bytes() - other.hash_bytes()) * 8;
        f64::from(unique_count_from_sorted_iterators(
            <H::Hashes as VariableWords<L>>::iter_variable_words(
                self.inner.registers(),
                self_number_of_hashes,
            )
            .map(|hash| <L::Word as TryInto<R::Word>>::try_into(hash >> shift).unwrap()),
            <H::Hashes as VariableWords<R>>::iter_variable_words(
                other.inner.registers(),
                other_number_of_hashes,
            ),
        ))
    }

    fn mixed_union<W>(&self, other: &Self, left_cardinality: f64, right_cardinality: f64) -> f64
    where
        H::Hashes: VariableWords<W>,
        W: VariableWord,
    {
        debug_assert!(self.is_hybrid());
        debug_assert!(!other.is_hybrid());
        let number_of_hashes = self.inner.get_number_of_zero_registers().to_usize();
        union_estimation_from_sorted_iterator_and_counter(
            <H::Hashes as VariableWords<W>>::iter_variable_words(
                self.inner.registers(),
                number_of_hashes,
            )
            .map(|hash| self.decode(hash.into())),
            &other.inner,
            left_cardinality,
            right_cardinality,
        )
    }

    #[inline]
    /// Downgrades the Hybrid hashes one level.
    fn downgrade(&mut self) {
        debug_assert!(self.is_hybrid());
        debug_assert!(self.will_downgrade_upon_insert());

        let number_of_hashes = self.inner.get_number_of_zero_registers().to_usize();
        let current_hash = usize::from(self.hash_bytes());
        let target_hash = self.downgrade_maximal_hash_bytes();
        let slice = self.inner.registers_mut().as_mut();

        let slice_to_update = &mut slice[..number_of_hashes * current_hash];

        shift_right_little_endian(
            slice_to_update,
            current_hash,
            current_hash - usize::from(target_hash),
            number_of_hashes,
        );

        encode_harmonic_flag(self.inner.harmonic_sum_mut(), target_hash);
        debug_assert_eq!(self.hash_bytes(), target_hash);
    }
}

impl<H: Clone + Correction + Estimator<f64> + Hybridazable + Default> Estimator<f64> for Hybrid<H> {
    #[inline]
    fn estimate_cardinality(&self) -> f64 {
        if self.is_hybrid() {
            f64::from(self.inner.get_number_of_zero_registers())
        } else {
            self.inner.estimate_cardinality()
        }
    }

    #[inline]
    fn is_union_estimate_non_deterministic(&self, other: &Self) -> bool {
        !(self.is_hybrid() && other.is_hybrid())
            && self.inner.is_union_estimate_non_deterministic(&other.inner)
    }

    #[inline]
    fn estimate_union_cardinality_with_cardinalities(
        &self,
        other: &Self,
        self_cardinality: f64,
        other_cardinality: f64,
    ) -> f64 {
        match (self.is_hybrid(), other.is_hybrid()) {
            (true, true) => {
                let self_hash = self.hash_bytes();
                let other_hash = other.hash_bytes();
                assert!(self_hash >= Self::smallest_viable_hash());
                assert!(other_hash >= Self::smallest_viable_hash());

                let union_cardinality = match (self_hash, other_hash) {
                    (1, 1) => self.unique_count_from_iterators::<u8, u8>(other),
                    (2, 2) => self.unique_count_from_iterators::<u16, u16>(other),
                    (3, 3) => self.unique_count_from_iterators::<u24, u24>(other),
                    (4, 4) => self.unique_count_from_iterators::<u32, u32>(other),
                    (a, b) if a < b => other.estimate_union_cardinality_with_cardinalities(
                        self,
                        other_cardinality,
                        self_cardinality,
                    ),
                    (2, 1) => self.unique_count_from_iterators::<u16, u8>(other),
                    (3, 1) => self.unique_count_from_iterators::<u24, u8>(other),
                    (3, 2) => self.unique_count_from_iterators::<u24, u16>(other),
                    (4, 1) => self.unique_count_from_iterators::<u32, u8>(other),
                    (4, 2) => self.unique_count_from_iterators::<u32, u16>(other),
                    (4, 3) => self.unique_count_from_iterators::<u32, u24>(other),
                    _ => {
                        unreachable!();
                    }
                };
                correct_union_estimate(self_cardinality, other_cardinality, union_cardinality)
            }
            (true, false) => {
                let self_hash = self.hash_bytes();
                assert!(self_hash >= Self::smallest_viable_hash());
                match self_hash {
                    1 => self.mixed_union::<u8>(other, self_cardinality, other_cardinality),
                    2 => self.mixed_union::<u16>(other, self_cardinality, other_cardinality),
                    3 => self.mixed_union::<u24>(other, self_cardinality, other_cardinality),
                    4 => self.mixed_union::<u32>(other, self_cardinality, other_cardinality),
                    _ => {
                        unreachable!();
                    }
                }
            }
            (false, true) => other.estimate_union_cardinality_with_cardinalities(
                self,
                self_cardinality,
                other_cardinality,
            ),
            (false, false) => self.inner.estimate_union_cardinality_with_cardinalities(
                &other.inner,
                self_cardinality,
                other_cardinality,
            ),
        }
    }
}

#[cfg(test)]
#[cfg(feature = "std")]
mod tests {
    use super::*;

    #[test]
    fn test_unique_count_from_sorted_iterators() {
        let number_of_iterations = 10;
        let mut random_state = splitmix64(3456789456776543);

        for _ in 0..number_of_iterations {
            random_state = splitmix64(random_state);
            let mut left = iter_var_len_random_values::<u64>(0, 1000, None, Some(random_state))
                .collect::<Vec<_>>();
            left.sort();
            random_state = splitmix64(random_state);
            let mut right = iter_var_len_random_values::<u64>(0, 1000, None, Some(random_state))
                .collect::<Vec<_>>();
            right.sort();

            let unique_values =
                unique_count_from_sorted_iterators(left.iter().cloned(), right.iter().cloned());
            let unique_values_set = u32::try_from(
                left.iter()
                    .chain(right.iter())
                    .collect::<std::collections::HashSet<_>>()
                    .len(),
            )
            .unwrap();
            assert_eq!(unique_values, unique_values_set);
        }
    }
}

#[cfg(test)]
mod test_hybrid_propertis {
    use super::*;
    use hyperloglog_derive::test_estimator;
    use twox_hash::XxHash;

    #[test_estimator]
    fn test_plusplus_hybrid_properties<P: Precision, B: Bits, R: Registers<P, B>, H: HasherType>() {
        let mut hybrid: Hybrid<PlusPlus<P, B, R, H>> = Default::default();
        assert!(hybrid.is_hybrid());
        assert!(hybrid.is_empty());
        assert!(!hybrid.is_full());
        assert!(hybrid.inner.get_number_of_zero_registers().is_zero());
        let mut normalized_error = 0.0;
        let mut non_normalized_error = 0.0;
        let mut random_state = 34567897654354_u64;
        let mut iterations = 0;

        while !hybrid.is_full() {
            iterations += 1;
            // To make the test a bit fairer using more random elements
            // than a numerical sequence.
            random_state = splitmix64(splitmix64(random_state));
            hybrid.insert(&random_state);
            assert!(hybrid.may_contain(&random_state));
            assert!(hybrid.is_hybrid());

            let estimated_cardinality = hybrid.estimate_cardinality();

            let error = iterations as f64 - estimated_cardinality;
            non_normalized_error += error;
            normalized_error += error / iterations as f64;
        }

        normalized_error /= iterations as f64;
        non_normalized_error /= iterations as f64;

        assert!(
            normalized_error <= P::error_rate() / 10.0,
            "The normalized error rate ({normalized_error}, {non_normalized_error}) must be less than or equal to the error rate ({}).",
            P::error_rate()
        );
        assert!(hybrid.is_full());
        assert!(!hybrid.is_empty());
        assert!(hybrid.is_hybrid());

        hybrid.insert(&0_128);

        assert!(!hybrid.is_hybrid());
    }
}

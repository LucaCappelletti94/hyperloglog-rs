//! Marker struct for the hybrid approach, that keeps the hash explicit up until they fit into the registers.

use crate::composite_hash::{CompositeHash, SaturationError, SwitchHash};
use crate::prelude::*;
use core::cmp::Ordering;
use core::fmt::Debug;
use core::hash::Hash;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "mem_dbg", derive(mem_dbg::MemDbg, mem_dbg::MemSize))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// A struct representing the hybrid for approximate set cardinality estimation,
/// where the hash values are kept explicit up until they fit into the registers.
pub struct Hybrid<
    H: HyperLogLog,
    CH = SwitchHash<<H as HyperLogLog>::Precision, <H as HyperLogLog>::Bits>,
> {
    /// The inner counter.
    inner: H,
    /// The composite hash used to encode the hashes.
    _composite_hash: core::marker::PhantomData<CH>,
}

impl<H: Hybridazable, CH: CompositeHash<Precision = H::Precision, Bits = H::Bits>> Default
    for Hybrid<H, CH>
where
    H: Default,
{
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<
        H: SetProperties + Hybridazable,
        CH: CompositeHash<Precision = H::Precision, Bits = H::Bits>,
    > SetProperties for Hybrid<H, CH>
{
    #[inline]
    fn is_empty(&self) -> bool {
        if self.is_hash_list() {
            self.inner.get_number_of_zero_registers().is_zero()
        } else {
            self.inner.is_empty()
        }
    }

    #[inline]
    fn is_full(&self) -> bool {
        if self.is_hash_list() {
            false
        } else {
            self.inner.is_full()
        }
    }
}

impl<
        T: Hash,
        H: ApproximatedSet<T> + Hybridazable,
        CH: CompositeHash<Precision = H::Precision, Bits = H::Bits>,
    > ApproximatedSet<T> for Hybrid<H, CH>
{
    #[inline]
    fn may_contain(&self, element: &T) -> bool {
        if self.is_hash_list() {
            let hash_bits = self.hash_bits();
            assert!(hash_bits >= CH::SMALLEST_VIABLE_HASH_BITS);
            let (index, register, original_hash) = H::index_and_register_and_hash(element);
            let writer_tell =
                usize::try_from(decode_writer_tell(self.inner.harmonic_sum())).unwrap();
            CH::find(
                self.inner.registers().as_ref(),
                self.inner.get_number_of_zero_registers().to_usize(),
                index,
                register,
                original_hash,
                self.hash_bits(),
                writer_tell,
            )
        } else {
            self.inner.may_contain(element)
        }
    }
}

impl<H: MutableSet + Hybridazable, CH: CompositeHash<Precision = H::Precision, Bits = H::Bits>>
    MutableSet for Hybrid<H, CH>
{
    #[inline]
    fn clear(&mut self) {
        self.inner.registers_mut().clear_registers();
        *self.inner.number_of_zero_registers_mut() =
            <H::Precision as Precision>::NumberOfRegisters::ZERO;
        *self.inner.harmonic_sum_mut() = f64::NEG_INFINITY;
        encode_target_hash(self.inner.harmonic_sum_mut(), 8);
    }
}

impl<
        T: Hash + Debug,
        H: ExtendableApproximatedSet<T> + Hybridazable,
        CH: CompositeHash<Precision = H::Precision, Bits = H::Bits>,
    > ExtendableApproximatedSet<T> for Hybrid<H, CH>
{
    #[inline]
    fn insert(&mut self, element: &T) -> bool {
        if self.is_hash_list() {
            let hash_bits = self.hash_bits();
            let number_of_hashes = self.inner.get_number_of_zero_registers().to_usize();
            let writer_tell = decode_writer_tell(self.inner.harmonic_sum());
            let hashes = self.inner.registers_mut().as_mut();
            let (index, register, original_hash) = H::index_and_register_and_hash(element);

            match CH::insert_sorted_desc(
                hashes,
                number_of_hashes,
                usize::try_from(writer_tell).unwrap(),
                index,
                register,
                original_hash,
                hash_bits,
            ) {
                Ok(inserted_position) => {
                    *self.inner.number_of_zero_registers_mut() +=
                        <H::Precision as Precision>::NumberOfRegisters::from(
                            inserted_position.is_some(),
                        );
                    if let Some(inserted_position) = inserted_position {
                        set_writer_tell(self.inner.harmonic_sum_mut(), u32::try_from(inserted_position).unwrap());
                    }

                    inserted_position.is_some()
                }
                Err(err) => match err {
                    SaturationError::DowngradableSaturation => {
                        self.downgrade();
                        debug_assert!(self.is_hash_list());
                        self.insert(element)
                    }
                    SaturationError::Saturation => {
                        self.dehybridize();
                        debug_assert!(!self.is_hash_list());
                        self.insert(element)
                    }
                },
            }
        } else {
            self.inner.insert(element)
        }
    }
}

#[inline]
/// Returns the number of unique shared values from two decreasingly sorted iterators.
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
pub fn intersection_from_sorted_iterators<T: Ord, I: Iterator<Item = T>, J: Iterator<Item = T>>(
    mut left: I,
    mut right: J,
) -> u32 {
    let mut intersection = u32::ZERO;
    let mut maybe_left_value = left.next();
    let mut maybe_right_value = right.next();
    while let (Some(left_value), Some(right_value)) =
        (maybe_left_value.as_ref(), maybe_right_value.as_ref())
    {
        let cmp = left_value.cmp(right_value);

        intersection += u32::from(cmp == Ordering::Equal);

        if cmp == Ordering::Equal || cmp == Ordering::Less {
            maybe_right_value = right.next();
        }
        if cmp == Ordering::Equal || cmp == Ordering::Greater {
            maybe_left_value = left.next();
        }
    }

    intersection
}

#[inline]
/// Returns the union estimation from a decreasingly sorted iterator and a counter.
///
/// # Implementative details
/// The provided iterator is expected to be sorted in ascending order,
/// in such a way that hash values that point to the same index are contiguos,
/// and ordered by value of the register as well.
pub fn union_estimation_from_sorted_iterator_and_counter<
    I: Iterator<Item = (u8, usize)>,
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

    for (left_register_value, index) in iterator {
        debug_assert!(
            index <= previous_index || previous_index == usize::MAX,
            "The index must be smaller than or equal to the previous index, but got {index} and {previous_index}",
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
impl<H: Named + HyperLogLog, CH: CompositeHash<Precision = H::Precision, Bits = H::Bits>> Named
    for Hybrid<H, CH>
{
    #[inline]
    fn name(&self) -> String {
        format!("H-{}", self.inner.name())
    }
}

const TARGET_HASH_MASK: u64 =
    0b0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0001_1111;

#[allow(unsafe_code)]
#[expect(
    clippy::transmute_ptr_to_ptr,
    reason = "We are transmuting a mutable reference to a mutable reference, which is safe."
)]
fn encode_target_hash(float: &mut f64, target_hash: u8) {
    debug_assert!((8..=32).contains(&target_hash));
    let harmonic_sum_as_u64: &mut u64 = unsafe { core::mem::transmute(float) };
    *harmonic_sum_as_u64 = (*harmonic_sum_as_u64 & !TARGET_HASH_MASK) | u64::from(target_hash - 8);
}

fn decode_target_hash(float: f64) -> u8 {
    u8::try_from(float.to_bits() & TARGET_HASH_MASK).unwrap() + 8
}

#[allow(unsafe_code)]
#[expect(
    clippy::transmute_ptr_to_ptr,
    reason = "We are transmuting a mutable reference to a mutable reference, which is safe."
)]
/// Adds the count of duplicates to the harmonic sum.
fn add_duplicates(float: &mut f64, new_duplicates: u32) {
    let expected_maximum_duplicates: u64 = 0xF_FFFF;
    let current_duplicates = decode_duplicates(*float);
    let new_duplicates = current_duplicates + new_duplicates;
    assert!(u64::from(new_duplicates) <= expected_maximum_duplicates);

    let harmonic_sum_as_u64: &mut u64 = unsafe { core::mem::transmute(float) };
    *harmonic_sum_as_u64 = (*harmonic_sum_as_u64 & !(expected_maximum_duplicates << 5))
        | (u64::from(new_duplicates) << 5);
}

fn decode_duplicates(float: f64) -> u32 {
    u32::try_from((float.to_bits() >> 5) & 0xF_FFFF).unwrap()
}

#[allow(unsafe_code)]
#[expect(
    clippy::transmute_ptr_to_ptr,
    reason = "We are transmuting a mutable reference to a mutable reference, which is safe."
)]
/// Sets the provided bit index to the harmonic sum.
fn set_writer_tell(float: &mut f64, bit_index: u32) {
    let expected_maximum_duplicates = 0xFF_FFFF;
    assert!(u64::from(bit_index) <= expected_maximum_duplicates);

    let harmonic_sum_as_u64: &mut u64 = unsafe { core::mem::transmute(float) };
    *harmonic_sum_as_u64 =
        (*harmonic_sum_as_u64 & !(expected_maximum_duplicates << 25)) | u64::from(bit_index) << 25;
}

fn decode_writer_tell(float: f64) -> u32 {
    u32::try_from(float.to_bits() >> 25 & 0xFF_FFFF).unwrap()
}

#[cfg(test)]
mod test_encode_decode_target_hash {
    use super::*;

    #[test]
    fn test_encode_decode_target_hash() {
        // The harmonic flag is initialized to minus infinity.
        let mut harmonic_sum = f64::NEG_INFINITY;
        for hash_bits in 8..=32 {
            encode_target_hash(&mut harmonic_sum, hash_bits);
            assert_eq!(decode_target_hash(harmonic_sum), hash_bits);
        }

        // We check that the harmonic sum has still a leading number of zeros
        // equal to zero, as we have initialized it to minus infinity and we
        // should not have touched those bits.
        assert_eq!(harmonic_sum.to_bits().leading_zeros(), 0);
    }

    #[test]
    fn test_encode_decode_duplicates() {
        let mut harmonic_sum = f64::NEG_INFINITY;
        add_duplicates(&mut harmonic_sum, 0);
        assert_eq!(decode_duplicates(harmonic_sum), 0);
        add_duplicates(&mut harmonic_sum, 1);
        assert_eq!(decode_duplicates(harmonic_sum), 1);
        add_duplicates(&mut harmonic_sum, 2);
        assert_eq!(decode_duplicates(harmonic_sum), 3);
        add_duplicates(&mut harmonic_sum, 3);
        assert_eq!(decode_duplicates(harmonic_sum), 6);
    }

    #[test]
    fn test_encode_decode_writer_tell() {
        let mut harmonic_sum = f64::NEG_INFINITY;
        set_writer_tell(&mut harmonic_sum, 0);
        assert_eq!(decode_writer_tell(harmonic_sum), 0);
        set_writer_tell(&mut harmonic_sum, 1);
        assert_eq!(decode_writer_tell(harmonic_sum), 1);
        set_writer_tell(&mut harmonic_sum, 2);
        assert_eq!(decode_writer_tell(harmonic_sum), 2);
        set_writer_tell(&mut harmonic_sum, 3);
        assert_eq!(decode_writer_tell(harmonic_sum), 3);
    }

    #[test]
    fn test_mixed_encode_decode_operations() {
        let mut harmonic_sum = f64::NEG_INFINITY;
        encode_target_hash(&mut harmonic_sum, 8);
        assert_eq!(decode_target_hash(harmonic_sum), 8);
        add_duplicates(&mut harmonic_sum, 1);
        assert_eq!(decode_duplicates(harmonic_sum), 1);
        set_writer_tell(&mut harmonic_sum, 1);
        assert_eq!(decode_target_hash(harmonic_sum), 8);
        assert_eq!(decode_duplicates(harmonic_sum), 1);
        assert_eq!(decode_writer_tell(harmonic_sum), 1);
        encode_target_hash(&mut harmonic_sum, 24);
        assert_eq!(decode_target_hash(harmonic_sum), 24);
        add_duplicates(&mut harmonic_sum, 1);
        assert_eq!(decode_duplicates(harmonic_sum), 2);
        set_writer_tell(&mut harmonic_sum, 10);
        assert_eq!(decode_target_hash(harmonic_sum), 24);
        assert_eq!(decode_duplicates(harmonic_sum), 2);
        assert_eq!(decode_writer_tell(harmonic_sum), 10);
    }
}

impl<H: Hybridazable, CH: CompositeHash<Precision = H::Precision, Bits = H::Bits>> Hybrid<H, CH> {
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
        encode_target_hash(inner.harmonic_sum_mut(), CH::LARGEST_VIABLE_HASH_BITS);

        Self {
            inner,
            _composite_hash: core::marker::PhantomData,
        }
    }

    #[inline]
    /// Returns whether the counter is in hybrid mode.
    pub fn is_hash_list(&self) -> bool {
        self.inner.harmonic_sum().to_bits().leading_zeros() == 0
    }

    #[inline]
    /// Returns the number of duplicates present in the hash list.
    ///
    /// # Errors
    /// Returns an error if the counter is not in hash list mode.
    pub fn duplicates(&self) -> Result<u32, &'static str> {
        if self.is_hash_list() {
            Ok(decode_duplicates(self.inner.harmonic_sum()))
        } else {
            Err("The counter is not in hash list mode.")
        }
    }

    #[inline]
    /// Returns the number of hashes in the hash list, not including the duplicates.
    ///
    /// # Errors
    /// Returns an error if the counter is not in hash list mode.
    pub fn number_of_hashes(&self) -> Result<u32, &'static str> {
        if self.is_hash_list() {
            Ok(self.inner.get_number_of_zero_registers())
        } else {
            Err("The counter is not in hash list mode.")
        }
    }

    #[inline]
    /// Returns the number of bits currently used for the hash.
    pub fn hash_bits(&self) -> u8 {
        debug_assert!(self.is_hash_list());
        let hash_bits = decode_target_hash(self.inner.harmonic_sum());
        debug_assert!(
            hash_bits >= CH::SMALLEST_VIABLE_HASH_BITS,
            "The number of bytes used for the hash ({hash_bits}) must be at least equal to the smallest viable hash ({})",
            CH::SMALLEST_VIABLE_HASH_BITS
        );
        hash_bits
    }

    #[inline]
    /// Returns an iterator over the current hashes if the counter is in hash list mode.
    /// The iterator is sorted in descending order.
    ///
    /// # Errors
    /// Returns an error if the counter is not in hash list mode.
    ///
    /// # Panics
    /// Panics if the number of bytes used for the hash is less than the smallest viable hash.
    pub fn hashes(&self) -> Result<CH::Downgraded<'_>, &'static str> {
        if self.is_hash_list() {
            let hash_bits = self.hash_bits();
            assert!(hash_bits >= CH::SMALLEST_VIABLE_HASH_BITS);
            let number_of_hashes = self.inner.get_number_of_zero_registers().to_usize();
            let hashes = self.inner.registers().as_ref();
            let writer_tell =
                usize::try_from(decode_writer_tell(self.inner.harmonic_sum())).unwrap();
            Ok(CH::downgraded(
                hashes,
                number_of_hashes,
                hash_bits,
                writer_tell,
                0,
            ))
        } else {
            Err("The counter is not in hash list mode.")
        }
    }

    #[inline]
    /// Converts the Hybrid counter to a regular [`HyperLogLog`] counter.
    fn dehybridize(&mut self) {
        debug_assert!(self.is_hash_list());
        debug_assert_eq!(self.hash_bits(), CH::SMALLEST_VIABLE_HASH_BITS);

        let mut new_counter: H = H::default();
        let hash_bits = self.hash_bits();
        let hashes = self.inner.registers().as_ref();
        let number_of_hashes = self.inner.get_number_of_zero_registers().to_usize();
        let writer_tell = usize::try_from(decode_writer_tell(self.inner.harmonic_sum())).unwrap();

        CH::decoded(hashes, number_of_hashes, hash_bits, writer_tell).for_each(
            |(new_register_value, index)| {
                new_counter.insert_register_value_and_index(new_register_value, index);
            },
        );

        *self = Self {
            inner: new_counter,
            _composite_hash: core::marker::PhantomData,
        };
    }

    #[inline]
    #[allow(unsafe_code)]
    /// Downgrades the Hybrid hashes one level.
    fn downgrade(&mut self) {
        debug_assert!(self.is_hash_list());

        let number_of_hashes = self.inner.get_number_of_zero_registers().to_usize();
        let current_hash_bits = self.hash_bits();
        let writer_tell = decode_writer_tell(self.inner.harmonic_sum());
        let target_hash_bits = CH::target_downgraded_hash_bits(
            number_of_hashes,
            usize::try_from(writer_tell).unwrap(),
            current_hash_bits,
        );

        let slice = self.inner.registers_mut().as_mut();

        let (new_duplicates, new_writer_tell) = CH::downgrade_inplace(
            slice,
            number_of_hashes,
            usize::try_from(writer_tell).unwrap(),
            current_hash_bits,
            current_hash_bits - target_hash_bits,
        );

        *self.inner.number_of_zero_registers_mut() -= unsafe {
            <H::Precision as Precision>::NumberOfRegisters::unchecked_from_u64(u64::from(
                new_duplicates,
            ))
        };

        set_writer_tell(
            self.inner.harmonic_sum_mut(),
            u32::try_from(new_writer_tell).unwrap(),
        );
        debug_assert_eq!(
            decode_writer_tell(self.inner.harmonic_sum()),
            u32::try_from(new_writer_tell).unwrap()
        );
        add_duplicates(self.inner.harmonic_sum_mut(), new_duplicates);
        encode_target_hash(self.inner.harmonic_sum_mut(), target_hash_bits);
        debug_assert_eq!(self.hash_bits(), target_hash_bits);
    }
}

impl<
        H: Clone + Correction + Estimator<f64> + Hybridazable + Default,
        CH: CompositeHash<Precision = H::Precision, Bits = H::Bits>,
    > Estimator<f64> for Hybrid<H, CH>
{
    #[inline]
    fn estimate_cardinality(&self) -> f64 {
        if self.is_hash_list() {
            CH::birthday_paradox_correction(
                self.number_of_hashes().unwrap() + self.duplicates().unwrap(),
            )
        } else {
            self.inner.estimate_cardinality()
        }
    }

    #[inline]
    fn is_union_estimate_non_deterministic(&self, other: &Self) -> bool {
        !(self.is_hash_list() && other.is_hash_list())
            && self.inner.is_union_estimate_non_deterministic(&other.inner)
    }

    #[inline]
    fn estimate_union_cardinality_with_cardinalities(
        &self,
        other: &Self,
        self_cardinality: f64,
        other_cardinality: f64,
    ) -> f64 {
        match (self.is_hash_list(), other.is_hash_list()) {
            (true, true) => {
                let left_hash_bits = self.hash_bits();
                let right_hash_bits = other.hash_bits();
                assert!(left_hash_bits >= CH::SMALLEST_VIABLE_HASH_BITS);
                assert!(right_hash_bits >= CH::SMALLEST_VIABLE_HASH_BITS);

                let left_shift = if left_hash_bits <= right_hash_bits {
                    0
                } else {
                    left_hash_bits - right_hash_bits
                };
                let right_shift = if right_hash_bits <= left_hash_bits {
                    0
                } else {
                    right_hash_bits - left_hash_bits
                };

                let left_hashes = self.inner.registers().as_ref();
                let right_hashes = other.inner.registers().as_ref();
                let left_bit_index =
                    usize::try_from(decode_writer_tell(self.inner.harmonic_sum())).unwrap();
                let right_bit_index =
                    usize::try_from(decode_writer_tell(other.inner.harmonic_sum())).unwrap();

                let intersection_cardinality = f64::from(intersection_from_sorted_iterators(
                    CH::downgraded(
                        left_hashes,
                        self.inner.get_number_of_zero_registers().to_usize(),
                        left_hash_bits,
                        left_bit_index,
                        left_shift,
                    ),
                    CH::downgraded(
                        right_hashes,
                        other.inner.get_number_of_zero_registers().to_usize(),
                        right_hash_bits,
                        right_bit_index,
                        right_shift,
                    ),
                ));

                let union_cardinality =
                    self_cardinality + other_cardinality - intersection_cardinality;

                correct_union_estimate(self_cardinality, other_cardinality, union_cardinality)
            }
            (true, false) => {
                let hash_bits = self.hash_bits();
                assert!(hash_bits >= CH::SMALLEST_VIABLE_HASH_BITS);
                let hashes = self.inner.registers().as_ref();
                let bit_index =
                    usize::try_from(decode_writer_tell(self.inner.harmonic_sum())).unwrap();

                assert!(CH::decoded(
                    hashes,
                    self.inner.get_number_of_zero_registers().to_usize(),
                    hash_bits,
                    bit_index,
                )
                .is_sorted_by(|a, b| { b.1 <= a.1 }));

                union_estimation_from_sorted_iterator_and_counter(
                    CH::decoded(
                        hashes,
                        self.inner.get_number_of_zero_registers().to_usize(),
                        hash_bits,
                        bit_index,
                    ),
                    &other.inner,
                    self_cardinality,
                    other_cardinality,
                )
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
    fn test_intersection_from_sorted_iterators() {
        let number_of_iterations = 10;
        let mut random_state = splitmix64(3456789456776543);

        for _ in 0..number_of_iterations {
            random_state = splitmix64(random_state);
            let mut left = iter_var_len_random_values::<u64>(0, 1000, None, Some(random_state))
                .collect::<Vec<_>>();
            left.sort_unstable_by(|a, b| b.cmp(a));
            random_state = splitmix64(random_state);
            let mut right = iter_var_len_random_values::<u64>(0, 1000, None, Some(random_state))
                .collect::<Vec<_>>();
            right.sort_unstable_by(|a, b| b.cmp(a));

            let intersection_cardinality =
                intersection_from_sorted_iterators(left.iter().cloned(), right.iter().cloned());
            let left_set = left.iter().collect::<std::collections::HashSet<_>>();
            let right_set = right.iter().collect::<std::collections::HashSet<_>>();
            let unique_values_set = left_set.intersection(&right_set).count() as u32;
            assert_eq!(intersection_cardinality, unique_values_set);
        }
    }
}

#[cfg(test)]
mod test_hybrid_propertis {
    use super::*;
    use hyperloglog_derive::test_estimator;
    use twox_hash::XxHash;

    #[test_estimator]
    fn test_plusplus_hybrid_properties<P: Precision, B: Bits, R: Registers<P, B>, H: HasherType>()
    where
        SwitchHash<P, B>: CompositeHash<Precision = P, Bits = B>,
    {
        let mut hybrid: Hybrid<PlusPlus<P, B, R, H>> = Default::default();
        assert!(hybrid.is_hash_list());
        assert!(hybrid.is_empty());
        assert!(!hybrid.is_full());
        assert!(hybrid.inner.get_number_of_zero_registers().is_zero());
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
                hybrid.hash_bits(),
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

        assert!(
            normalized_error <= P::error_rate() / 13.0,
            "The normalized error rate ({normalized_error}, {non_normalized_error}) must be less than or equal to the error rate ({}).",
            P::error_rate()
        );

        assert!(!hybrid.is_hash_list());
    }
}

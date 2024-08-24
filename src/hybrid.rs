//! Marker struct for the hybrid approach, that keeps the hash explicit up until they fit into the registers.

// use crate::composite_hash::current::CurrentHash;
use crate::composite_hash::switch::SwitchHash;
use crate::composite_hash::CompositeHash;
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
            self.will_dehybridize_upon_new_insert()
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
            let hash_bytes = self.hash_bytes();
            assert!(hash_bytes >= CH::SMALLEST_VIABLE_HASH_BITS / 8);
            let (index, register, original_hash) = H::hash_and_register_and_index(element);
            CH::find(
                self.inner.registers().as_ref(),
                self.inner.get_number_of_zero_registers().to_usize(),
                index,
                register,
                original_hash,
                self.hash_bytes() * 8,
            )
            .is_ok()
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
        encode_harmonic_flag(self.inner.harmonic_sum_mut(), 8);
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
            if self.will_dehybridize_upon_new_insert() {
                if self.may_contain(element) {
                    return false;
                }
                self.dehybridize();
                debug_assert!(!self.is_hash_list());
                self.insert(element)
            } else if self.will_downgrade_upon_new_insert() {
                if self.may_contain(element) {
                    return false;
                }
                self.downgrade();
                debug_assert!(self.is_hash_list());
                debug_assert!(!self.will_downgrade_upon_new_insert());
                self.insert(element)
            } else {
                let hash_bytes = self.hash_bytes();
                assert!(hash_bytes >= CH::SMALLEST_VIABLE_HASH_BITS / 8);
                let number_of_hashes = self.inner.get_number_of_zero_registers().to_usize();
                let hashes = self.inner.registers_mut().as_mut();
                let (index, register, original_hash) = H::hash_and_register_and_index(element);

                let inserted = CH::insert_sorted_desc(
                    hashes,
                    number_of_hashes,
                    index,
                    register,
                    original_hash,
                    hash_bytes * 8,
                );
                *self.inner.number_of_zero_registers_mut() +=
                    <H::Precision as Precision>::NumberOfRegisters::from(inserted);
                inserted
            }
        } else {
            self.inner.insert(element)
        }
    }
}

#[inline]
/// Returns the number of unique values from two decreasingly sorted iterators.
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
pub fn unique_count_from_sorted_iterators<T: Ord, I: Iterator<Item = T>, J: Iterator<Item = T>>(
    mut left: I,
    mut right: J,
    left_length: u32,
    right_length: u32,
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

    left_length + right_length - intersection
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
        encode_harmonic_flag(inner.harmonic_sum_mut(), 4);

        Self {
            inner,
            _composite_hash: core::marker::PhantomData,
        }
    }

    #[inline]
    /// Returns whether the counter is in hybrid mode.
    fn is_hash_list(&self) -> bool {
        self.inner.harmonic_sum().to_bits().leading_zeros() == 0
    }

    #[inline]
    /// Returns the maximum number of hashes that can be stored in the counter.
    pub fn maximal_number_of_hashes() -> usize {
        H::Registers::bitsize() / CH::SMALLEST_VIABLE_HASH_BITS as usize
    }

    #[inline]
    /// Returns the number of bytes currently used for the hash.
    pub fn hash_bytes(&self) -> u8 {
        debug_assert!(self.is_hash_list());
        let hash_bytes = decode_harmonic_flag(self.inner.harmonic_sum());
        debug_assert!(
            hash_bytes >= CH::SMALLEST_VIABLE_HASH_BITS / 8,
            "The number of bytes used for the hash ({hash_bytes}) must be at least equal to the smallest viable hash ({})",
            CH::SMALLEST_VIABLE_HASH_BITS / 8
        );
        hash_bytes
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
        let smallest_viable_hash = CH::SMALLEST_VIABLE_HASH_BITS / 8;
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
    pub fn will_dehybridize_upon_new_insert(&self) -> bool {
        self.is_hash_list() && self.inner.get_number_of_zero_registers().to_usize() == Self::maximal_number_of_hashes()
    }

    #[inline]
    /// Returns whether the hasher will have to downgrade the hash at the next insert.
    pub fn will_downgrade_upon_new_insert(&self) -> bool {
        self.is_hash_list() && self.inner.get_number_of_zero_registers().to_usize() == self.current_hash_capacity()
    }

    #[inline]
    /// Returns an iterator over the current hashes if the counter is in hash list mode.
    /// The iterator is sorted in descending order.
    pub fn hashes(&self) -> Result<CH::Downgraded<'_>, &'static str> {
        if self.is_hash_list() {
            let hash_bytes = self.hash_bytes();
            assert!(hash_bytes >= CH::SMALLEST_VIABLE_HASH_BITS / 8);
            let number_of_hashes = self.inner.get_number_of_zero_registers().to_usize();
            let hashes = self.inner.registers().as_ref();
            Ok(CH::downgraded(hashes, number_of_hashes, hash_bytes * 8, 0))
        } else {
            Err("The counter is not in hash list mode.")
        }
    }

    #[inline]
    /// Converts the Hybrid counter to a regular [`HyperLogLog`] counter.
    fn dehybridize(&mut self) {
        let hash_bytes = self.hash_bytes();
        assert!(hash_bytes >= CH::SMALLEST_VIABLE_HASH_BITS / 8);
        debug_assert!(self.is_hash_list());
        debug_assert!(self.will_dehybridize_upon_new_insert());
        debug_assert_eq!(
            self.current_hash_capacity(),
            Self::maximal_number_of_hashes()
        );
        debug_assert_eq!(self.hash_bytes(), CH::SMALLEST_VIABLE_HASH_BITS / 8);

        let mut new_counter: H = H::default();
        let hash_bits = self.hash_bytes() * 8;
        let hashes = self.inner.registers().as_ref();
        let number_of_hashes = self.inner.get_number_of_zero_registers().to_usize();

        CH::decoded(hashes, number_of_hashes, hash_bits).for_each(|(new_register_value, index)| {
            new_counter.insert_register_value_and_index(new_register_value, index);
        });

        *self = Self {
            inner: new_counter,
            _composite_hash: core::marker::PhantomData,
        };
    }

    #[inline]
    /// Downgrades the Hybrid hashes one level.
    fn downgrade(&mut self) {
        debug_assert!(self.is_hash_list());
        debug_assert!(self.will_downgrade_upon_new_insert());

        let number_of_hashes = self.inner.get_number_of_zero_registers().to_usize();
        let current_hash = self.hash_bytes();
        let current_hash_bits = current_hash * 8;
        let target_hash_bits = self.downgrade_maximal_hash_bytes() * 8;

        let slice = self.inner.registers_mut().as_mut();
        let slice_to_update = &mut slice[..number_of_hashes * usize::from(current_hash)];

        CH::downgrade_inplace(
            slice_to_update,
            number_of_hashes,
            current_hash_bits,
            current_hash_bits - target_hash_bits,
        );

        encode_harmonic_flag(self.inner.harmonic_sum_mut(), target_hash_bits / 8);
        debug_assert_eq!(self.hash_bytes(), target_hash_bits / 8);
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
            f64::from(self.inner.get_number_of_zero_registers())
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
                let left_hash_bytes = self.hash_bytes();
                let right_hash_bytes = other.hash_bytes();
                assert!(left_hash_bytes >= CH::SMALLEST_VIABLE_HASH_BITS / 8);
                assert!(right_hash_bytes >= CH::SMALLEST_VIABLE_HASH_BITS / 8);

                let self_number_of_hashes = self.inner.get_number_of_zero_registers().to_usize();
                let other_number_of_hashes = other.inner.get_number_of_zero_registers().to_usize();
                let left_shift = if left_hash_bytes <= right_hash_bytes {
                    0
                } else {
                    (left_hash_bytes - right_hash_bytes) * 8
                };
                let right_shift = if right_hash_bytes <= left_hash_bytes {
                    0
                } else {
                    (right_hash_bytes - left_hash_bytes) * 8
                };

                let left_hashes = self.inner.registers().as_ref();
                let right_hashes = other.inner.registers().as_ref();

                let union_cardinality = f64::from(unique_count_from_sorted_iterators(
                    CH::downgraded(
                        left_hashes,
                        self_number_of_hashes,
                        left_hash_bytes * 8,
                        left_shift,
                    ),
                    CH::downgraded(
                        right_hashes,
                        other_number_of_hashes,
                        right_hash_bytes * 8,
                        right_shift,
                    ),
                    self_number_of_hashes.try_into().unwrap(),
                    other_number_of_hashes.try_into().unwrap(),
                ));

                correct_union_estimate(self_cardinality, other_cardinality, union_cardinality)
            }
            (true, false) => {
                let self_hash = self.hash_bytes();
                assert!(self_hash >= CH::SMALLEST_VIABLE_HASH_BITS / 8);
                let number_of_hashes = self.inner.get_number_of_zero_registers().to_usize();
                let hash_bits = self.hash_bytes() * 8;
                let hashes = self.inner.registers().as_ref();

                assert!(CH::decoded(hashes, number_of_hashes, hash_bits)
                    .is_sorted_by(|a, b| { b.1 <= a.1 }));

                union_estimation_from_sorted_iterator_and_counter(
                    CH::decoded(hashes, number_of_hashes, hash_bits),
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
    fn test_unique_count_from_sorted_iterators() {
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

            let unique_values = unique_count_from_sorted_iterators(
                left.iter().cloned(),
                right.iter().cloned(),
                left.len().try_into().unwrap(),
                right.len().try_into().unwrap(),
            );
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
        assert!(hybrid.is_hash_list());
        assert!(hybrid.is_empty());
        assert!(!hybrid.is_full());
        assert!(hybrid.inner.get_number_of_zero_registers().is_zero());
        let mut normalized_error = 0.0;
        let mut non_normalized_error = 0.0;
        let mut random_state = 34567897654354_u64;
        let mut iterations = 0;

        loop {
            iterations += 1;
            // To make the test a bit fairer using more random elements
            // than a numerical sequence.
            random_state = splitmix64(splitmix64(random_state));
            hybrid.insert(&random_state);
            assert!(
                !hybrid.insert(&random_state),
                "The Hybrid counter should already contain the element {random_state}. Iteration n. {iterations}. Hash list status: {}",
                hybrid.is_hash_list()
            );
            assert!(
                hybrid.may_contain(&random_state),
                "The Hybrid counter must contain the element {random_state}. Iteration n. {iterations}.",
            );
            assert!(hybrid.is_hash_list());

            let estimated_cardinality = hybrid.estimate_cardinality();

            let error = iterations as f64 - estimated_cardinality;
            non_normalized_error += error;
            normalized_error += error / iterations as f64;

            if hybrid.will_dehybridize_upon_new_insert() {
                break;
            }
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
        assert!(hybrid.is_hash_list());

        hybrid.insert(&0_128);

        assert!(!hybrid.is_hash_list());
    }
}

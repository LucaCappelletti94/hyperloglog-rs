//! Marker struct for the hybrid approach, that keeps the hash explicit up until they fit into the registers.

use crate::prelude::*;
use core::cmp::Ordering;
use core::hash::Hash;
use core::marker::PhantomData;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "mem_dbg", derive(mem_dbg::MemDbg, mem_dbg::MemSize))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// A struct representing the hybrid for approximate set cardinality estimation,
/// where the hash values are kept explicit up until they fit into the registers.
pub struct Hybrid<H, CH = u32> {
    /// The inner counter.
    inner: H,
    /// The type of the composite hash to employ.
    composite_hash: PhantomData<CH>,
}

impl<H: Hybridazable<CH>, CH> Default for Hybrid<H, CH>
where
    H: Default,
    CH: CompositeHash<H::Precision, H::Bits>,
{
    #[inline]
    fn default() -> Self {
        Self {
            inner: H::new_hybrid(),
            composite_hash: PhantomData,
        }
    }
}

impl<
        H: SetProperties + HyperLogLog + Hybridazable<CH>,
        CH: CompositeHash<H::Precision, H::Bits>,
    > SetProperties for Hybrid<H, CH>
{
    #[inline]
    fn is_empty(&self) -> bool {
        if self.inner.is_hybrid() {
            self.inner.number_of_hashes() == 0
        } else {
            self.inner.is_empty()
        }
    }

    #[inline]
    fn is_full(&self) -> bool {
        if self.inner.is_hybrid() {
            self.inner.number_of_hashes() == self.inner.capacity()
        } else {
            self.inner.is_full()
        }
    }
}

impl<
        T: Hash,
        H: ApproximatedSet<T> + Hybridazable<CH>,
        CH: CompositeHash<H::Precision, H::Bits>,
    > ApproximatedSet<T> for Hybrid<H, CH>
where
    CH:,
{
    #[inline]
    fn may_contain(&self, element: &T) -> bool {
        if self.inner.is_hybrid() {
            Hybridazable::contains(&self.inner, element)
        } else {
            self.inner.may_contain(element)
        }
    }
}

impl<H: MutableSet + Hybridazable<CH>, CH: CompositeHash<H::Precision, H::Bits>> MutableSet
    for Hybrid<H, CH>
{
    #[inline]
    fn clear(&mut self) {
        self.inner.clear_words();
    }
}

impl<
        T: Hash,
        H: ExtendableApproximatedSet<T> + Hybridazable<CH>,
        CH: CompositeHash<H::Precision, H::Bits>,
    > ExtendableApproximatedSet<T> for Hybrid<H, CH>
{
    #[inline]
    fn insert(&mut self, element: &T) -> bool {
        if self.inner.is_hybrid() {
            Hybridazable::hybrid_insert(&mut self.inner, element)
        } else {
            self.inner.insert(element)
        }
    }
}

#[allow(unsafe_code)]
#[inline]
#[expect(clippy::cast_possible_truncation, reason = "The value is guaranteed to be less than 2**32")]
/// Method to convert a usize to a u32.
/// 
/// # Arguments
/// * `value` - The value to be converted.
/// 
/// # Safety
/// This method needs to be used with caution, as it will truncate values
/// that are greater than 2**32.
const unsafe fn usize_to_u32(value: usize) -> u32 {
    debug_assert!(value < (2_usize << 32), "The value should be less than 2**32");
    value as u32
}

#[inline]
#[allow(unsafe_code)]
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
pub fn unique_count_from_sorted_iterators<
    T: Ord,
    I: ExactSizeIterator<Item = T>,
    J: ExactSizeIterator<Item = T>,
>(
    mut left: I,
    mut right: J,
) -> u32 {
    let mut intersection = u32::ZERO;
    let left_length = unsafe { usize_to_u32(left.len()) };
    let right_length = unsafe { usize_to_u32(right.len()) };
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
    I: ExactSizeIterator<Item = (u8, <H::Precision as Precision>::NumberOfRegisters)>
        + DoubleEndedIterator,
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
    let mut previous_index = <H::Precision as Precision>::NUMBER_OF_REGISTERS;

    for (left_register_value, index) in iterator.rev() {
        debug_assert!(
            index <= previous_index,
            "The index must be less than or equal to the previous index."
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
        number_of_zeros -=
            <H::Precision as Precision>::NumberOfRegisters::from(right_register_value == 0);
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
pub trait Hybridazable<CH: CompositeHash<Self::Precision, Self::Bits>>: HyperLogLog {
    /// The type of the iterator over the sorted hashes.
    type IterSortedHashes<'words>: ExactSizeIterator<Item = CH::Word> + DoubleEndedIterator
    where
        CH: 'words,
        Self: 'words;

    /// Returns whether the struct is currently behaving as
    /// a hybrid counter.
    fn is_hybrid(&self) -> bool;

    /// Returns the capacity of the counter.
    fn capacity(&self) -> usize;

    /// De-hybridize the struct, i.e., convert it to a register-based counter.
    fn dehybridize(&mut self);

    /// Returns a new hybrid instance.
    fn new_hybrid() -> Self;

    /// Returns the number of hashes.
    fn number_of_hashes(&self) -> usize;

    /// Clears the counter.
    fn clear_words(&mut self);

    /// Returns an iterator over the sorted hashes.
    fn iter_sorted_hashes(&self) -> Self::IterSortedHashes<'_>;

    /// Returns whether the counter contains the element.
    fn contains<T: Hash>(&self, element: &T) -> bool;

    /// Inserts a value into the counter.
    fn hybrid_insert<T: Hash>(&mut self, value: &T) -> bool;
}

#[cfg(feature = "std")]
impl<H: Named, CH: Default + Named> Named for Hybrid<H, CH> {
    #[inline]
    fn name(&self) -> String {
        format!("H[{}]-{}", CH::default().name(), self.inner.name())
    }
}

impl<H: Hybridazable<CH> + HyperLogLog, CH: CompositeHash<H::Precision, H::Bits>>  Hybrid<H, CH>
{
    #[inline]
    /// Returns whether the counter is in hybrid mode.
    pub fn is_hybrid(&self) -> bool {
        self.inner.is_hybrid()
    }

    #[inline]
    /// Returns the maximum number of hashes that can be stored in the counter.
    pub fn capacity(&self) -> usize {
        self.inner.capacity()
    }
}

#[cfg(test)]
mod test_hybrid_propertis {
    use super::*;
    use hyperloglog_derive::test_estimator;
    use twox_hash::XxHash;
    use wyhash::WyHash;
    use ahash::AHasher;

    #[cfg(feature = "plusplus")]
    fn test_plusplus_hybrid_properties_for_word<
        P: Precision,
        B: Bits,
        R: Registers<P, B> + VariableWords<W>,
        H: HasherType,
        W: CompositeHash<P, B>,
    >() {
        let mut hybrid: Hybrid<PlusPlus<P, B, R, H>, W> = Default::default();
        assert!(hybrid.is_hybrid());
        assert!(hybrid.is_empty());
        assert_eq!(hybrid.inner.number_of_hashes(), 0);
        assert_eq!(
            hybrid.capacity(),
            <R as VariableWords<W>>::number_of_words(hybrid.inner.registers())
        );
        let mut normalized_error = 0.0;
        let mut count = 0;

        while !hybrid.is_full() {
            count += 1;
            hybrid.insert(&count);
            assert!(hybrid.is_hybrid());

            let estimated_cardinality = hybrid.estimate_cardinality();
            let number_of_hashes = hybrid.inner.number_of_hashes();
            assert_eq!(estimated_cardinality as usize, number_of_hashes);
            assert!(estimated_cardinality <= count as f64);

            let error = (estimated_cardinality - count as f64).abs();
            normalized_error += error / count as f64;
        }

        normalized_error /= hybrid.capacity() as f64;

        assert!(normalized_error <= P::error_rate() / 10.0);
        assert_eq!(hybrid.inner.number_of_hashes(), hybrid.capacity());
        assert!(hybrid.is_full());
        assert!(!hybrid.is_empty());
        assert!(hybrid.is_hybrid());

        hybrid.insert(&0_128);

        assert!(!hybrid.is_hybrid());
    }

    #[test_estimator]
    fn test_plusplus_hybrid_properties<
        P: Precision,
        B: Bits,
        R: Registers<P, B>
            + VariableWords<u32>
            + VariableWords<u40>
            + VariableWords<u48>
            + VariableWords<u56>
            + VariableWords<u64>,
        H: HasherType,
    >() {
        test_plusplus_hybrid_properties_for_word::<P, B, R, H, u32>();
        test_plusplus_hybrid_properties_for_word::<P, B, R, H, u40>();
        test_plusplus_hybrid_properties_for_word::<P, B, R, H, u48>();
        test_plusplus_hybrid_properties_for_word::<P, B, R, H, u56>();
        test_plusplus_hybrid_properties_for_word::<P, B, R, H, u64>();
    }
}

impl<
        H: Clone + Correction + Estimator<f64> + Hybridazable<CH> + Default,
        CH: CompositeHash<H::Precision, H::Bits>,
    > Estimator<f64> for Hybrid<H, CH>
where
    Hybrid<H, CH>: Default,
{
    #[inline]
    fn estimate_cardinality(&self) -> f64 {
        if self.inner.is_hybrid() {
            // We can safely convert this usize to an u32 because the maximal value that
            // can be stored in an Hybrid counter with the largest possible number of words
            // using the largest possible bit size (8) is 2**21 / 64 = 2**15, which fits
            // cosily in an u16.
            f64::from(u16::try_from(self.inner.number_of_hashes()).unwrap())
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
                // In the case where both counters are in hybrid mode, we can
                // simply iterate on the two sorted hash arrays and determine the number
                // of unique hashes.
                f64::from(unique_count_from_sorted_iterators(
                    self.inner.iter_sorted_hashes(),
                    other.inner.iter_sorted_hashes(),
                ))
            }
            (true, false) => union_estimation_from_sorted_iterator_and_counter(
                self.inner.iter_sorted_hashes().map(CH::decode),
                &other.inner,
                self_cardinality,
                other_cardinality,
            ),
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

    #[test]
    #[cfg(feature = "precision_10")]
    fn test_hybrid_plusplus() {
        let number_of_iterations = 10;
        let mut hybrid: Hybrid<
            PlusPlus<
                Precision10,
                Bits6,
                <Precision10 as ArrayRegister<Bits6>>::Array,
                twox_hash::XxHash64,
            >,
        > = Default::default();

        // The estimations up until the number of words is reached should be exact.
        for _ in 0..number_of_iterations {
            hybrid.clear();
            assert!(hybrid.is_empty());
            let cardinality: f64 = hybrid.estimate_cardinality();
            assert_eq!(cardinality, 0.0_f64);

            assert!(hybrid.is_hybrid());
            let mut exact_set = std::collections::HashSet::new();
            let mut random_state = splitmix64(3456789456776543);

            for element in iter_var_len_random_values::<u64>(0, 1000, None, Some(random_state)) {
                random_state = splitmix64(random_state);
                hybrid.insert(&element);
                exact_set.insert(element);
                assert!(hybrid.may_contain(&element));
                if !hybrid.is_hybrid() {
                    break;
                }
                let estimated_cardinality: f64 = hybrid.estimate_cardinality();
                assert_eq!(estimated_cardinality, exact_set.len() as f64);
            }
        }
    }
}

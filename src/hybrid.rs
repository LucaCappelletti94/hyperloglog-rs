//! Marker struct for the hybrid approach, that keeps the hash explicit up until they fit into the registers.

use crate::prelude::*;
use core::cmp::Ordering;
use core::hash::Hash;
use core::marker::PhantomData;

#[derive(Debug, Clone, Copy)]
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
    CH: CompositeHash<H::P, H::B>,
{
    #[inline]
    fn default() -> Self {
        Self {
            inner: H::new_hybrid(),
            composite_hash: PhantomData,
        }
    }
}

impl<H: Hybridazable<CH>, CH> Hybridazable<CH> for Hybrid<H, CH>
where
    H: Hybridazable<CH>,
    CH: CompositeHash<H::P, H::B>,
{
    type IterSortedHashes<'words> = H::IterSortedHashes<'words> where Self: 'words;
    type B = H::B;
    type P = H::P;

    #[inline]
    fn dehybridize(&mut self) {
        self.inner.dehybridize();
    }

    #[inline]
    fn new_hybrid() -> Self {
        Self::default()
    }

    #[inline]
    fn is_hybrid(&self) -> bool {
        self.inner.is_hybrid()
    }

    #[inline]
    fn iter_sorted_hashes(&self) -> Self::IterSortedHashes<'_> {
        self.inner.iter_sorted_hashes()
    }

    #[inline]
    fn number_of_hashes(&self) -> usize {
        self.inner.number_of_hashes()
    }

    #[inline]
    fn capacity(&self) -> usize {
        self.inner.capacity()
    }

    #[inline]
    fn clear_words(&mut self) {
        self.inner.clear_words();
    }

    #[inline]
    fn contains<T: Hash>(&self, element: &T) -> bool {
        self.inner.contains(element)
    }

    #[inline]
    fn hybrid_insert<T: Hash>(&mut self, value: &T) -> bool {
        self.inner.hybrid_insert(value)
    }
}

impl<H: PartialEq, CH> PartialEq<Self> for Hybrid<H, CH> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}

impl<H: PartialEq<H>, CH> PartialEq<H> for Hybrid<H, CH> {
    #[inline]
    fn eq(&self, other: &H) -> bool {
        &self.inner == other
    }
}

impl<H: Eq, CH> Eq for Hybrid<H, CH> {}

impl<H: SetProperties + Hybridazable<CH>, CH: CompositeHash<H::P, H::B>> SetProperties
    for Hybrid<H, CH>
{
    #[inline]
    fn is_empty(&self) -> bool {
        if self.is_hybrid() {
            self.inner.number_of_hashes() == 0
        } else {
            self.inner.is_empty()
        }
    }

    #[inline]
    fn is_full(&self) -> bool {
        if self.is_hybrid() {
            self.inner.number_of_hashes() == self.inner.capacity()
        } else {
            self.inner.is_full()
        }
    }
}

impl<T: Hash, H: ApproximatedSet<T> + Hybridazable<CH>, CH: CompositeHash<H::P, H::B>>
    ApproximatedSet<T> for Hybrid<H, CH>
where
    CH:,
{
    #[inline]
    fn may_contain(&self, element: &T) -> bool {
        if self.is_hybrid() {
            Hybridazable::contains(&self.inner, element)
        } else {
            self.inner.may_contain(element)
        }
    }
}

impl<H: MutableSet + Hybridazable<CH>, CH: CompositeHash<H::P, H::B>> MutableSet for Hybrid<H, CH> {
    #[inline]
    fn clear(&mut self) {
        self.inner.clear_words();
    }
}

impl<
        T: Hash,
        H: ExtendableApproximatedSet<T> + Hybridazable<CH>,
        CH: CompositeHash<H::P, H::B>,
    > ExtendableApproximatedSet<T> for Hybrid<H, CH>
{
    #[inline]
    fn insert(&mut self, element: &T) -> bool {
        if self.is_hybrid() {
            Hybridazable::hybrid_insert(&mut self.inner, element)
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
pub fn unique_count_from_sorted_iterators<T: Ord, I: ExactSizeIterator<Item = T>, J: ExactSizeIterator<Item = T>>(
    mut left: I,
    mut right: J,
) -> u32 {
    let mut intersection = u32::ZERO;
    let left_length = left.len() as u32;
    let right_length = right.len() as u32;
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

/// Trait for a struct that can be used in the hybrid approach.
pub trait Hybridazable<CH: CompositeHash<Self::P, Self::B>>: Default {
    /// The type of the iterator over the sorted hashes.
    type IterSortedHashes<'words>: ExactSizeIterator<Item = CH::Word>
    where
        CH: 'words,
        Self: 'words;
    /// The precision that needs to be reserved for the index in the composite hash.
    type P: Precision;
    /// The number of bits that need to be reserved for the index in the composite hash.
    type B: Bits;

    /// De-hybridize the struct, i.e., convert it to a register-based counter.
    fn dehybridize(&mut self);

    /// Returns a new hybrid instance.
    fn new_hybrid() -> Self;

    /// Returns whether the struct is currently behaving as
    /// a hybrid counter.
    fn is_hybrid(&self) -> bool;

    /// Returns the number of hashes currently inserted.
    fn number_of_hashes(&self) -> usize;

    /// Returns the capacity of the counter.
    fn capacity(&self) -> usize;

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

impl<H: Clone + Estimator<f64> + Hybridazable<CH> + Default, CH: CompositeHash<H::P, H::B>>
    Estimator<f64> for Hybrid<H, CH>
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
                    self.iter_sorted_hashes(),
                    other.iter_sorted_hashes(),
                ))
            }
            (true, false) => {
                let mut copy = self.clone();
                copy.dehybridize();
                copy.estimate_union_cardinality_with_cardinalities(
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

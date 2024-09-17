//! This module provides a function to compute the intersection of two sorted iterators.
use core::cmp::Ordering;

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
pub(crate) fn intersection_from_sorted_iterators<
    T: Ord,
    I: Iterator<Item = T>,
    J: Iterator<Item = T>,
>(
    mut left: I,
    mut right: J,
) -> u32 {
    let mut intersection = 0;
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

#[cfg(test)]
#[cfg(feature = "std")]
mod tests {
    use super::*;
    use crate::prelude::{iter_var_len_random_values, splitmix64};

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

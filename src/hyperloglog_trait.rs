use crate::estimated_union_cardinalities::EstimatedUnionCardinalities;
use crate::precisions::Precision;
use crate::prelude::*;
use crate::sip::hash_and_index;
use crate::utils::FloatNumber;
use core::hash::Hash;

pub trait HyperLogLogTrait<P: Precision, B: Bits>:
    Sized + Default + Eq + PartialEq + BitOrAssign<Self> + BitOr<Self, Output = Self>
{
    type IterRegisters<'a>: Iterator<Item = u32>
    where
        Self: 'a;

    fn adjust_estimate_with_zeros<F: FloatNumber>(
        harmonic_sum: F,
        number_of_zeros: P::NumberOfZeros,
    ) -> F
    where
        P: PrecisionConstants<F>,
    {
        if number_of_zeros >= P::LINEAR_COUNT_ZEROS {
            return P::small_correction(number_of_zeros);
        }
        P::adjust_estimate(harmonic_sum)
    }

    /// Returns an iterator over the registers of the HyperLogLog counter.
    fn iter_registers(&self) -> Self::IterRegisters<'_>;

    /// Returns the harmonic sum of the registers.
    fn harmonic_sum<F: FloatNumber>(&self) -> F
    where
        P: PrecisionConstants<F>;

    #[inline(always)]
    /// Estimates the cardinality of the set based on the HLL counter data.
    ///
    /// # Example
    ///
    /// ```
    /// # use hyperloglog_rs::prelude::*;
    /// let mut hll = HyperLogLog::<
    ///     Precision9,
    ///     Bits5,
    ///     <Precision9 as ArrayRegister<Bits5>>::ArrayRegister,
    /// >::default();
    /// let elements = vec![1, 2, 3, 4, 5];
    /// for element in &elements {
    ///     hll.insert(element);
    /// }
    /// let estimated_cardinality: f32 = hll.estimate_cardinality();
    /// assert!(
    ///     estimated_cardinality >= elements.len() as f32 * 0.9
    ///         && estimated_cardinality <= elements.len() as f32 * 1.1
    /// );
    /// ```
    ///
    /// # Returns
    /// * `f32` - The estimated cardinality of the set.
    fn estimate_cardinality<F: FloatNumber>(&self) -> F
    where
        P: PrecisionConstants<F>,
    {
        Self::adjust_estimate_with_zeros(self.harmonic_sum(), self.get_number_of_zero_registers())
    }

    #[inline(always)]
    /// Returns an estimate of the cardinality of the union of two HyperLogLog counters.
    ///
    /// This method calculates an estimate of the cardinality of the union of two HyperLogLog counters
    /// using the raw estimation values of each counter. It combines the estimation values by iterating
    /// over the register words of both counters and performing necessary calculations.
    ///
    /// # Arguments
    /// * `other`: A reference to the other HyperLogLog counter.
    ///
    /// # Returns
    /// An estimation of the cardinality of the union of the two HyperLogLog counters.
    ///
    /// # Example
    ///
    /// ```
    /// use hyperloglog_rs::prelude::*;
    ///
    /// let mut hll1 = HyperLogLog::<
    ///     Precision12,
    ///     Bits6,
    ///     <Precision12 as ArrayRegister<Bits6>>::ArrayRegister,
    /// >::default();
    /// hll1.insert(1);
    /// hll1.insert(2);
    ///
    /// let mut hll2 = HyperLogLog::<
    ///     Precision12,
    ///     Bits6,
    ///     <Precision12 as ArrayRegister<Bits6>>::ArrayRegister,
    /// >::default();
    /// hll2.insert(2);
    /// hll2.insert(3);
    ///
    /// let union_cardinality: f32 = hll1.estimate_union_cardinality(&hll2);
    ///
    /// assert!(union_cardinality >= 3.0 * 0.9 && union_cardinality <= 3.0 * 1.1);
    /// ```
    fn estimate_union_cardinality<F: FloatNumber, Rhs: HyperLogLogTrait<P, B>>(
        &self,
        other: &Rhs,
    ) -> F
    where
        P: PrecisionConstants<F>,
    {
        let mut harmonic_sum = F::ZERO;
        let mut union_zeros = P::NumberOfZeros::ZERO;

        for (left, right) in self.iter_registers().zip(other.iter_registers()) {
            let max_register = if left > right { left } else { right };
            harmonic_sum += F::inverse_register(max_register as i32);
            union_zeros += P::NumberOfZeros::from_bool(max_register.is_zero());
        }

        Self::adjust_estimate_with_zeros(harmonic_sum, union_zeros)
    }

    #[inline(always)]
    /// Returns an estimate of the cardinality of the two HLL counters union.
    fn estimate_union_and_sets_cardinality<F: FloatNumber, Rhs: HyperLogLogTrait<P, B>>(
        &self,
        other: &Rhs,
    ) -> EstimatedUnionCardinalities<F>
    where
        P: PrecisionConstants<F>,
    {
        EstimatedUnionCardinalities::with_correction(
            self.estimate_cardinality(),
            other.estimate_cardinality(),
            self.estimate_union_cardinality(other),
        )
    }

    #[inline(always)]
    /// Returns an estimate of the cardinality of the intersection of two HyperLogLog counters.
    ///
    /// This method calculates an estimate of the cardinality of the intersection of two HyperLogLog
    /// counters using the raw estimation values of each counter. It combines the estimation values by
    /// iterating over the register words of both counters and performing necessary calculations.
    ///
    /// # Arguments
    /// * `other`: A reference to the other HyperLogLog counter.
    ///
    /// # Returns
    /// An estimation of the cardinality of the intersection of the two HyperLogLog counters.
    ///
    /// # Example
    ///
    /// ```
    /// use hyperloglog_rs::prelude::*;
    ///
    /// let mut hll1 = HyperLogLog::<
    ///     Precision12,
    ///     Bits6,
    ///     <Precision12 as ArrayRegister<Bits6>>::ArrayRegister,
    /// >::default();
    /// hll1.insert(&1);
    /// hll1.insert(&2);
    ///
    /// let mut hll2 = HyperLogLog::<
    ///     Precision12,
    ///     Bits6,
    ///     <Precision12 as ArrayRegister<Bits6>>::ArrayRegister,
    /// >::default();
    /// hll2.insert(&2);
    /// hll2.insert(&3);
    ///
    /// let intersection_cardinality: f32 = hll1.estimate_intersection_cardinality(&hll2);
    ///
    /// assert!(intersection_cardinality >= 1.0 * 0.9 && intersection_cardinality <= 1.0 * 1.1);
    /// ```
    fn estimate_intersection_cardinality<F: FloatNumber>(&self, other: &Self) -> F
    where
        P: PrecisionConstants<F>,
    {
        self.estimate_union_and_sets_cardinality(other)
            .get_intersection_cardinality()
    }

    #[inline(always)]
    /// Returns an estimate of the Jaccard index between two HyperLogLog counters.
    ///
    /// The Jaccard index is a measure of similarity between two sets. In the context of HyperLogLog
    /// counters, it represents the ratio of the size of the intersection of the sets represented by
    /// the counters to the size of their union. This method estimates the Jaccard index by utilizing
    /// the cardinality estimation values of the intersection, left set, and right set.
    ///
    /// # Arguments
    /// * `other`: A reference to the other HyperLogLog counter.
    ///
    /// # Returns
    /// An estimation of the Jaccard index between the two HyperLogLog counters.
    ///
    /// # Example
    ///
    /// ```
    /// use hyperloglog_rs::prelude::*;
    ///
    /// let mut hll1 = HyperLogLog::<
    ///     Precision12,
    ///     Bits6,
    ///     <Precision12 as ArrayRegister<Bits6>>::ArrayRegister,
    /// >::default();
    /// hll1.insert(&1);
    /// hll1.insert(&2);
    /// hll1.insert(&3);
    /// hll1.insert(&4);
    ///
    /// let mut hll2 = HyperLogLog::<
    ///     Precision12,
    ///     Bits6,
    ///     <Precision12 as ArrayRegister<Bits6>>::ArrayRegister,
    /// >::default();
    /// hll2.insert(&2);
    /// hll2.insert(&3);
    /// hll2.insert(&5);
    /// hll2.insert(&6);
    ///
    /// let jaccard_index: f32 = hll1.estimate_jaccard_index(&hll2);
    ///
    /// let expected = 2.0 / 6.0;
    ///
    /// assert!(jaccard_index >= expected * 0.9 && jaccard_index <= expected * 1.1);
    /// ```
    fn estimate_jaccard_index<F: FloatNumber>(&self, other: &Self) -> F
    where
        P: PrecisionConstants<F>,
    {
        self.estimate_union_and_sets_cardinality(other)
            .get_jaccard_index()
    }

    #[inline(always)]
    /// Returns an estimate of the cardinality of the current HyperLogLog counter minus the provided one.
    ///
    /// # Arguments
    /// * `other`: A reference to the other HyperLogLog counter.
    ///
    /// # Example
    ///
    /// ```
    /// use hyperloglog_rs::prelude::*;
    ///
    /// let mut hll1 = HyperLogLog::<
    ///     Precision12,
    ///     Bits6,
    ///     <Precision12 as ArrayRegister<Bits6>>::ArrayRegister,
    /// >::default();
    /// hll1.insert(&1);
    /// hll1.insert(&2);
    /// hll1.insert(&3);
    /// hll1.insert(&4);
    ///
    /// let mut hll2 = HyperLogLog::<
    ///     Precision12,
    ///     Bits6,
    ///     <Precision12 as ArrayRegister<Bits6>>::ArrayRegister,
    /// >::default();
    /// hll2.insert(&2);
    /// hll2.insert(&3);
    /// hll2.insert(&5);
    /// hll2.insert(&6);
    ///
    /// let difference_cardinality: f32 = hll1.estimate_difference_cardinality(&hll2);
    ///
    /// assert!(difference_cardinality >= 2.0 * 0.9 && difference_cardinality <= 2.0 * 1.1);
    /// ```
    fn estimate_difference_cardinality<F: FloatNumber>(&self, other: &Self) -> F
    where
        P: PrecisionConstants<F>,
    {
        self.estimate_union_and_sets_cardinality(other)
            .get_left_difference_cardinality()
    }

    #[inline(always)]
    /// Returns whether no element was yet added to the HLL counter.
    ///
    ///
    /// # Examples
    ///
    /// ```
    /// use hyperloglog_rs::prelude::*;
    ///
    /// let mut hll: HyperLogLog<
    ///     Precision8,
    ///     Bits4,
    ///     <Precision8 as ArrayRegister<Bits4>>::ArrayRegister,
    /// > = HyperLogLog::default();
    ///
    /// assert!(hll.is_empty());
    ///
    /// hll.insert(&1);
    ///
    /// assert!(!hll.is_empty());
    /// ```
    fn is_empty(&self) -> bool {
        P::NUMBER_OF_REGISTERS
            == unsafe {
                self.get_number_of_zero_registers()
                    .try_into()
                    .unwrap_unchecked()
            }
    }

    /// Returns whether the HLL counter is full.
    ///
    /// A counter is considered full when all registers are maximally filled.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use hyperloglog_rs::prelude::*;
    ///
    /// let mut hll: HyperLogLog<
    ///     Precision4,
    ///     Bits4,
    ///     <Precision4 as ArrayRegister<Bits4>>::ArrayRegister,
    /// > = HyperLogLog::from_registers([u32::MAX; 2]);
    ///
    /// assert!(
    ///     hll.is_full(),
    ///     "1) The counter is not full: {:?}",
    ///     hll.harmonic_sum::<f64>()
    /// );
    ///
    /// let mut hll: HyperLogLog<
    ///     Precision10,
    ///     Bits4,
    ///     <Precision10 as ArrayRegister<Bits4>>::ArrayRegister,
    /// > = HyperLogLog::from_registers([u32::MAX; 128]);
    ///
    /// assert!(
    ///     hll.is_full(),
    ///     "2) The counter is not full: {:?}",
    ///     hll.harmonic_sum::<f64>()
    /// );
    ///
    /// let mut hll: HyperLogLog<
    ///     Precision4,
    ///     Bits4,
    ///     <Precision4 as ArrayRegister<Bits4>>::ArrayRegister,
    /// > = HyperLogLog::from_registers([1; 2]);
    ///
    /// assert!(!hll.is_full());
    ///
    /// let mut hll: HyperLogLog<
    ///     Precision10,
    ///     Bits4,
    ///     <Precision10 as ArrayRegister<Bits4>>::ArrayRegister,
    /// > = HyperLogLog::from_registers([1; 128]);
    ///
    /// assert!(!hll.is_full());
    ///
    /// let mut hll: HyperLogLog<
    ///     Precision10,
    ///     Bits4,
    ///     <Precision10 as ArrayRegister<Bits4>>::ArrayRegister,
    /// > = HyperLogLog::default();
    ///
    /// assert!(!hll.is_full());
    /// ```
    fn is_full(&self) -> bool;

    /// Returns the number of registers with zero values. This value is used for computing a small
    /// correction when estimating the cardinality of a small set.
    ///
    /// # Examples
    ///
    /// ```
    /// # use hyperloglog_rs::prelude::*;
    ///
    /// // Create a new HyperLogLog counter with precision 14 and 5 bits per register.
    /// let mut hll = HyperLogLog::<
    ///     Precision14,
    ///     Bits5,
    ///     <Precision14 as ArrayRegister<Bits5>>::ArrayRegister,
    /// >::default();
    ///
    /// // Add some elements to the counter.
    /// hll.insert(&1);
    /// hll.insert(&2);
    /// hll.insert(&3);
    ///
    /// // Get the number of zero registers.
    /// let number_of_zero_registers = hll.get_number_of_zero_registers();
    ///
    /// assert_eq!(number_of_zero_registers, 16381);
    /// ```
    fn get_number_of_zero_registers(&self) -> P::NumberOfZeros;

    #[inline(always)]
    /// Returns `true` if the HyperLogLog counter may contain the given element.
    ///
    /// # Arguments
    /// * `rhs` - The element to check.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use hyperloglog_rs::prelude::*;
    ///
    /// let mut hll: HyperLogLog<
    ///     Precision8,
    ///     Bits6,
    ///     <Precision8 as ArrayRegister<Bits6>>::ArrayRegister,
    /// > = HyperLogLog::default();
    /// assert_eq!(hll.may_contain(&42), false);
    ///
    /// hll.insert(&42);
    /// assert_eq!(hll.may_contain(&42), true);
    /// ```
    fn may_contain<T: Hash>(&self, rhs: &T) -> bool {
        self.get_register(hash_and_index::<T, P, B>(rhs).1) > 0
    }

    #[inline(always)]
    /// Returns whether the provided HyperLogLog counter may be fully contained in the current HyperLogLog counter.
    ///
    /// # Arguments
    /// * `rhs` - The HyperLogLog counter to check.
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
    /// let mut hll1: HyperLogLog<
    ///     Precision8,
    ///     Bits6,
    ///     <Precision8 as ArrayRegister<Bits6>>::ArrayRegister,
    /// > = HyperLogLog::default();
    /// let mut hll2: HyperLogLog<
    ///     Precision8,
    ///     Bits6,
    ///     <Precision8 as ArrayRegister<Bits6>>::ArrayRegister,
    /// > = HyperLogLog::default();
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
    fn may_contain_all<Rhs: HyperLogLogTrait<P, B>>(&self, rhs: &Rhs) -> bool {
        self.iter_registers()
            .zip(rhs.iter_registers())
            .all(|(lhs, rhs)| lhs >= rhs)
    }

    /// Insert a value into the HyperLogLog counter.
    ///
    /// # Arguments
    /// * `value` - A reference to the value to be inserted.
    ///
    /// # Returns
    /// Whether the counter has changed after the insertion.
    fn insert<T: Hash>(&mut self, value: T) -> bool;

    /// Return the value of the register at the given index.
    fn get_register(&self, index: usize) -> u32;

    /// Extend the HyperLogLog counter with the elements from an iterator.
    fn extend<I: IntoIterator<Item = T>, T: Hash>(&mut self, iter: I) {
        for value in iter {
            self.insert(&value);
        }
    }
}

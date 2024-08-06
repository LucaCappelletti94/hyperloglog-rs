use crate::precisions::Precision;
use crate::prelude::*;
use crate::sip::hash_and_index;
use crate::utils::FloatNumber;
use core::hash::Hash;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// A HyperLogLog counter with multiplicities.
///
/// # Implementation details
/// This struct differs from the traditional HyperLogLog counter in that it stores the multiplicities
/// of the registers. This allows us to speed up significantly the computation of the cardinality of
/// the counter, as we do not need to compute the harmonic mean of the registers but we can instead
/// use the multiplities instead, reducing by a large amount the sums we need to compute.
///
/// For instance, for a counter with 2^14 registers, we need to compute the harmonic mean of 2^14
/// registers, i.e. 16384 registers. With the multiplicities, we only need to compute the sum of the
/// multiplicities, which is much smaller, and at most equal to 52 when you use 6 bits per register.
///
/// That being said, when memory is an extreme concern, you may want to use the traditional HyperLogLog
/// as this struct contains the multiplicities vector, which in the example case we considered above
/// would be adding u16 * 52 = 104 bytes to the size of the counter.
///
/// Additionally, note that while one may expect to obtain better accuracy by executing less sums,
/// we do not observe any statistically significant difference in the accuracy of the counter when
/// using the multiplicities instead of the registers in our tests.
///
/// Note that this struct DOES NOT provide any other faster operation other than the estimation of the
/// cardinality of the counter. All other operations, such as the union of two counters, are fast as
/// they are implemented using the traditional HyperLogLog counter.
pub struct HLLMultiplicities<P: Precision, B: Bits, R: Registers<P, B>, M: Multiplicities<P, B>> {
    registers: R,
    multiplicities: M,
    _precision: core::marker::PhantomData<P>,
    _bits: core::marker::PhantomData<B>,
}

impl<P: Precision, B: Bits, R: Registers<P, B>, M: Multiplicities<P, B>> Default
    for HLLMultiplicities<P, B, R, M>
{
    fn default() -> Self {
        Self::new()
    }
}

impl<P: Precision, B: Bits, R: Registers<P, B>, M: Multiplicities<P, B>>
    HLLMultiplicities<P, B, R, M>
{
    fn new() -> Self {
        Self {
            registers: R::zeroed(),
            multiplicities: M::initialized(),
            _precision: core::marker::PhantomData,
            _bits: core::marker::PhantomData,
        }
    }

    pub(crate) fn multiplicities(&self) -> &M {
        &self.multiplicities
    }

    /// Create a new HyperLogLog counter from an array of registers.
    ///
    /// # Arguments
    ///
    /// * `registers` - An array of u32 registers to use for the HyperLogLog counter.
    ///
    /// # Returns
    ///
    /// A new HyperLogLog counter initialized with the given registers.
    ///
    /// # Examples
    ///
    /// ```
    /// use hyperloglog_rs::prelude::*;
    ///
    /// let registers = [0_u32; 4];
    /// let hll = HLLMultiplicities::<
    ///     Precision4,
    ///     Bits6,
    ///     <Precision4 as ArrayRegister<Bits6>>::ArrayRegister,
    ///     <Precision4 as ArrayMultiplicities<Bits6>>::ArrayMultiplicities,
    /// >::from_registers(registers);
    /// let collected_registers = hll.iter_registers().collect::<Vec<u32>>();
    /// assert_eq!(collected_registers, vec![0_u32; 1 << 4]);
    /// ```
    pub fn from_registers(registers: R) -> Self {
        let mut multiplicities = M::zeroed();

        for register in registers.iter_registers() {
            multiplicities.increment(register as usize);
        }

        Self {
            registers,
            multiplicities,
            _precision: core::marker::PhantomData,
            _bits: core::marker::PhantomData,
        }
    }
}

impl<P: Precision, B: Bits, R: Registers<P, B>, M: Multiplicities<P, B>>
    From<HLLMultiplicities<P, B, R, M>> for HyperLogLog<P, B, R>
{
    fn from(hll: HLLMultiplicities<P, B, R, M>) -> Self {
        Self::from_registers(hll.registers)
    }
}

impl<P: Precision, B: Bits, R: Registers<P, B>, M: Multiplicities<P, B>> HyperLogLogTrait<P, B>
    for HLLMultiplicities<P, B, R, M>
{
    type IterRegisters<'a> = <R as Registers<P, B>>::Iter<'a> where Self: 'a;

    #[inline(always)]
    /// Returns the number of registers in the counter.
    ///
    /// # Implementation details
    /// This function is overriding the estimate_cardinality function of the HyperLogLogTrait trait
    /// as we can compute the cardinality of the counter using the multiplicities instead of the
    /// registers. This is much faster as we do not need to compute the harmonic mean of the registers.
    fn estimate_cardinality<F: FloatNumber>(&self) -> F
    where
        P: PrecisionConstants<F>,
    {
        if !self.get_number_of_zero_registers().is_zero() {
            let low_range_correction = P::small_correction(self.get_number_of_zero_registers());
            if low_range_correction <= P::LINEAR_COUNT_THRESHOLD {
                return low_range_correction;
            }
        }

        P::adjust_estimate(
            self.multiplicities
                .iter_multiplicities()
                .enumerate()
                .map(|(current_register, multeplicity)| {
                    F::inverse_register_with_scalar(current_register as u32, multeplicity as u32)
                })
                .sum(),
        )
    }

    fn iter_registers(&self) -> Self::IterRegisters<'_> {
        self.registers.iter_registers()
    }

    #[inline(always)]
    /// Returns the number of registers with zero values. This value is used for computing a small
    /// correction when estimating the cardinality of a small set.
    ///
    /// # Examples
    ///
    /// ```
    /// # use hyperloglog_rs::prelude::*;
    ///
    /// // Create a new HyperLogLog counter with precision 14 and 5 bits per register.
    /// let mut hll = HLLMultiplicities::<
    ///     Precision14,
    ///     Bits5,
    ///     <Precision14 as ArrayRegister<Bits5>>::ArrayRegister,
    ///     <Precision14 as ArrayMultiplicities<Bits5>>::ArrayMultiplicities,
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
    fn get_number_of_zero_registers(&self) -> P::NumberOfZeros {
        self.multiplicities.get(0)
    }

    fn get_register(&self, index: usize) -> u32 {
        self.registers.get_register(index)
    }

    #[inline(always)]
    /// Adds an element to the HyperLogLog counter , and returns whether the counter has changed.
    ///
    /// # Arguments
    /// * `rhs` - The element to add.
    ///
    /// # Examples
    ///
    /// ```
    /// use hyperloglog_rs::prelude::*;
    ///
    /// let mut hll = HLLMultiplicities::<
    ///     Precision10,
    ///     Bits6,
    ///     <Precision10 as ArrayRegister<Bits6>>::ArrayRegister,
    ///     <Precision10 as ArrayMultiplicities<Bits6>>::ArrayMultiplicities,
    /// >::default();
    ///
    /// hll.insert("Hello");
    /// hll.insert("World");
    ///
    /// assert!(hll.estimate_cardinality::<f64>() >= 2.0);
    /// ```
    ///
    /// # Performance
    ///
    /// The performance of this function depends on the size of the HyperLogLog counter (`N`), the number
    /// of distinct elements in the input, and the hash function used to hash elements. For a given value of `N`,
    /// the function has an average time complexity of O(1) and a worst-case time complexity of O(log N).
    /// However, the actual time complexity may vary depending on the distribution of the hashed elements.
    ///
    /// # Errors
    ///
    /// This function does not return any errors.
    fn insert<T: Hash>(&mut self, rhs: T) -> bool {
        let (hash, index) = hash_and_index::<T, P, B>(&rhs);

        // Count leading zeros.
        let number_of_zeros: u32 = 1 + hash.leading_zeros();

        unsafe {
            if let Some(old_register) = self.registers.set_greater(index, number_of_zeros) {
                self.multiplicities.decrement(old_register as usize);
                self.multiplicities.increment(number_of_zeros as usize);
                true
            } else {
                false
            }
        }
    }
}

impl<
        F: FloatNumber,
        P: Precision + PrecisionConstants<F>,
        B: Bits,
        R: Registers<P, B>,
        M: Multiplicities<P, B>,
    > SetLike<F> for HLLMultiplicities<P, B, R, M>
{
    fn get_estimated_union_cardinality(
        &self,
        self_cardinality: F,
        other: &Self,
        other_cardinality: F,
    ) -> EstimatedUnionCardinalities<F> {
        let (raw_union_estimate, union_zeros) =
            self.iter_registers().zip(other.iter_registers()).fold(
                (F::ZERO, P::NumberOfZeros::ZERO),
                |(raw_union_estimate, union_zeros), (left, right)| {
                    let max_register = left.max(right);
                    (
                        raw_union_estimate + F::inverse_register(max_register),
                        union_zeros
                            + if max_register.is_zero() {
                                P::NumberOfZeros::ONE
                            } else {
                                P::NumberOfZeros::ZERO
                            },
                    )
                },
            );

        let union_estimate = Self::adjust_estimate_with_zeros(raw_union_estimate, union_zeros);

        EstimatedUnionCardinalities::with_correction(
            self_cardinality,
            other_cardinality,
            union_estimate,
        )
    }

    fn get_cardinality(&self) -> F {
        self.estimate_cardinality()
    }
}

impl<P: Precision, B: Bits, R: Registers<P, B>, M: Multiplicities<P, B>, A: Hash>
    core::iter::FromIterator<A> for HLLMultiplicities<P, B, R, M>
{
    fn from_iter<T: IntoIterator<Item = A>>(iter: T) -> Self {
        let mut hll = Self::new();
        hll.extend(iter);
        hll
    }
}

#[allow(clippy::suspicious_op_assign_impl)]
impl<
        P: Precision,
        B: Bits,
        R: Registers<P, B>,
        M: Multiplicities<P, B>,
        Rhs: HyperLogLogTrait<P, B>,
    > core::ops::BitOrAssign<Rhs> for HLLMultiplicities<P, B, R, M>
{
    #[inline(always)]
    fn bitor_assign(&mut self, rhs: Rhs) {
        let mut rhs_registers = rhs.iter_registers();
        let mut multiplicities = M::zeroed();

        self.registers.apply(|register| {
            let rhs_register: u32 = rhs_registers.next().unwrap();
            let new_register: u32 = register.max(rhs_register);
            multiplicities.increment(new_register as usize);

            new_register
        });

        self.multiplicities = multiplicities;
    }
}

impl<
        P: Precision,
        B: Bits,
        R: Registers<P, B>,
        M: Multiplicities<P, B>,
        Rhs: HyperLogLogTrait<P, B>,
    > core::ops::BitOr<Rhs> for HLLMultiplicities<P, B, R, M>
{
    type Output = Self;

    #[inline(always)]
    fn bitor(mut self, rhs: Rhs) -> Self {
        self.bitor_assign(rhs);
        self
    }
}

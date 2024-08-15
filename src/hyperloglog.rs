//! The `hyperloglog` module contains the `HyperLogLog` trait that defines the interface for HyperLogLog counters.
use crate::prelude::*;

/// Trait for HyperLogLog counters.
pub trait HyperLogLog<P: Precision, B: Bits, Hasher: core::hash::Hasher + Default>:
    Sized + Default + Eq + PartialEq + BitOrAssign<Self> + BitOr<Self, Output = Self> + Send + Sync + SetProperties + MutableSet
{
    /// The type of the registers of the HyperLogLog counter.
    type Registers: Registers<P, B>;

    /// Returns a reference to the registers of the HyperLogLog counter.
    fn registers(&self) -> &Self::Registers;

    /// Returns the harmonic sum of the registers.
    fn harmonic_sum<F: FloatNumber>(&self) -> F
    where
        P: PrecisionConstants<F>;

    /// Returns the number of registers with zero values. This value is used for computing a small
    /// correction when estimating the cardinality of a small set.
    ///
    /// # Examples
    ///
    /// ```
    /// # use hyperloglog_rs::prelude::*;
    ///
    /// // Create a new HyperLogLog counter with precision 14 and 5 bits per register.
    /// let mut hll = LogLogBeta::<
    ///     Precision6,
    ///     Bits5,
    ///     <Precision6 as ArrayRegister<Bits5>>::ArrayRegister,
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
    /// assert_eq!(number_of_zero_registers, 61);
    /// ```
    fn get_number_of_zero_registers(&self) -> P::NumberOfZeros;

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
    /// let mut hll1: PlusPlus<Precision8, Bits6, <Precision8 as ArrayRegister<Bits6>>::ArrayRegister> =
    ///     Default::default();
    /// let mut hll2: PlusPlus<Precision8, Bits6, <Precision8 as ArrayRegister<Bits6>>::ArrayRegister> =
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
    fn may_contain_all(&self, rhs: &Self) -> bool {
        self.registers()
            .iter_registers_zipped(rhs.registers())
            .all(|(lhs, rhs)| lhs >= rhs)
    }

    #[inline(always)]
    /// Slits the hash into two parts: the register value and the index of the register.
    fn split_hash(hash: u64) -> (u32, usize) {
        let index: usize = hash as usize & (P::NUMBER_OF_REGISTERS - 1);

        // And we delete the used bits from the hash.
        let mut hash: u64 = hash >> P::EXPONENT;

        debug_assert!(
            index < P::NUMBER_OF_REGISTERS,
            "The index {} must be less than the number of registers {}.",
            index,
            P::NUMBER_OF_REGISTERS
        );

        // We need to add ones to the hash to make sure that the
        // the number of zeros we obtain afterwards is never higher
        // than the maximal value that may be represented in a register
        // with BITS bits.
        if B::NUMBER_OF_BITS < 6 {
            hash |= 1 << (64 - ((1 << B::NUMBER_OF_BITS) - 1));
        }

        let register_value = hash.leading_zeros() + 1 - P::EXPONENT as u32;

        (register_value, index)
    }

    #[inline(always)]
    /// Hashes the element and returns the register value and the index of the register.
    fn hash_and_index<T: core::hash::Hash>(element: &T) -> (u32, usize) {
        let mut hasher = Hasher::default();
        element.hash(&mut hasher);
        let hash = hasher.finish();

        Self::split_hash(hash)
    }

    /// Return the value of the register at the given index.
    fn get_register(&self, index: usize) -> u32;

    /// Create a new HyperLogLog counter from an array of registers.
    fn from_registers(registers: Self::Registers) -> Self;
}

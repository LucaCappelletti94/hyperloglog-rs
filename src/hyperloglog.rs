//! The `hyperloglog` module contains the [`HyperLogLog`] trait that defines the interface for [`HyperLogLog`] counters.
use crate::prelude::*;
use core::hash::{Hash, Hasher};

/// Trait for [`HyperLogLog`] counters.
pub trait HyperLogLog<P: Precision, B: Bits, HS: Hasher + Default>:
    Sized
    + Default
    + Eq
    + PartialEq
    + BitOrAssign<Self>
    + BitOr<Self, Output = Self>
    + Send
    + Sync
    + SetProperties
    + MutableSet
{
    /// The type of the registers of the [`HyperLogLog`] counter.
    type Registers: Registers<P, B>;

    /// Returns a reference to the registers of the [`HyperLogLog`] counter.
    fn registers(&self) -> &Self::Registers;

    /// Returns the harmonic sum of the registers.
    fn harmonic_sum(&self) -> f64;

    /// Returns the number of registers with zero values. This value is used for computing a small
    /// correction when estimating the cardinality of a small set.
    ///
    /// # Examples
    ///
    /// ```
    /// #[cfg(feature = "beta")]
    /// {
    ///     use hyperloglog_rs::prelude::*;
    ///
    ///     // Create a new HyperLogLog counter with precision 14 and 5 bits per register.
    ///     let mut hll = LogLogBeta::<
    ///         Precision6,
    ///         Bits5,
    ///         <Precision6 as ArrayRegister<Bits5>>::Array,
    ///     >::default();
    ///
    ///     // Add some elements to the counter.
    ///     hll.insert(&1);
    ///     hll.insert(&2);
    ///     hll.insert(&3);
    ///
    ///     // Get the number of zero registers.
    ///     let number_of_zero_registers = hll.get_number_of_zero_registers();
    ///
    ///     assert_eq!(number_of_zero_registers, 61);
    /// }
    /// ```
    fn get_number_of_zero_registers(&self) -> P::NumberOfRegisters;

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
    /// let mut hll1: PlusPlus<Precision8, Bits6, <Precision8 as ArrayRegister<Bits6>>::Array> =
    ///     Default::default();
    /// let mut hll2: PlusPlus<Precision8, Bits6, <Precision8 as ArrayRegister<Bits6>>::Array> =
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
            .all(|[left_register, right_register]| left_register >= right_register)
    }

    #[must_use]
    #[inline]
    /// Slits the hash into two parts: the register value and the index of the register.
    fn split_hash(hash: u64) -> (u8, P::NumberOfRegisters) {
        let index: P::NumberOfRegisters = P::NumberOfRegisters::try_from_u64(
            hash & (P::NUMBER_OF_REGISTERS - P::NumberOfRegisters::ONE).into(),
        )
        .unwrap();

        debug_assert!(
            index < P::NUMBER_OF_REGISTERS,
            "The index {} must be less than the number of registers {}.",
            index,
            P::NUMBER_OF_REGISTERS
        );

        // And we censor we just used for the index.
        // let mut censored_hash: u64 = hash | 1 << P::EXPONENT;
        let mut censored_hash: u64 = hash;

        // We need to add ones to the hash to make sure that the
        // the number of zeros we obtain afterwards is never higher
        // than the maximal value that may be represented in a register
        // with BITS bits.
        if B::NUMBER_OF_BITS < 6_u8 {
            censored_hash |= 1_u64 << 64_u64 - B::MASK;
        }

        let register_value = u8::try_from(censored_hash.leading_zeros() + 1).unwrap();

        debug_assert!(
            register_value <= u8::try_from(B::MASK).unwrap(),
            "The register value {} must be less than or equal to the maximum register value {}.",
            register_value,
            (1 << B::NUMBER_OF_BITS) - 1
        );

        (register_value, index)
    }

    #[inline]
    /// Hashes the element and returns the register value and the index of the register.
    fn hash_and_index<T: Hash>(element: &T) -> (u8, P::NumberOfRegisters) {
        let mut hasher = HS::default();
        element.hash(&mut hasher);
        let hash = hasher.finish();

        Self::split_hash(hash)
    }

    /// Return the value of the register at the given index.
    fn get_register(&self, index: P::NumberOfRegisters) -> u8;

    /// Create a new [`HyperLogLog`] counter from an array of registers.
    fn from_registers(registers: Self::Registers) -> Self;
}

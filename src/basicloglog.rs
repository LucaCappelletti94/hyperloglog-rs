//! Implementation of the basic struct for [`HyperLogLog`] counter.

use crate::prelude::*;
use crate::utils::{HasherType, Number, One, PositiveInteger, Words, Zero};
use core::fmt::Debug;
use core::fmt::Formatter;
use core::hash::Hash;
use core::iter::{FromIterator, Take};
use core::marker::PhantomData;
use core::ops::{BitOr, BitOrAssign};

#[derive(Clone, Copy)]
#[cfg_attr(feature = "mem_dbg", derive(mem_dbg::MemDbg, mem_dbg::MemSize))]
/// A basic counter data structure for HyperLogLog-like counters.
pub(crate) struct BasicLogLog<P: Precision, B: Bits, R: Registers<P, B>, Hasher: HasherType> {
    /// The registers of the counter.
    registers: R,
    /// The number of registers with zero values.
    number_of_zero_registers: P::NumberOfRegisters,
    /// The harmonic sum of the registers, i.e. the sum of 2^(-register_value) for all registers.
    harmonic_sum: f64,
    /// Phantom data to ensure the type parameters are used.
    _phantom: PhantomData<(P, B, Hasher)>,
}

impl<P: Precision, B: Bits, R: Registers<P, B>, Hasher: HasherType> Debug
    for BasicLogLog<P, B, R, Hasher>
{
    fn fmt(&self, formatter: &mut Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("BasicLogLog")
            .field("registers", &self.registers)
            .field("number_of_zero_registers", &self.number_of_zero_registers)
            .field("harmonic_sum", &self.harmonic_sum)
            .finish()
    }
}

/// Implementation of partial equality for [`HyperLogLog`] so as to compare two [`HyperLogLog`] instances
/// ignoring the harmonic sum.
impl<P: Precision, B: Bits, R: Registers<P, B>, Hasher: HasherType> PartialEq
    for BasicLogLog<P, B, R, Hasher>
{
    fn eq(&self, other: &Self) -> bool {
        self.registers == other.registers
    }
}

/// Implementation of equality for [`HyperLogLog`] so as to compare two [`HyperLogLog`] instances
/// ignoring the harmonic sum.
impl<P: Precision, B: Bits, R: Registers<P, B>, Hasher: HasherType> Eq
    for BasicLogLog<P, B, R, Hasher>
{
}

impl<P: Precision, B: Bits, R: Registers<P, B>, Hasher: HasherType> BasicLogLog<P, B, R, Hasher> {
    /// Create a new [`HyperLogLog`] counter.
    fn new() -> Self {
        Self {
            registers: R::zeroed(),
            number_of_zero_registers: P::NUMBER_OF_REGISTERS,
            harmonic_sum: f64::integer_exp2(P::EXPONENT),
            _phantom: PhantomData,
        }
    }

    /// Computes the hash of a value and splits it into a register value and an index.
    fn compute_hash<T: Hash>(value: T) -> u64 {
        let mut hasher = Hasher::default();
        value.hash(&mut hasher);
        hasher.finish()
    }

    /// Splits a hash into a register value and an index.
    fn insert_register_value_and_index(
        &mut self,
        new_register_value: u8,
        index: P::NumberOfRegisters,
    ) -> bool {
        // Count leading zeros.
        debug_assert!(
            new_register_value < 1 << B::NUMBER_OF_BITS,
            "Register value is too large."
        );

        let (old_register_value, larger_register_value) =
            self.registers.set_greater(index, new_register_value);

        self.number_of_zero_registers -= P::NumberOfRegisters::from_bool(old_register_value == 0);

        self.harmonic_sum += f64::integer_exp2_minus(larger_register_value)
            - f64::integer_exp2_minus(old_register_value);

        old_register_value != new_register_value
    }
}

impl<P: Precision, B: Bits, R: Registers<P, B>, Hasher: HasherType> Default
    for BasicLogLog<P, B, R, Hasher>
{
    /// Returns a new [`HyperLogLog`] instance with default configuration settings.
    fn default() -> Self {
        Self::new()
    }
}

impl<P: Precision, B: Bits, R: Registers<P, B>, Hasher: HasherType> HyperLogLog<P, B, Hasher>
    for BasicLogLog<P, B, R, Hasher>
{
    type Registers = R;

    /// Returns the number of registers with zero values.
    fn get_number_of_zero_registers(&self) -> P::NumberOfRegisters {
        self.number_of_zero_registers
    }

    /// Returns a reference to the registers of the [`HyperLogLog`] counter.
    fn registers(&self) -> &Self::Registers {
        &self.registers
    }

    /// Returns the harmonic sum of the registers.
    fn harmonic_sum(&self) -> f64 {
        self.harmonic_sum
    }

    fn get_register(&self, index: P::NumberOfRegisters) -> u8 {
        self.registers.get_register(index)
    }

    fn from_registers(registers: R) -> Self {
        let mut number_of_zero_registers = P::NumberOfRegisters::ZERO;
        let mut harmonic_sum = f64::ZERO;

        for register in registers.iter_registers() {
            number_of_zero_registers += P::NumberOfRegisters::from_bool(register == 0);
            harmonic_sum += f64::integer_exp2_minus(register);
        }

        Self {
            registers,
            number_of_zero_registers,
            harmonic_sum,
            _phantom: PhantomData,
        }
    }
}

impl<P: Precision, B: Bits, R: Registers<P, B>, A: Hash, Hasher: HasherType> FromIterator<A>
    for BasicLogLog<P, B, R, Hasher>
{
    fn from_iter<T: IntoIterator<Item = A>>(iter: T) -> Self {
        let mut hll = Self::new();
        hll.extend(iter);
        hll
    }
}

impl<
        P: Precision,
        B: Bits,
        R: Registers<P, B>,
        Hasher: HasherType,
        Rhs: HyperLogLog<P, B, Hasher>,
    > BitOrAssign<Rhs> for BasicLogLog<P, B, R, Hasher>
{
    fn bitor_assign(&mut self, rhs: Rhs) {
        let mut rhs_registers = rhs.registers().iter_registers();

        self.registers.apply(|old_register| {
            let rhs_register: u8 = rhs_registers.next().unwrap();

            if rhs_register > old_register {
                self.harmonic_sum +=
                    f64::integer_exp2_minus(rhs_register) - f64::integer_exp2_minus(old_register);
                self.number_of_zero_registers -= P::NumberOfRegisters::from_bool(old_register == 0);
                rhs_register
            } else {
                old_register
            }
        });
    }
}

impl<
        P: Precision,
        B: Bits,
        R: Registers<P, B>,
        Rhs: HyperLogLog<P, B, Hasher>,
        Hasher: HasherType,
    > BitOr<Rhs> for BasicLogLog<P, B, R, Hasher>
{
    type Output = Self;

    fn bitor(mut self, rhs: Rhs) -> Self {
        self.bitor_assign(rhs);
        self
    }
}

impl<P: Precision, B: Bits, R: Registers<P, B> + Words, Hasher: HasherType> Hybridazable
    for BasicLogLog<P, B, R, Hasher>
{
    type IterSortedHashes<'words> = Take<R::WordIter<'words>> where Self: 'words;

    fn is_hybrid(&self) -> bool {
        // We employ the harmonic sum as a flag to represent whether the counter is in hybrid mode.
        self.harmonic_sum < f64::ZERO
    }

    fn new_hybrid() -> Self {
        let mut default: Self = BasicLogLog::default();
        default.clear_words();
        default
    }

    fn dehybridize(&mut self) {
        if self.is_hybrid() {
            let number_of_hashes = self.number_of_hashes();
            self.number_of_zero_registers = P::NUMBER_OF_REGISTERS;
            self.harmonic_sum = f64::integer_exp2(P::EXPONENT);
            let registers = self.registers.clone();
            self.registers = R::zeroed();
            for hash in registers.words().take(number_of_hashes) {
                let (register_value, index) = Self::split_hash(hash);
                self.insert_register_value_and_index(register_value, index);
            }
        }
    }

    fn number_of_hashes(&self) -> usize {
        self.get_number_of_zero_registers().to_usize()
    }

    fn capacity(&self) -> usize {
        self.registers.number_of_words()
    }

    fn clear_words(&mut self) {
        self.registers.clear();
        self.number_of_zero_registers = P::NumberOfRegisters::ZERO;
        self.harmonic_sum = f64::NEG_INFINITY;
    }

    fn iter_sorted_hashes(&self) -> Self::IterSortedHashes<'_> {
        self.registers.words().take(self.number_of_hashes())
    }

    fn contains<T: Hash>(&self, element: &T) -> bool {
        debug_assert!(
            self.number_of_hashes() <= self.registers.number_of_words(),
            "Number of hashes ({}) is greater than the number of words ({}) in the list of hashes.",
            self.number_of_hashes(),
            self.registers.number_of_words()
        );
        self.registers
            .find_sorted_with_len(Self::compute_hash(element), self.number_of_hashes())
    }

    fn hybrid_insert<T: Hash>(&mut self, value: &T) -> bool {
        // In hybrid setting, we are using the registers as a list of hashes
        // instead of the actual registers of an HyperLogLog counter, and we
        // use the number of zeros as the number of words in the list.

        if self.is_hybrid() {
            // If the counter in hybrid mode has reached saturation, i.e. has as many
            // hashes stored as it can fit, we switch to the normal HyperLogLog mode.
            if self.capacity() == self.number_of_hashes() {
                debug_assert_eq!(
                    self.number_of_hashes(),
                    self.registers.number_of_words(),
                    "Number of hashes ({}) is not equal to the number of words ({}) in the list of hashes.",
                    self.number_of_hashes(),
                    self.registers.number_of_words()
                );
                self.dehybridize();
                self.insert(value)
            } else {
                let hash = Self::compute_hash(value);
                if self
                    .registers
                    .sorted_insert_with_len(hash, self.number_of_hashes())
                {
                    debug_assert!(
                        self.number_of_zero_registers <= P::NUMBER_OF_REGISTERS,
                        "Number of zero registers ({}) is greater than the number of registers ({})",
                        self.number_of_zero_registers,
                        P::NUMBER_OF_REGISTERS
                    );
                    self.number_of_zero_registers += P::NumberOfRegisters::ONE;
                    true
                } else {
                    false
                }
            }
        } else {
            self.insert(value)
        }
    }
}

impl<P: Precision, B: Bits, Hasher: HasherType, R: Registers<P, B>> SetProperties
    for BasicLogLog<P, B, R, Hasher>
{
    fn is_empty(&self) -> bool {
        self.number_of_zero_registers == P::NUMBER_OF_REGISTERS
    }

    fn is_full(&self) -> bool {
        // The harmonic sum is defined as Sum(2^(-register_value)) for all registers.
        // When all registers are maximally filled, i.e. equal to the maximal multiplicity value,
        // the harmonic sum is equal to (2^(-max_multiplicity)) * number_of_registers.
        // Since number_of_registers is a power of 2, specifically 2^exponent, the harmonic sum
        // is equal to 2^(exponent - max_multiplicity).
        self.harmonic_sum
            <= f64::integer_exp2_minus_signed(
                i8::try_from(maximal_multeplicity(P::EXPONENT, B::NUMBER_OF_BITS)).unwrap()
                    - i8::try_from(P::EXPONENT).unwrap()
                    - 1,
            )
    }
}

impl<P: Precision, B: Bits, Hasher: HasherType, R: Registers<P, B>, T: Hash> ApproximatedSet<T>
    for BasicLogLog<P, B, R, Hasher>
{
    fn may_contain(&self, element: &T) -> bool {
        self.get_register(Self::hash_and_index::<T>(element).1) > 0
    }
}

impl<P: Precision, B: Bits, Hasher: HasherType, R: Registers<P, B>> MutableSet
    for BasicLogLog<P, B, R, Hasher>
{
    fn clear(&mut self) {
        self.registers.clear();
        self.number_of_zero_registers = P::NUMBER_OF_REGISTERS;
        self.harmonic_sum = f64::integer_exp2(P::EXPONENT);
    }
}

impl<P: Precision, B: Bits, Hasher: HasherType, R: Registers<P, B>, T: Hash>
    ExtendableApproximatedSet<T> for BasicLogLog<P, B, R, Hasher>
{
    fn insert(&mut self, element: &T) -> bool {
        let (new_register_value, index) = Self::hash_and_index::<T>(element);

        self.insert_register_value_and_index(new_register_value, index)
    }
}

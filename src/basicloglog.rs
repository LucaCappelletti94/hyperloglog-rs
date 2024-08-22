//! Implementation of the basic struct for [`HyperLogLog`] counter.

use crate::prelude::*;
use crate::utils::{HasherType, Zero};
use core::fmt::Debug;
use core::fmt::Formatter;
use core::hash::Hash;
use core::iter::FromIterator;
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

impl<P: Precision, B: Bits, R: Registers<P, B>, Hasher: HasherType>
    AsMut<BasicLogLog<P, B, R, Hasher>> for BasicLogLog<P, B, R, Hasher>
{
    fn as_mut(&mut self) -> &mut Self {
        self
    }
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
            registers: R::default(),
            number_of_zero_registers: P::NUMBER_OF_REGISTERS,
            harmonic_sum: f64::integer_exp2(P::EXPONENT),
            _phantom: PhantomData,
        }
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

impl<P: Precision, B: Bits, R: Registers<P, B>, Hasher: HasherType> HyperLogLog
    for BasicLogLog<P, B, R, Hasher>
{
    type Registers = R;
    type Precision = P;
    type Bits = B;
    type Hasher = Hasher;

    /// Returns the number of registers with zero values.
    fn get_number_of_zero_registers(&self) -> usize {
        self.number_of_zero_registers.to_usize()
    }

    /// Returns a reference to the registers of the [`HyperLogLog`] counter.
    fn registers(&self) -> &Self::Registers {
        &self.registers
    }

    /// Returns the harmonic sum of the registers.
    fn harmonic_sum(&self) -> f64 {
        self.harmonic_sum
    }

    fn get_register(&self, index: usize) -> u8 {
        self.registers.get_register(index)
    }

    /// Splits a hash into a register value and an index.
    fn insert_register_value_and_index(
        &mut self,
        new_register_value: u8,
        index: usize,
    ) -> bool {
        // Count leading zeros.
        debug_assert!(
            new_register_value <= u8::try_from(B::MASK).unwrap(),
            "Register value is too large."
        );
        debug_assert!(
            new_register_value > 0,
            "Register value is zero, which is not allowed."
        );

        let (old_register_value, larger_register_value) =
            self.registers.set_greater(index, new_register_value);

        self.number_of_zero_registers -= P::NumberOfRegisters::from(old_register_value == 0);

        self.harmonic_sum += f64::integer_exp2_minus(larger_register_value)
            - f64::integer_exp2_minus(old_register_value);

        old_register_value != new_register_value
    }

    fn from_registers(registers: R) -> Self {
        let mut number_of_zero_registers = P::NumberOfRegisters::ZERO;
        let mut harmonic_sum = f64::ZERO;

        for register in registers.iter_registers() {
            number_of_zero_registers += P::NumberOfRegisters::from(register == 0);
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

impl<P: Precision, B: Bits, R: Registers<P, B>, Hasher: HasherType, Rhs: HyperLogLog>
    BitOrAssign<Rhs> for BasicLogLog<P, B, R, Hasher>
{
    fn bitor_assign(&mut self, rhs: Rhs) {
        let mut rhs_registers = rhs.registers().iter_registers();

        self.registers.apply_to_registers(|old_register| {
            let rhs_register: u8 = rhs_registers.next().unwrap();

            if rhs_register > old_register {
                self.harmonic_sum +=
                    f64::integer_exp2_minus(rhs_register) - f64::integer_exp2_minus(old_register);
                self.number_of_zero_registers -= P::NumberOfRegisters::from(old_register == 0);
                rhs_register
            } else {
                old_register
            }
        });
    }
}

impl<P: Precision, B: Bits, R: Registers<P, B>, Rhs: HyperLogLog, Hasher: HasherType> BitOr<Rhs>
    for BasicLogLog<P, B, R, Hasher>
{
    type Output = Self;

    fn bitor(mut self, rhs: Rhs) -> Self {
        self.bitor_assign(rhs);
        self
    }
}

impl<H> Hybridazable for H
where
    H: HyperLogLog
        + AsMut<BasicLogLog<H::Precision, H::Bits, H::Registers, H::Hasher>>
        + Correction,
    H::Registers: HybridRegisters<H::Precision, H::Bits>,
{
    type Hashes = H::Registers;

    fn registers_mut(&mut self) -> &mut Self::Registers {
        &mut self.as_mut().registers
    }

    fn harmonic_sum_mut(&mut self) -> &mut f64 {
        &mut self.as_mut().harmonic_sum
    }

    fn number_of_zero_registers_mut(
        &mut self,
    ) -> &mut <H::Precision as Precision>::NumberOfRegisters {
        &mut self.as_mut().number_of_zero_registers
    }
}

impl<H> MutableSet for H
where
    H: HyperLogLog + AsMut<BasicLogLog<H::Precision, H::Bits, H::Registers, H::Hasher>>,
{
    fn clear(&mut self) {
        self.as_mut().registers.clear_registers();
        self.as_mut().number_of_zero_registers = H::Precision::NUMBER_OF_REGISTERS;
        self.as_mut().harmonic_sum = f64::integer_exp2(H::Precision::EXPONENT);
    }
}
